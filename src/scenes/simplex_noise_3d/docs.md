# 3D Simplex Noise

3D simplex noise extends the [2D algorithm](?scene=simplex-noise) into a volume. Instead of dividing space into triangles, it divides space into tetrahedra.

The idea is the same:

- Find the simplex cell that contains the point.
- Select deterministic gradients at the cell corners.
- Weight each corner contribution by distance.
- Sum the contributions into one smooth value.

The skew constants are simpler in 3D:

:::code-tabs simplex-3d-constants
```rust
const F3: f32 = 1.0 / 3.0;
const G3: f32 = 1.0 / 6.0;
```

```wgsl
const F3: f32 = 1.0 / 3.0;
const G3: f32 = 1.0 / 6.0;
```

```glsl
const float F3 = 1.0 / 3.0;
const float G3 = 1.0 / 6.0;
```
:::

Skewing works like the 2D version, but it includes all three axes:

:::code-tabs simplex-3d-skew
```rust
let s = (p.x + p.y + p.z) * F3;
let i = (p.x + s).floor() as i32;
let j = (p.y + s).floor() as i32;
let k = (p.z + s).floor() as i32;
```

```wgsl
let s = (p.x + p.y + p.z) * F3;
let i = i32(floor(p.x + s));
let j = i32(floor(p.y + s));
let k = i32(floor(p.z + s));
```

```glsl
float s = (p.x + p.y + p.z) * F3;
int i = int(floor(p.x + s));
int j = int(floor(p.y + s));
int k = int(floor(p.z + s));
```
:::

After unskewing, the point is relative to the first tetrahedron corner:

:::code-tabs simplex-3d-unskew
```rust
let t = (i + j + k) as f32 * G3;
let x0 = p.x - (i as f32 - t);
let y0 = p.y - (j as f32 - t);
let z0 = p.z - (k as f32 - t);
```

```wgsl
let t = f32(i + j + k) * G3;
let x0 = p.x - (f32(i) - t);
let y0 = p.y - (f32(j) - t);
let z0 = p.z - (f32(k) - t);
```

```glsl
float t = float(i + j + k) * G3;
float x0 = p.x - (float(i) - t);
float y0 = p.y - (float(j) - t);
float z0 = p.z - (float(k) - t);
```
:::

The main extra step in 3D is deciding which tetrahedron the point is inside. This is done by ranking `x0`, `y0`, and `z0`. The largest axis gets the first offset, the two largest axes get the second offset, and the final corner is always `(1, 1, 1)`.

For example, if `x0 >= y0 >= z0`, the four corners are:

:::code-tabs simplex-3d-corners
```rust
let first = (1, 0, 0);
let second = (1, 1, 0);
let last = (1, 1, 1);
```

```wgsl
let first = vec3i(1, 0, 0);
let second = vec3i(1, 1, 0);
let last = vec3i(1, 1, 1);
```

```glsl
ivec3 first = ivec3(1, 0, 0);
ivec3 second = ivec3(1, 1, 0);
ivec3 last = ivec3(1, 1, 1);
```
:::

Other orderings choose different offsets, but the idea stays the same: sort the coordinate components and walk through the tetrahedron corners.

Each corner contribution is still a local gradient dot product with a smooth falloff:

:::code-tabs simplex-3d-corner-contribution
```rust
fn corner_contribution(gradient: [f32; 3], x: f32, y: f32, z: f32) -> f32 {
    let t = 0.6 - x * x - y * y - z * z;

    if t <= 0.0 {
        return 0.0;
    }

    let t2 = t * t;
    let t4 = t2 * t2;
    t4 * (gradient[0] * x + gradient[1] * y + gradient[2] * z)
}
```

```wgsl
fn corner_contribution(gradient: vec3f, x: f32, y: f32, z: f32) -> f32 {
    let t = 0.6 - x * x - y * y - z * z;

    if (t <= 0.0) {
        return 0.0;
    }

    let t2 = t * t;
    let t4 = t2 * t2;
    return t4 * dot(gradient, vec3f(x, y, z));
}
```

```glsl
float cornerContribution(vec3 gradient, float x, float y, float z) {
    float t = 0.6 - x * x - y * y - z * z;

    if (t <= 0.0) {
        return 0.0;
    }

    float t2 = t * t;
    float t4 = t2 * t2;
    return t4 * dot(gradient, vec3(x, y, z));
}
```
:::

The final 3D value has four corner contributions instead of three:

:::code-tabs simplex-3d-final-scale
```rust
32.0 * (n0 + n1 + n2 + n3)
```

```wgsl
32.0 * (n0 + n1 + n2 + n3)
```

```glsl
32.0 * (n0 + n1 + n2 + n3)
```
:::

The useful part of 3D noise is that a 2D image can be treated as a slice through a 3D volume. For animation, use the screen coordinate as `x` and `y`, and time as `z`:

:::code-tabs simplex-3d-time-slice
```rust
let p = Vec3::from_2d(coord, time);
let value = p.noise_simplex();
```

```wgsl
let p = vec3f(coord.x, coord.y, time);
let value = simplex_noise(p);
```

```glsl
vec3 p = vec3(coord, time);
float value = simplexNoise(p);
```
:::

As `time` changes, the slice moves through the volume. The result animates smoothly because neighboring time values sample neighboring points in the same continuous field.

This is different from generating a new 2D noise pattern every frame. A new random pattern would flicker. Moving through 3D noise produces temporal continuity.

3D noise can also be layered with fractal Brownian motion, just like the 2D version. The same octave idea applies: sample at increasing frequencies and decreasing amplitudes. When using 3D coordinates, the octave transform can rotate or mix `x`, `y`, and `z` to reduce visible repetition across all axes.
