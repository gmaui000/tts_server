use assert_eq_float::assert_eq_float;

#[test]
fn basic() {
    assert_eq_float!(0.0f32, 0.0f32);
    assert_eq_float!(0.0f64, 0.0f64);

    assert_eq_float!(f32::INFINITY, f32::INFINITY);
    assert_eq_float!(f64::INFINITY, f64::INFINITY);
}

#[test]
fn default_error_ok() {
    assert_eq_float!(0.0, 0.0);
    assert_eq_float!(f64::INFINITY, f64::INFINITY);
    assert_eq_float!(f64::INFINITY, f64::INFINITY);

    assert_eq_float!(1.0f32, 1.0f32);
    assert_eq_float!(1.1f32, 1.1f32);
    assert_eq_float!(100.001f32, 100.001f32);

    assert_eq_float!(1.0f64, 1.0f64);
    assert_eq_float!(1.1f64, 1.1f64);
    assert_eq_float!(100.001f64, 100.001f64);
    assert_eq_float!(-1001.1001f64, -1001.1001f64);

    assert_ne!(1.1 + 0.1, 1.2);
    assert_eq_float!(1.1 + 0.1, 1.2);

    assert_ne!(1e100 + 2e100, 3e100);
    assert_eq_float!(1e100 + 2e100, 3e100; "not equal");
}

#[test]
fn custom_error_ok() {
    assert_eq_float!(0.0, 0.0, 1e-7);
    assert_eq_float!(f64::INFINITY, f64::INFINITY, 1e-7);
    assert_eq_float!(f64::INFINITY, f64::INFINITY, 1e-7);

    assert_eq_float!(1.0f32, 1.0f32, 1e-7);
    assert_eq_float!(1.1f32, 1.1f32, 1e-7);
    assert_eq_float!(100.001f32, 100.001f32, 1e-7);

    assert_eq_float!(1.0f64, 1.0f64, 1e-7);
    assert_eq_float!(1.1f64, 1.1f64, 1e-7);
    assert_eq_float!(100.001f64, 100.001f64, 1e-7);
    assert_eq_float!(-1001.1001f64, -1001.1001f64, 1e-7);

    assert_ne!(1.1 + 0.1, 1.2);
    assert_eq_float!(1.1 + 0.1, 1.2, 1e-7);

    assert_ne!(1e100 + 2e100, 3e100);
    assert_eq_float!(1e100 + 2e100, 3e100, 1e90; "not equal");
}

#[test]
#[should_panic]
fn custom_error_not_ok() {
    assert_eq_float!(1e100 + 2e100, 3e100, 1e-7);
}
