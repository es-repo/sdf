# Smooth Union

A smooth union combines two signed distance fields while rounding the seam where the shapes meet.

A signed distance field, or SDF, is a function that returns the distance from a point to a surface:

- Negative values mean the point is inside the shape.
- Positive values mean the point is outside the shape.
- Zero means the point is exactly on the surface.

For two SDF values, `d1` and `d2`, a normal union keeps the smaller distance:

:::code-tabs union
```rust
fn sdf_union(d1: f32, d2: f32) -> f32 {
    d1.min(d2)
}
```

```wgsl
fn sdf_union(d1: f32, d2: f32) -> f32 {
    return min(d1, d2);
}
```

```glsl
float sdfUnion(float d1, float d2) {
    return min(d1, d2);
}
```
:::

This works because the closest surface wins. The downside is that the transition between the shapes is sharp.

Smooth union replaces that sharp minimum with a blend around the intersection. The common polynomial form is:

:::code-tabs smooth-union
```rust
fn smooth_union(d1: f32, d2: f32, k: f32) -> f32 {
    let h = (0.5 + 0.5 * (d2 - d1) / k).clamp(0.0, 1.0);
    d2 * (1.0 - h) + d1 * h - k * h * (1.0 - h)
}
```

```wgsl
fn smooth_union(d1: f32, d2: f32, k: f32) -> f32 {
    let h = clamp(0.5 + 0.5 * (d2 - d1) / k, 0.0, 1.0);
    return mix(d2, d1, h) - k * h * (1.0 - h);
}
```

```glsl
float smoothUnion(float d1, float d2, float k) {
    float h = clamp(0.5 + 0.5 * (d2 - d1) / k, 0.0, 1.0);
    return mix(d2, d1, h) - k * h * (1.0 - h);
}
```
:::

The value `h` is the blend factor. When `d1` is clearly smaller, `h` is close to `1.0`. When `d2` is clearly smaller, `h` is close to `0.0`. Near the boundary between them, `h` moves smoothly between both values.

The final subtraction:

:::code-tabs blend-offset
```rust
k * h * (1.0 - h)
```

```wgsl
k * h * (1.0 - h)
```

```glsl
k * h * (1.0 - h)
```
:::

pushes the blended distance inward. That creates the rounded bridge instead of just cross-fading between the two fields.

The `k` parameter controls the blend radius:

- Small `k`: almost a hard union.
- Large `k`: wider, softer blending.

If you also want to blend material or color, return `h` together with the distance:

:::code-tabs smooth-union-with-blend
```rust
fn smooth_union_with_blend(d1: f32, d2: f32, k: f32) -> (f32, f32) {
    let h = (0.5 + 0.5 * (d2 - d1) / k).clamp(0.0, 1.0);
    let d = d2 * (1.0 - h) + d1 * h - k * h * (1.0 - h);
    (d, h)
}
```

```wgsl
fn smooth_union_with_blend(d1: f32, d2: f32, k: f32) -> vec2f {
    let h = clamp(0.5 + 0.5 * (d2 - d1) / k, 0.0, 1.0);
    let d = mix(d2, d1, h) - k * h * (1.0 - h);
    return vec2f(d, h);
}
```

```glsl
vec2 smoothUnionWithBlend(float d1, float d2, float k) {
    float h = clamp(0.5 + 0.5 * (d2 - d1) / k, 0.0, 1.0);
    float d = mix(d2, d1, h) - k * h * (1.0 - h);
    return vec2(d, h);
}
```
:::

Then `h` can be used to interpolate any extra data:

:::code-tabs color-blend
```rust
let color = color2 * (1.0 - h) + color1 * h;
```

```wgsl
let color = mix(color2, color1, h);
```

```glsl
vec3 color = mix(color2, color1, h);
```
:::
