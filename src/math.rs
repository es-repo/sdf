use std::ops::{Add, Mul, Sub};

/// Returns the largest integer less than or equal to `x`.
pub fn floor_i32(x: f32) -> i32 {
    x.floor() as i32
}

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

/// Returns the interpolation parameter for `value` between `a` and `b`.
///
/// This is the inverse of `lerp`: `value = a` returns `0`, `value = b`
/// returns `1`, and values between them return points in between. The
/// result is clamped to `0..=1`; if `a == b`, this returns `0`.
pub fn unlerp(a: f32, b: f32, value: f32) -> f32 {
    if a == b {
        0.0
    } else {
        ((value - a) / (b - a)).clamp(0.0, 1.0)
    }
}

/// Returns the interpolation parameter for `value` between `0` and `max`.
///
/// This is equivalent to `unlerp(0.0, max, value)`: `value = 0` returns `0`,
/// `value = max` returns `1`, and values between them return points in between.
/// The result is clamped to `0..=1`; if `max == 0`, this returns `0`.
pub fn unlerp_unit(max: f32, value: f32) -> f32 {
    unlerp(0.0, max, value)
}
