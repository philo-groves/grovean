use crate::dev::framebuffer::Framebuffer;
#[cfg(not(test))]
use font8x8::legacy::BASIC_LEGACY as FONT;
#[cfg(not(test))]
use limine::request::FramebufferRequest;

#[used]
#[cfg(not(test))]
#[unsafe(link_section = ".requests")]
static FRAMEBUFFER_REQUEST: FramebufferRequest = FramebufferRequest::new();

#[used]
pub static mut FRONT_BUFFER: Option<Framebufferterminal> = None;

pub fn with_front_buffer<F>(f: F)
where
    F: FnOnce(&mut Framebufferterminal),
{
    let front_buffer = core::ptr::addr_of_mut!(FRONT_BUFFER);
    unsafe {
        if let Some(framebuffer) = (*front_buffer).as_mut() {
            f(framebuffer);
        }
    }
}

/// Initialize the framebuffer terminal
#[cfg(not(test))]
pub fn init() {
    if let Some(framebuffer_response) = FRAMEBUFFER_REQUEST.get_response() {
        if let Some(framebuffer_metadata) = framebuffer_response.framebuffers().next() {
            let framebuffer = Framebuffer::new(
                framebuffer_metadata.addr() as *mut u8,
                framebuffer_metadata.width(),
                framebuffer_metadata.height(),
                framebuffer_metadata.pitch(),
                framebuffer_metadata.bpp(),
            );
            unsafe {
                FRONT_BUFFER = Some(Framebufferterminal::new(
                    framebuffer,
                    crate::dat::terminal::BACKGROUND,
                    FONT,
                ));
            }
        }
    }
}

pub struct Framebufferterminal {
    framebuffer: Framebuffer,
    background_color: u32,
    font: [[u8; 8]; 128],
    cursor_x: u64,
    cursor_y: u64,
    cell_width: u64,
    cell_height: u64,
}

impl Framebufferterminal {
    /// Create a new framebuffer terminal
    pub fn new(framebuffer: Framebuffer, background_color: u32, font: [[u8; 8]; 128]) -> Self {
        let cell_width = 8;
        let cell_height = 8;

        let terminal = Self {
            framebuffer,
            background_color,
            font,
            cursor_x: 0,
            cursor_y: 0,
            cell_width,
            cell_height,
        };
        terminal.clear_screen();
        terminal
    }

    /// Clear the framebuffer
    pub fn clear_screen(&self) {
        self.framebuffer.set_background(self.background_color);
    }

    /// Draw a character on the framebuffer
    pub fn draw_char(&self, x: u64, y: u64, ch: char, color: u32) {
        let index = ch as usize;
        let glyph = self.font[index];
        for (row, byte) in glyph.iter().enumerate() {
            for bit in 0..8 {
                if byte & (1 << (7 - bit)) != 0 {
                    self.framebuffer
                        .draw_pixel(x + 8 - bit, y + row as u64, color);
                }
            }
        }
    }

    /// Write a string to the framebuffer
    pub fn write_string(&mut self, s: &str, color: u32) {
        for ch in s.chars() {
            if ch == '\n' {
                self.cursor_x = 0;
                self.cursor_y += self.cell_height;
                if self.cursor_y >= self.framebuffer.height {
                    self.scroll();
                    self.cursor_y -= self.cell_height;
                }
            } else {
                self.draw_char(self.cursor_x, self.cursor_y, ch, color);
                self.cursor_x += self.cell_width;
                if self.cursor_x >= self.framebuffer.width {
                    self.cursor_x = 0;
                    self.cursor_y += self.cell_height;
                    if self.cursor_y >= self.framebuffer.height {
                        self.scroll();
                        self.cursor_y -= self.cell_height;
                    }
                }
            }
        }
    }

    /// Write a string to the framebuffer followed by a newline
    pub fn write_line(&mut self, s: &str, color: u32) {
        self.write_string(s, color);
        self.write_string("\n", color);
    }

    /// Scroll the framebuffer up by one cell height
    fn scroll(&mut self) {
        let row_size = self.framebuffer.pitch as usize * self.cell_height as usize;
        unsafe {
            core::ptr::copy(
                self.framebuffer.address.add(row_size),
                self.framebuffer.address,
                (self.framebuffer.height as usize - self.cell_height as usize)
                    * self.framebuffer.pitch as usize,
            );
            let offset = (self.framebuffer.height - self.cell_height) * self.framebuffer.pitch;
            let row = self.framebuffer.address.add(offset as usize) as *mut u32;
            for i in 0..self.framebuffer.width {
                *row.add(i as usize) = self.background_color;
            }
        }
    }
}

/// Write INFO string to the framebuffer
#[macro_export]
macro_rules! fb0_info {
  ($($arg:tt)*) => {
    $crate::dev::framebuffer::fb0::with_front_buffer(|fb| {
      fb.write_string($($arg)*, $crate::dat::terminal::ON_BACKGROUND);
    });
  };
}

/// Write INFO string to the framebuffer followed by a newline
#[macro_export]
macro_rules! fb0_info_ln {
  ($($arg:tt)*) => {
    $crate::dev::framebuffer::fb0::with_front_buffer(|fb| {
      fb.write_line(concat!("INFO: ", $($arg)*), $crate::dat::terminal::ON_BACKGROUND);
    });
  };
}

/// Write DEBUG string to the framebuffer
#[macro_export]
macro_rules! fb0_debug {
  ($($arg:tt)*) => {
    $crate::dev::framebuffer::fb0::with_front_buffer(|fb| {
      fb.write_string($($arg)*, $crate::dat::terminal::ACCENT);
    });
  };
}

/// Write DEBUG string to the framebuffer followed by a newline
#[macro_export]
macro_rules! fb0_debug_ln {
  ($($arg:tt)*) => {
    $crate::dev::framebuffer::fb0::with_front_buffer(|fb| {
      fb.write_line(concat!("DEBUG: ", $($arg)*), $crate::dat::terminal::ACCENT);
    });
  };
}

/// Write WARN string to the framebuffer
#[macro_export]
macro_rules! fb0_warn {
  ($($arg:tt)*) => {
    $crate::dev::framebuffer::fb0::with_front_buffer(|fb| {
      fb.write_string($($arg)*, $crate::dat::terminal::WARN);
    });
  };
}

/// Write WARN string to the framebuffer followed by a newline
#[macro_export]
macro_rules! fb0_warn_ln {
  ($($arg:tt)*) => {
    $crate::dev::framebuffer::fb0::with_front_buffer(|fb| {
      fb.write_line(concat!("WARN: ", $($arg)*), $crate::dat::terminal::WARN);
    });
  };
}

/// Write DANGER string to the framebuffer
#[macro_export]
macro_rules! fb0_danger {
  ($($arg:tt)*) => {
    $crate::dev::framebuffer::fb0::with_front_buffer(|fb| {
      fb.write_string($($arg)*, $crate::dat::terminal::DANGER);
    });
  };
}

/// Write DANGER string to the framebuffer followed by a newline
#[macro_export]
macro_rules! fb0_danger_ln {
  ($($arg:tt)*) => {
    $crate::dev::framebuffer::fb0::with_front_buffer(|fb| {
      fb.write_string(concat!("DANGER: ", $($arg)*), $crate::dat::terminal::DANGER);
    });
  };
}
