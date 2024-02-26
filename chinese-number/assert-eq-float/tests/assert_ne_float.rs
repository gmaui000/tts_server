use assert_eq_float::assert_ne_float;

#[test]
fn basic() {
    assert_ne_float!(0.0f32, 1.0e-7f32);
    assert_ne_float!(0.0f64, 1.0e-15f64);
}

#[test]
fn default_error_ok() {
    assert_ne_float!(0.0, 1.0e-7);

    assert_ne_float!(1.0f32, 1.01f32);
    assert_ne_float!(1.1f32, 1.11f32);
    assert_ne_float!(100.001f32, 100.0011f32);

    assert_ne_float!(1.0f64, 1.01f64);
    assert_ne_float!(1.1f64, 1.11f64);
    assert_ne_float!(100.001f64, 100.0011f64);
    assert_ne_float!(-1001.1001f64, -1001.10011f64);

    assert_ne_float!(1.1 + 0.1, 1.21);

    assert_ne_float!(1e100 + 2e100, 3.1e100; "equal");
}

#[test]
fn custom_error_ok() {
    assert_ne_float!(0.0, 1.0e-7, 1e-7);

    assert_ne_float!(1.0f32, 1.01f32, 1e-7);
    assert_ne_float!(1.1f32, 1.11f32, 1e-7);
    assert_ne_float!(100.001f32, 100.0011f32, 1e-7);

    assert_ne_float!(1.0f64, 1.01f64, 1e-7);
    assert_ne_float!(1.1f64, 1.11f64, 1e-7);
    assert_ne_float!(100.001f64, 100.0011f64, 1e-7);
    assert_ne_float!(-1001.1001f64, -1001.10011f64, 1e-7);

    assert_ne_float!(1.1 + 0.1, 1.21, 1e-7);

    assert_ne_float!(1e100 + 2e100, 3e100, 1e-7);

    assert_ne_float!(1e100 + 2e100, 3.1e100, 1e90; "equal");
}

#[test]
#[should_panic]
fn custom_error_not_ok() {
    assert_ne_float!(1e100 + 2e100, 3e100);
}

#[test]
#[should_panic]
fn infinite_f32() {
    assert_ne_float!(f32::INFINITY, f32::INFINITY);
}

#[test]
#[should_panic]
fn infinite_f64() {
    assert_ne_float!(f64::INFINITY, f64::INFINITY);
}

#[test]
#[should_panic]
fn zero_f32() {
    assert_ne_float!(0.0f32, 0.0f32);
}

#[test]
#[should_panic]
fn zero_f64() {
    assert_ne_float!(0.0f64, 0.0f64);
}
