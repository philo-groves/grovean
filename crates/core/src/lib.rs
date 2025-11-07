#![no_std]
#![cfg_attr(test, no_main)]

#![feature(abi_x86_interrupt)]

#![cfg_attr(test, feature(custom_test_frameworks))]          // test setup: enable custom test frameworks
#![cfg_attr(test, test_runner(ktest::runner))]               // test setup: use the custom test runner only in test mode
#![cfg_attr(test, reexport_test_harness_main = "test_main")] // test setup: rename the test harness entry point

#[cfg(test)]
ktest::klib!("library", klib_config = &KLIB_CONFIG);

#[cfg(test)]
#[allow(dead_code)]
const KLIB_CONFIG: ktest::KlibConfig = ktest::KlibConfigBuilder::new_default()
    .before_tests(|| init())
    .build();

extern crate alloc;

pub mod allocator;
pub mod dat;
pub mod dev;

#[used]
#[unsafe(link_section = ".requests")]
static BASE_REVISION: limine::BaseRevision = limine::BaseRevision::new();

#[used]
#[unsafe(link_section = ".requests_start_marker")]
#[cfg(not(test))]
static _START_MARKER: limine::request::RequestsStartMarker = limine::request::RequestsStartMarker::new();

#[used]
#[unsafe(link_section = ".requests_end_marker")]
#[cfg(not(test))]
static _END_MARKER: limine::request::RequestsEndMarker = limine::request::RequestsEndMarker::new();

/// Initialize the kernel.
pub fn init() {
    assert!(BASE_REVISION.is_supported());
    dev::framebuffer::fb0::init();
}

/// Halt the CPU.
pub fn hlt_loop() -> ! {
    loop {
        x86_64::instructions::hlt();
    }
}

/// Prints INFO to serial and framebuffer terminals.
#[macro_export]
macro_rules! info {
    ($($arg:tt)*) => {
        serial_info!($($arg)*);
        fb0_info!($($arg)*);
    };
}

/// Prints INFO to serial and framebuffer terminals, followed by a newline.
#[macro_export]
macro_rules! info_ln {
    ($($arg:tt)*) => {
        serial_info_ln!($($arg)*);
        fb0_info_ln!($($arg)*);
    };
}

/// Prints DEBUG to serial and framebuffer terminals.
#[macro_export]
macro_rules! debug {
    ($($arg:tt)*) => {
        serial_debug!($($arg)*);
        fb0_debug!($($arg)*);
    };
}

/// Prints DEBUG to serial and framebuffer terminals, followed by a newline.
#[macro_export]
macro_rules! debug_ln {
    ($($arg:tt)*) => {
        serial_debug_ln!($($arg)*);
        fb0_debug_ln!($($arg)*);
    };
}

/// Prints WARN to serial and framebuffer terminals.
#[macro_export]
macro_rules! warn {
    ($($arg:tt)*) => {
        serial_warn!($($arg)*);
        fb0_warn!($($arg)*);
    };
}

/// Prints WARN to serial and framebuffer terminals, followed by a newline.
#[macro_export]
macro_rules! warn_ln {
    ($($arg:tt)*) => {
        serial_warn_ln!($($arg)*);
        fb0_warn_ln!($($arg)*);
    };
}

/// Prints DANGER to serial and framebuffer terminals.
#[macro_export]
macro_rules! danger {
    ($($arg:tt)*) => {
        serial_danger!($($arg)*);
        fb0_danger!($($arg)*);
    };
}

/// Prints DANGER to serial and framebuffer terminals, followed by a newline.
#[macro_export]
macro_rules! danger_ln {
    ($($arg:tt)*) => {
        serial_danger_ln!($($arg)*);
        fb0_danger_ln!($($arg)*);
    };
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u32)]
pub enum QemuExitCode {
    Success = 0x10,
    Failed = 0x11,
}

pub fn exit_qemu(exit_code: QemuExitCode) {
    use x86_64::instructions::port::Port;

    unsafe {
        let mut port = Port::new(0xf4);
        port.write(exit_code as u32);
    }
}

#[cfg(test)]
mod tests {
    use ktest::ktest;

    #[ktest]
    fn trivial_lib_assertion() {
        assert_eq!(1, 1);
    }
}
