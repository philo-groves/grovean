pub mod fb0;

pub struct Framebuffer {
  address: *mut u8,
  width: u64,
  height: u64,
  pitch: u64,
  bpp: u16,
}

impl Framebuffer {
  /// Create a new framebuffer
  pub fn new(address: *mut u8, width: u64, height: u64, pitch: u64, bpp: u16) -> Self {
    Self {
      address,
      width,
      height,
      pitch,
      bpp
    }
  }

  /// Set the background color of the framebuffer
  pub fn set_background(&self, color: u32) {
    for j in 0..self.height {
      let offset = (j * self.pitch) as isize;
      unsafe {
        let row = self.address.offset(offset) as *mut u32;
        for i in 0..self.width {
          *row.offset(i as isize) = color;
        }
      }
    }
  }

  /// Draw a pixel on the framebuffer
  pub fn draw_pixel(&self, x: u64, y: u64, color: u32) {
    let offset = (y * self.pitch + x * (self.bpp as u64 / 8)) as isize;
    unsafe {
      let pixel = self.address.offset(offset) as *mut u32;
      *pixel = color;
    }
  }

  /// Draw a rectangle on the framebuffer
  pub fn draw_rect(&self, x: u64, y: u64, width: u64, height: u64, color: u32) {
    let bytes_per_pixel = self.bpp as usize / 8;
    for j in 0..height {
      let offset = ((y + j) * self.pitch + x * bytes_per_pixel as u64) as isize;
      unsafe {
        let row = self.address.offset(offset) as *mut u32;
        for i in 0..width {
          *row.offset(i as isize) = color;
        }
      }
    }
  }

  /// Draw a line on the framebuffer
  pub fn draw_line(&self, x1: u64, y1: u64, x2: u64, y2: u64) {
    let dx = x2 as i64 - x1 as i64;
    let dy = y2 as i64 - y1 as i64;
    let dx1 = dx.abs();
    let dy1 = dy.abs();
    let mut px = 2 * dy1 - dx1;
    let mut py = 2 * dx1 - dy1;
    let (mut x, mut y): (i64, i64);
    if dy1 <= dx1 {
      if dx >= 0 {
        x = x1 as i64;
        y = y1 as i64;
      } else {
        x = x2 as i64;
        y = y2 as i64;
      }
      self.draw_pixel(x as u64, y as u64, 0xFFFFFFFF);
      for _i in 0..dx1 {
        if px < 0 {
          px = px + 2 * dy1;
        } else {
          if (dx < 0 && dy < 0) || (dx > 0 && dy > 0) {
            y = y + 1;
          } else {
            y = y - 1;
          }
          px = px + 2 * (dy1 - dx1);
        }
        x = x + 1;
        self.draw_pixel(x as u64, y as u64, 0xFFFFFFFF);
      }
    } else {
      if dy >= 0 {
        x = x1 as i64;
        y = y1 as i64;
      } else {
        x = x2 as i64;
        y = y2 as i64;
      }
      self.draw_pixel(x as u64, y as u64, 0xFFFFFFFF);
      for _i in 0..dy1 {
        if py <= 0 {
          py = py + 2 * dx1;
        } else {
          if (dx < 0 && dy < 0) || (dx > 0 && dy > 0) {
            x = x + 1;
          } else {
            x = x - 1;
          }
          py = py + 2 * (dx1 - dy1);
        }
        y = y + 1;
      }
    }
  }

  /// Get the address of the framebuffer
  pub fn get_address(&self) -> *mut u8 {
    self.address
  }

  /// Get the width of the framebuffer
  pub fn get_width(&self) -> u64 {
    self.width
  }

  /// Get the height of the framebuffer
  pub fn get_height(&self) -> u64 {
    self.height
  }

  /// Get the pitch of the framebuffer
  pub fn get_pitch(&self) -> u64 {
    self.pitch
  }

  /// Get the bits per pixel of the framebuffer
  pub fn get_bpp(&self) -> u16 {
    self.bpp
  }
}