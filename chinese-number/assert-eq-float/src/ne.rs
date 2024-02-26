/// Asserts that two float expressions are not (approximately) equal to each other.
///
/// On panic, this macro will print the values of the expressions with their
/// debug representations.
///
/// Like [`assert_ne!`], this macro has a second form, where a custom
/// panic message can be provided.
///
/// # Examples
///
/// ```rust
/// use assert_eq_float::assert_ne_float;
///
/// let a = 3.01;
/// let b = 1.0 + 2.0;
///
/// assert_ne_float!(a, b);
///
/// assert_ne_float!(a, b; "we are testing addition with {} and {}", a, b);
///
/// assert_ne_float!(a, b, 1e-7); // use a fixed error
/// ```
///
/// Note that when setting a custom panic message, you should use a semicolon `;` instead of a comma `,`.
#[macro_export]
macro_rules! assert_ne_float {
    (@standard $left:ident, $right:ident, $error:ident) => {
        {
            use $crate::num_traits::float::FloatCore;

            let d_abs = ($left - $right).abs();

            if $left == $right || d_abs < $error {
                panic!(
                    "assertion failed: `(left != right)`\n  left: `{:?}`,\n right: `{:?}`\n  diff: `{:?}`\n error: `{:?}`",
                    $left,
                    $right,
                    d_abs,
                    $error,
                );
            }
        }
    };
    (@custom $left:ident, $right:ident, $error:ident; $($arg:tt)+) => {
        if $left == $right {
            panic!($($arg)+);
        }

        {
            use $crate::num_traits::float::FloatCore;

            let d_abs = ($left - $right).abs();

            assert!(
                d_abs >= $error,
                $($arg)+
            );
        }
    };
    ($left:expr, $right:expr $(,)?) => {{
        let left = $left;
        let right = $right;
        let error = $crate::get_error(left, right);

        $crate::assert_ne_float!(@standard left, right, error);
    }};
    ($left:expr, $right:expr; $($arg:tt)+) => {{
        let left = $left;
        let right = $right;
        let error = $crate::get_error(left, right);

        $crate::assert_ne_float!(@custom left, right, error; $($arg)+);
    }};
    ($left:expr, $right:expr, $error:expr $(,)?) => {{
        let left = $left;
        let right = $right;
        let error = $error;

        $crate::assert_ne_float!(@standard left, right, error);
    }};
    ($left:expr, $right:expr, $error:expr; $($arg:tt)+) => {{
        let left = $left;
        let right = $right;
        let error = $error;

        $crate::assert_ne_float!(@custom left, right, error; $($arg)+);
    }};
}

/// Asserts that two float expressions are not (approximately) equal to each other.
///
/// Unlike [`assert_ne_float!`], `debug_assert_ne_float!` statements are only enabled in non
/// optimized builds by default. An optimized build will not execute
/// `debug_assert_ne_float!` statements unless `-C debug-assertions` is passed to the
/// compiler. This makes `debug_assert_ne_float!` useful for checks that are too
/// expensive to be present in a release build but may be helpful during
/// development. The result of expanding `debug_assert_ne_float!` is always type checked.
#[macro_export]
macro_rules! debug_assert_ne_float {
    ($left:expr, $right:expr $(,)?) => {
        #[cfg(debug_assertions)]
        $crate::assert_ne_float!($left, $right);
    };
    ($left:expr, $right:expr; $($arg:tt)+) => {
        #[cfg(debug_assertions)]
        $crate::assert_ne_float!($left, $right; $($arg)+);
    };
    ($left:expr, $right:expr, $error:expr $(,)?) => {
        #[cfg(debug_assertions)]
        $crate::assert_ne_float!($left, $right, $error);
    };
    ($left:expr, $right:expr, $error:expr; $($arg:tt)+) => {
        #[cfg(debug_assertions)]
        $crate::assert_ne_float!($left, $right, $error; $($arg)+);
    };
}
