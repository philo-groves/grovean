#![no_std]
#![no_main]
#![cfg_attr(target_arch = "x86_64", feature(abi_x86_interrupt))]
#![cfg_attr(test, feature(custom_test_frameworks))] // test setup: enable custom test frameworks
#![cfg_attr(test, test_runner(kunit::runner))] // test setup: use the custom test runner only in test mode
#![cfg_attr(test, reexport_test_harness_main = "test_main")] // test setup: rename the test harness entry point

#[cfg(test)]
kunit::klib!("grovean");

extern crate alloc;

pub mod allocator;
pub mod dat;
pub mod dev;

#[cfg(not(test))]
#[used]
#[unsafe(link_section = ".requests")]
static BASE_REVISION: limine::BaseRevision = limine::BaseRevision::new();

#[used]
#[unsafe(link_section = ".requests_start_marker")]
#[cfg(not(test))]
static _START_MARKER: limine::request::RequestsStartMarker =
    limine::request::RequestsStartMarker::new();

#[used]
#[unsafe(link_section = ".requests_end_marker")]
#[cfg(not(test))]
static _END_MARKER: limine::request::RequestsEndMarker = limine::request::RequestsEndMarker::new();

#[cfg(not(test))]
#[unsafe(no_mangle)]
pub unsafe extern "C" fn _start() -> ! {
    init();

    info_ln!("hello framebuffer");

    hlt_loop()
}

#[cfg(not(test))]
#[panic_handler]
fn rust_panic(_info: &core::panic::PanicInfo) -> ! {
    hlt_loop()
}

/// Initialize the kernel.
pub fn init() {
    #[cfg(not(test))]
    {
        assert!(BASE_REVISION.is_supported());
        dev::framebuffer::fb0::init();
    }
}

#[cfg(all(test, target_arch = "aarch64"))]
pub fn init_for_tests() {}

#[cfg(all(test, not(target_arch = "aarch64")))]
pub fn init_for_tests() {
    init();
}

/// Halt the CPU.
pub fn hlt_loop() -> ! {
    #[cfg(target_arch = "x86_64")]
    loop {
        x86_64::instructions::hlt();
    }

    #[cfg(target_arch = "aarch64")]
    loop {
        unsafe {
            core::arch::asm!("wfi", options(nomem, nostack, preserves_flags));
        }
    }
}

/// Prints INFO to serial and framebuffer terminals.
#[macro_export]
macro_rules! info {
    ($($arg:tt)*) => {
        fb0_info!($($arg)*);
        serial_info!($($arg)*);
    };
}

/// Prints INFO to serial and framebuffer terminals, followed by a newline.
#[macro_export]
macro_rules! info_ln {
    ($($arg:tt)*) => {
        fb0_info_ln!($($arg)*);
        serial_info_ln!($($arg)*);
    };
}

/// Prints DEBUG to serial and framebuffer terminals.
#[macro_export]
macro_rules! debug {
    ($($arg:tt)*) => {
        fb0_debug!($($arg)*);
        serial_debug!($($arg)*);
    };
}

/// Prints DEBUG to serial and framebuffer terminals, followed by a newline.
#[macro_export]
macro_rules! debug_ln {
    ($($arg:tt)*) => {
        fb0_debug_ln!($($arg)*);
        serial_debug_ln!($($arg)*);
    };
}

/// Prints WARN to serial and framebuffer terminals.
#[macro_export]
macro_rules! warn {
    ($($arg:tt)*) => {
        fb0_warn!($($arg)*);
        serial_warn!($($arg)*);
    };
}

/// Prints WARN to serial and framebuffer terminals, followed by a newline.
#[macro_export]
macro_rules! warn_ln {
    ($($arg:tt)*) => {
        fb0_warn_ln!($($arg)*);
        serial_warn_ln!($($arg)*);
    };
}

/// Prints DANGER to serial and framebuffer terminals.
#[macro_export]
macro_rules! danger {
    ($($arg:tt)*) => {
        fb0_danger!($($arg)*);
        serial_danger!($($arg)*);
    };
}

/// Prints DANGER to serial and framebuffer terminals, followed by a newline.
#[macro_export]
macro_rules! danger_ln {
    ($($arg:tt)*) => {
        fb0_danger_ln!($($arg)*);
        serial_danger_ln!($($arg)*);
    };
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u32)]
pub enum QemuExitCode {
    Success = 0x10,
    Failed = 0x11,
}

pub fn exit_qemu(exit_code: QemuExitCode) {
    #[cfg(target_arch = "x86_64")]
    {
        use x86_64::instructions::port::Port;

        unsafe {
            let mut port = Port::new(0xf4);
            port.write(exit_code as u32);
        }
    }

    #[cfg(target_arch = "aarch64")]
    {
        let _ = exit_code;
        hlt_loop();
    }
}

#[cfg(test)]
mod tests {
    use kunit::kunit;

    #[kunit]
    fn trivial_lib_assertion() {
        assert_eq!(1, 1);
    }
}
