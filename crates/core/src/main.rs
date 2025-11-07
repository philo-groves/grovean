#![no_std]
#![no_main]

#![feature(abi_x86_interrupt)]

#![cfg_attr(test, feature(custom_test_frameworks))]          // test setup: enable custom test frameworks
#![cfg_attr(test, test_runner(ktest::runner))]               // test setup: use the custom test runner only in test mode
#![cfg_attr(test, reexport_test_harness_main = "test_main")] // test setup: rename the test harness entry point

#[macro_use]
extern crate gk;

#[unsafe(no_mangle)]
unsafe extern "C" fn _start() -> ! {  
    gk::init();

    #[cfg(test)]
    {
        serial_info_ln!("!!! RUNNING BINARY TESTS !!!");
        ktest::init_harness("binary");
        test_main();
    }

    // individual device output
    fb0_info_ln!("hello framebuffer");
    serial_info_ln!("hello serial");

    // combined device output
    info_ln!("information for all devices");
    debug_ln!("debug for all devices");
    warn_ln!("warning for all devices");
    danger_ln!("danger for all devices");

    gk::hlt_loop();
}

#[cfg(not(test))]
#[panic_handler]
fn rust_panic(info: &core::panic::PanicInfo) -> ! {
    serial_danger_ln!("{}", info);
    gk::hlt_loop();
}

#[cfg(test)]
#[panic_handler]
fn rust_panic(info: &core::panic::PanicInfo) -> ! {
    ktest::panic(info);
}

#[cfg(test)]
mod tests {
    use ktest::ktest;

    #[ktest]
    fn trivial_main_assertion() {
        assert_eq!(1, 1);
    }
}