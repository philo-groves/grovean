#![no_std]
#![cfg_attr(test, no_main)]

#![cfg_attr(test, feature(custom_test_frameworks))]          // test setup: enable custom test frameworks
#![cfg_attr(test, test_runner(ktest::runner))]               // test setup: use the custom test runner only in test mode
#![cfg_attr(test, reexport_test_harness_main = "test_main")] // test setup: rename the test harness entry point

#[cfg(test)]
ktest::klib!("extended-crate", klib_config = &KLIB_CONFIG);

#[cfg(test)]
const KLIB_CONFIG: ktest::KlibConfig = ktest::KlibConfigBuilder::new_default()
    .before_tests(|| init())
    .after_tests(|| teardown())
    .build();

#[allow(dead_code)]
fn init() {
    // some init here
}

#[allow(dead_code)]
fn teardown() {
    // some teardown here
}

#[cfg(test)]
mod tests {
    use ktest::ktest;

    #[ktest]
    fn trivial_basic_crate_assertion() {
        assert_eq!(2 + 2, 4);
    }
}
