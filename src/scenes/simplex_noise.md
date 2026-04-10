# 2D Simplex Noise

Simplex noise is a gradient-noise algorithm. It produces smooth pseudo-random values that change continuously as the input coordinate changes.

In 2D, simplex noise divides space into triangles. This is different from classic Perlin noise, which uses square grid cells. Triangles reduce directional artifacts and require fewer corner evaluations.

The algorithm has four main steps:

1. Skew the input coordinate so the triangular simplex grid can be addressed like a square grid.
2. Find the simplex cell that contains the point.
3. Pick deterministic gradient vectors for the cell corners.
4. Add the weighted contribution from each corner.

The skew constants are derived from equilateral triangle geometry:

:::code-tabs simplex-2d-constants
```rust
const F2: f32 = 0.3660254;  // (sqrt(3) - 1) / 2
const G2: f32 = 0.21132487; // (3 - sqrt(3)) / 6
```

```wgsl
const F2: f32 = 0.3660254;  // (sqrt(3) - 1) / 2
const G2: f32 = 0.21132487; // (3 - sqrt(3)) / 6
```

```glsl
const float F2 = 0.3660254;  // (sqrt(3) - 1) / 2
const float G2 = 0.21132487; // (3 - sqrt(3)) / 6
```
:::

First, skew the input point into simplex space and find the integer cell:

:::code-tabs simplex-2d-skew
```rust
let s = (x + y) * F2;
let i = (x + s).floor() as i32;
let j = (y + s).floor() as i32;
```

```wgsl
let s = (p.x + p.y) * F2;
let i = i32(floor(p.x + s));
let j = i32(floor(p.y + s));
```

```glsl
float s = (p.x + p.y) * F2;
int i = int(floor(p.x + s));
int j = int(floor(p.y + s));
```
:::

Then unskew the cell origin back into normal coordinate space:

:::code-tabs simplex-2d-unskew
```rust
let t = (i + j) as f32 * G2;
let x0 = x - (i as f32 - t);
let y0 = y - (j as f32 - t);
```

```wgsl
let t = f32(i + j) * G2;
let x0 = p.x - (f32(i) - t);
let y0 = p.y - (f32(j) - t);
```

```glsl
float t = float(i + j) * G2;
float x0 = p.x - (float(i) - t);
float y0 = p.y - (float(j) - t);
```
:::

The values `x0` and `y0` are the point's position relative to the first simplex corner.

In 2D, each simplex cell is split into two triangles. The second corner depends on which side of the diagonal the point is on:

:::code-tabs simplex-2d-corner-order
```rust
let (i1, j1) = if x0 > y0 {
    (1, 0)
} else {
    (0, 1)
};
```

```wgsl
let corner = select(vec2i(0, 1), vec2i(1, 0), x0 > y0);
let i1 = corner.x;
let j1 = corner.y;
```

```glsl
ivec2 corner = x0 > y0 ? ivec2(1, 0) : ivec2(0, 1);
int i1 = corner.x;
int j1 = corner.y;
```
:::

Now compute the offsets from the point to the three triangle corners:

:::code-tabs simplex-2d-corner-offsets
```rust
let x1 = x0 - i1 as f32 + G2;
let y1 = y0 - j1 as f32 + G2;
let x2 = x0 - 1.0 + 2.0 * G2;
let y2 = y0 - 1.0 + 2.0 * G2;
```

```wgsl
let x1 = x0 - f32(i1) + G2;
let y1 = y0 - f32(j1) + G2;
let x2 = x0 - 1.0 + 2.0 * G2;
let y2 = y0 - 1.0 + 2.0 * G2;
```

```glsl
float x1 = x0 - float(i1) + G2;
float y1 = y0 - float(j1) + G2;
float x2 = x0 - 1.0 + 2.0 * G2;
float y2 = y0 - 1.0 + 2.0 * G2;
```
:::

Each corner gets a gradient selected from a fixed permutation table. This makes the result deterministic while still looking random:

:::code-tabs simplex-2d-gradient-indices
```rust
let ii = (i & 255) as usize;
let jj = (j & 255) as usize;

let gi0 = perm(ii + perm(jj) as usize) % 8;
let gi1 = perm(ii + i1 as usize + perm(jj + j1 as usize) as usize) % 8;
let gi2 = perm(ii + 1 + perm(jj + 1) as usize) % 8;
```

```wgsl
let ii = u32(i & 255);
let jj = u32(j & 255);

let gi0 = perm(ii + perm(jj)) % 8u;
let gi1 = perm(ii + u32(i1) + perm(jj + u32(j1))) % 8u;
let gi2 = perm(ii + 1u + perm(jj + 1u)) % 8u;
```

```glsl
int ii = i & 255;
int jj = j & 255;

int gi0 = perm(ii + perm(jj)) % 8;
int gi1 = perm(ii + i1 + perm(jj + j1)) % 8;
int gi2 = perm(ii + 1 + perm(jj + 1)) % 8;
```
:::

The contribution from a corner fades to zero as the point moves away from it:

:::code-tabs simplex-2d-corner-contribution
```rust
fn corner_contribution(gradient: [f32; 2], x: f32, y: f32) -> f32 {
    let t = 0.5 - x * x - y * y;

    if t <= 0.0 {
        return 0.0;
    }

    let t2 = t * t;
    let t4 = t2 * t2;
    t4 * (gradient[0] * x + gradient[1] * y)
}
```

```wgsl
fn corner_contribution(gradient: vec2f, x: f32, y: f32) -> f32 {
    let t = 0.5 - x * x - y * y;

    if (t <= 0.0) {
        return 0.0;
    }

    let t2 = t * t;
    let t4 = t2 * t2;
    return t4 * dot(gradient, vec2f(x, y));
}
```

```glsl
float cornerContribution(vec2 gradient, float x, float y) {
    float t = 0.5 - x * x - y * y;

    if (t <= 0.0) {
        return 0.0;
    }

    float t2 = t * t;
    float t4 = t2 * t2;
    return t4 * dot(gradient, vec2(x, y));
}
```
:::

The dot product points the gradient in a direction. The `t4` multiplier makes the contribution smooth and local.

The final value is the sum of all corner contributions, scaled into a useful range:

:::code-tabs simplex-2d-final-scale
```rust
70.0 * (n0 + n1 + n2)
```

```wgsl
70.0 * (n0 + n1 + n2)
```

```glsl
70.0 * (n0 + n1 + n2)
```
:::

## Fractal Brownian Motion

A single noise sample is smooth, but often too simple. Fractal Brownian motion, or FBM, layers several noise samples called octaves:

:::code-tabs simplex-2d-fbm
```rust
fn fbm(mut coord: Vec2<f32>, octaves: u32, mut amplitude: f32, gain: f32, lacunarity: f32) -> f32 {
    let mut value = 0.0;

    for _ in 0..octaves {
        value += amplitude * coord.noise_simplex();
        coord = coord * lacunarity;
        amplitude *= gain;
    }

    value
}
```

```wgsl
fn fbm(coord_start: vec2f, octaves: i32, amplitude_start: f32, gain: f32, lacunarity: f32) -> f32 {
    var coord = coord_start;
    var amplitude = amplitude_start;
    var value = 0.0;

    for (var octave = 0; octave < octaves; octave = octave + 1) {
        value += amplitude * simplex_noise(coord);
        coord *= lacunarity;
        amplitude *= gain;
    }

    return value;
}
```

```glsl
float fbm(vec2 coord, int octaves, float amplitude, float gain, float lacunarity) {
    float value = 0.0;

    for (int octave = 0; octave < octaves; octave++) {
        value += amplitude * simplexNoise(coord);
        coord *= lacunarity;
        amplitude *= gain;
    }

    return value;
}
```
:::

`lacunarity` controls how quickly frequency increases. `gain` controls how quickly amplitude decreases.

Typical values are:

:::code-tabs simplex-2d-fbm-call
```rust
let value = fbm(coord, 4, 0.5, 0.5, 2.0);
```

```wgsl
let value = fbm(coord, 4, 0.5, 0.5, 2.0);
```

```glsl
float value = fbm(coord, 4, 0.5, 0.5, 2.0);
```
:::

Instead of only multiplying by `lacunarity`, each octave can also rotate the coordinate. Rotation reduces visible repetition and axis-aligned artifacts:

:::code-tabs simplex-2d-rotation
```rust
coord = Vec2::new(
    1.6 * coord.x + 1.2 * coord.y,
    -1.2 * coord.x + 1.6 * coord.y,
);
```

```wgsl
coord = vec2f(
    1.6 * coord.x + 1.2 * coord.y,
    -1.2 * coord.x + 1.6 * coord.y,
);
```

```glsl
coord = vec2(
    1.6 * coord.x + 1.2 * coord.y,
    -1.2 * coord.x + 1.6 * coord.y
);
```
:::

This produces richer procedural texture while keeping the underlying noise deterministic.
