# Ray Marching

Ray marching renders 3D signed distance fields by sending a ray from the camera through each pixel. At every step, the scene SDF tells how far the current point is from the closest surface.

That distance is safe to move along the ray:

:::code-tabs ray-marching-step
```rust
let ray = camera.ray(coord);
let mut marching_dist = 0.0;

for _ in 0..max_steps {
    let point = ray.at(marching_dist);
    let sample = sample_scene(point);

    if sample.dist < hit_epsilon {
        return shade(point, sample);
    }

    marching_dist += sample.dist;
}
```
:::

If the distance becomes very small, the ray is close enough to a surface and the pixel can be shaded. If the accumulated distance grows past a maximum range, the ray missed the scene.

## Scene Sample

The scene function returns more than just distance. It also carries the color or material for the closest surface:

:::code-tabs ray-marching-sample
```rust
struct SdfSample {
    dist: f32,
    color: Color,
}
```
:::

The distance drives the ray marching step. The color is used only after the ray hits.

## Normals

Lighting needs a surface normal. For a general SDF scene, the normal can be estimated by sampling the distance field near the hit point:

:::code-tabs ray-marching-normal
```rust
let normal = estimate_normal(hit_point);
let light_dir = (light - hit_point).normalize();
let intensity = normal.dot(light_dir).max(0.0);
```
:::

This works even when the scene combines multiple shapes, because the normal comes from the final distance field.
