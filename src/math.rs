use std::ops::{Add, Mul, Sub};

/// Linearly interpolates from `a` to `b` by `t`.
///
/// This is equivalent to `a + (b - a) * t`: `t = 0` returns `a`,
/// `t = 1` returns `b`, and values between them return points in between.
pub fn lerp<T, U>(a: T, b: T, t: U) -> T
where
    T: Copy + Add<Output = T> + Sub<Output = T> + Mul<U, Output = T>,
    U: Copy,
{
    a + (b - a) * t
}
