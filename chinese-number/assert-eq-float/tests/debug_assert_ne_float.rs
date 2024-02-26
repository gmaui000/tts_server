use assert_eq_float::debug_assert_ne_float;

#[test]
fn basic() {
    debug_assert_ne_float!(0.0, 1e-7);
    debug_assert_ne_float!(0.0, 1e-7; "eq");
    debug_assert_ne_float!(0.0, 1e-7, 1e-7);
    debug_assert_ne_float!(0.0, 1e-7, 1e-7; "eq");
}

#[cfg(debug_assertions)]
#[test]
#[should_panic]
fn should_panic_not_release() {
    debug_assert_ne_float!(0.0, 0.0);
}

#[cfg(not(debug_assertions))]
#[test]
fn should_panic_not_release() {
    debug_assert_ne_float!(0.0, 0.0);
}
