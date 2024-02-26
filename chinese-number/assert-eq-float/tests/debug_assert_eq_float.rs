use assert_eq_float::debug_assert_eq_float;

#[test]
fn basic() {
    debug_assert_eq_float!(0.0, 0.0);
    debug_assert_eq_float!(0.0, 0.0; "not eq");
    debug_assert_eq_float!(0.0, 0.0, 1e-7);
    debug_assert_eq_float!(0.0, 0.0, 1e-7; "not eq");
}

#[cfg(debug_assertions)]
#[test]
#[should_panic]
fn should_panic_not_release() {
    debug_assert_eq_float!(0.0, 0.1);
}

#[cfg(not(debug_assertions))]
#[test]
fn should_panic_not_release() {
    debug_assert_eq_float!(0.0, 0.1);
}
