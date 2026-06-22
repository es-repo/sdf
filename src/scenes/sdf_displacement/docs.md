# SDF Displacement

SDF displacement changes the distance returned by a signed distance function using another scalar field, usually noise. The point being sampled stays the same. Only the distance value is pushed inward or outward.

:::code-tabs sdf-displacement-basic
```rust
let base = sphere.dist(&point);
let displacement = (point * scale).noise_simplex() * strength;
let dist = base + displacement;
```

```wgsl
let base = sdSphere(p, radius);
let displacement = noise(p * scale) * strength;
let dist = base + displacement;
```

```glsl
float base = sdSphere(p, radius);
float displacement = noise(p * scale) * strength;
float dist = base + displacement;
```
:::

With noise, the displacement changes across space, so the originally smooth shape gets procedural bumps and dents.

FBM makes the displacement more detailed by adding several noise layers:

:::code-tabs sdf-displacement-fbm
```rust
let displacement = (point * scale)
    .fbm(octaves, amplitude, gain, lacunarity, |p| p.noise_simplex())
    * strength;
let dist = sphere.dist(&point) + displacement;
```

```wgsl
let displacement = fbm(p * scale, octaves, amplitude, gain, lacunarity) * strength;
let dist = sdSphere(p, radius) + displacement;
```

```glsl
float displacement = fbm(p * scale) * strength;
float dist = sdSphere(p, radius) + displacement;
```
:::

`scale` controls how large the noise features are. `strength` controls how far the surface moves. `octaves`, `gain`, and `lacunarity` control how much layered detail is added.

This is different from domain warping. Domain warping changes the sampled coordinate before evaluating the shape:

:::code-tabs sdf-displacement-domain-warping
```rust
let dist = sphere.dist(&(point + warp));
```

```wgsl
let dist = sdSphere(p + warp, radius);
```

```glsl
float dist = sdSphere(p + warp, radius);
```
:::

SDF displacement evaluates the original shape first and then modifies the resulting distance:

:::code-tabs sdf-displacement-distance
```rust
let dist = sphere.dist(&point) + displacement;
```

```wgsl
let dist = sdSphere(p, radius) + displacement;
```

```glsl
float dist = sdSphere(p, radius) + displacement;
```
:::

Large displacement can break the strict signed-distance property. When that happens, ray marching may need smaller steps, more iterations, or a smaller displacement strength.
