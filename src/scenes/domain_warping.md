# Domain Warping

Domain warping bends the input space before a shape or texture is evaluated.

For a normal circle SDF, each pixel coordinate is passed directly to the circle:

:::code-tabs circle-sdf
```rust
let dist = circle.dist(&coord);
```

```wgsl
let dist = circle_sdf(coord, center, radius);
```

```glsl
float dist = circleSdf(coord, center, radius);
```
:::

The circle boundary is where `dist == 0.0`. Negative distances are inside the circle, and positive distances are outside.

Domain warping changes the coordinate first:

:::code-tabs warped-circle-sdf
```rust
let warped_coord = coord + offset;
let dist = circle.dist(&warped_coord);
```

```wgsl
let warped_coord = coord + offset;
let dist = circle_sdf(warped_coord, center, radius);
```

```glsl
vec2 warpedCoord = coord + offset;
float dist = circleSdf(warpedCoord, center, radius);
```
:::

The circle SDF itself is still simple. The interesting part is the coordinate field that feeds it. If nearby pixels receive slightly different offsets, the original circle appears pulled, bent, and rippled.

## Noise Offset

This scene uses simplex noise layered with fractal Brownian motion, or FBM, to create the offset:

:::code-tabs domain-warping-offset
```rust
let noise_coord = coord * scale + time_scaled;
let offset = noise_coord.fbm(octaves, 0.5, 0.5, lacunarity, |coord| coord.noise_simplex())
    * warp_strength;
let warped_coord = coord + offset;
```

```wgsl
let noise_coord = coord * scale + time_scaled;
let offset = fbm(noise_coord, octaves, 0.5, 0.5, lacunarity) * warp_strength;
let warped_coord = coord + vec2f(offset);
```

```glsl
vec2 noiseCoord = coord * scale + timeScaled;
float offset = fbm(noiseCoord, octaves, 0.5, 0.5, lacunarity) * warpStrength;
vec2 warpedCoord = coord + vec2(offset);
```
:::

`noise_coord` is the coordinate used for the noise lookup. It is separate from the original `coord`, which is still the coordinate used to render the image.

The `time_scaled` term moves the noise field over time. Since the warped coordinate depends on that moving field, the circle deformation animates.

This demo uses the same scalar `offset` for both coordinates. That keeps the example small and makes the core idea easy to see. A fuller domain-warping field often uses separate noise samples for `x` and `y`:

:::code-tabs vector-offset
```rust
let offset = Vec2::new(noise_x, noise_y) * warp_strength;
let warped_coord = coord + offset;
```

```wgsl
let offset = vec2f(noise_x, noise_y) * warp_strength;
let warped_coord = coord + offset;
```

```glsl
vec2 offset = vec2(noiseX, noiseY) * warpStrength;
vec2 warpedCoord = coord + offset;
```
:::

## Parameters

`scale` controls the size of the noise features. It does not scale the circle directly. A small scale produces broad, slow bends. A large scale produces smaller, more frequent ripples.

`warp_strength` controls how far the coordinate is moved before evaluating the circle SDF. A value of `0.0` disables the warp. Larger values make the circle boundary bend more strongly.

`octaves` controls how many FBM layers are added together. One octave is smooth and simple. More octaves add finer detail on top of the broad deformation.

`lacunarity` controls how much the noise frequency increases between octaves. The default value is `2.0`, so each octave samples the noise field at twice the previous frequency.

## Why FBM Is Used

A single simplex-noise sample creates one smooth field. FBM combines several samples at increasing frequencies and decreasing amplitudes:

:::code-tabs fbm-shape
```rust
value += amplitude * noise(coord);
coord *= lacunarity;
amplitude *= gain;
```

```wgsl
value += amplitude * noise(coord);
coord *= lacunarity;
amplitude *= gain;
```

```glsl
value += amplitude * noise(coord);
coord *= lacunarity;
amplitude *= gain;
```
:::

Higher lacunarity values make the added FBM detail shrink faster from octave to octave. Lower values keep neighboring octaves closer in scale.

After arbitrary warping, the returned SDF value is no longer a perfect Euclidean distance to the visible boundary. For this scene, that is fine: the sign of the field still gives a useful inside/outside test, and the warped boundary is the visual effect.
