/*!
# assert-eq-float

This crate provides the `assert_eq_float!` macros that support floats.

## Examples

```rust
use assert_eq_float::*;

assert_eq_float!(1.1 + 0.1, 1.2);       // error = 0.0000000000000021316282072803005
assert_eq_float!(1e100 + 2e100, 3e100); // error = 53290705182007510000000000000000000000000000000000000000000000000000000000000000000000

// other macros
debug_assert_eq_float!(0.0, 0.0);
assert_ne_float!(0.0, 0.1);
debug_assert_ne_float!(0.0, 0.1);
```

The default margin of error is dynamically computed by properties of IEEE 754 floating point numbers. You don't need to worry about it if you just want to check two float values are **equal**.
*/

#![no_std]

#[doc(hidden)]
pub extern crate num_traits;

mod eq;
mod ne;

use num_traits::float::FloatCore;

#[doc(hidden)]
#[inline]
pub fn get_error<T: FloatCore>(a: T, b: T) -> T {
    // See: https://magiclen.org/float-precision/
    a.abs().min(b.abs()) * T::epsilon() * T::from(8).unwrap()
}
