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

/// Smoothly interpolates from `0` to `1` between two edge values.
///
/// Values below `edge0` return `0`, values above `edge1` return `1`, and
/// values between them use a cubic curve with zero slope at both edges.
pub fn smoothstep(edge0: f32, edge1: f32, value: f32) -> f32 {
    let t = ((value - edge0) / (edge1 - edge0)).clamp(0.0, 1.0);

    t * t * (3.0 - 2.0 * t)
}

/// Returns the pair whose first value is smaller.
///
/// The second value is carried along unchanged. Floating-point values are
/// compared with `total_cmp`, so `NaN` ordering is deterministic.
pub fn min_pair<T>(a: (f32, T), b: (f32, T)) -> (f32, T) {
    if a.0.total_cmp(&b.0).is_le() { a } else { b }
}

/// Returns the pair with the smallest first value from an iterator.
///
/// Returns `None` when no pairs are provided. Floating-point values use the
/// same deterministic ordering as [`min_pair`].
pub fn min_pair_many<T, I>(pairs: I) -> Option<(f32, T)>
where
    I: IntoIterator<Item = (f32, T)>,
{
    pairs.into_iter().reduce(min_pair)
}

/// Returns the pair with the smallest first value from one or more arguments.
///
/// Unlike the [`min_pair_many`] function, this macro requires at least one
/// pair and returns the selected pair directly instead of wrapping it in
/// `Option`.
#[macro_export]
macro_rules! min_pair_many {
    ($first:expr $(, $rest:expr)* $(,)?) => {{
        let result = $first;
        $(
            let result = $crate::math::min_pair(result, $rest);
        )*
        result
    }};
}

/// Returns the pair whose first value is larger.
///
/// The second value is carried along unchanged. Floating-point values are
/// compared with `total_cmp`, so `NaN` ordering is deterministic.
pub fn max_pair<T>(a: (f32, T), b: (f32, T)) -> (f32, T) {
    if a.0.total_cmp(&b.0).is_ge() { a } else { b }
}

/// Returns the pair with the largest first value from an iterator.
///
/// Returns `None` when no pairs are provided. Floating-point values use the
/// same deterministic ordering as [`max_pair`].
pub fn max_pair_many<T, I>(pairs: I) -> Option<(f32, T)>
where
    I: IntoIterator<Item = (f32, T)>,
{
    pairs.into_iter().reduce(max_pair)
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
