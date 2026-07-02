# Ray Marching

Ray marching renders a scene by advancing a ray through space and sampling the scene along its path. When the scene is represented by signed distance fields, each sample tells the renderer how far it can move without crossing a surface.

Using the SDF distance as the step length is more specifically called **sphere tracing**. It is the common ray-marching method used for SDF rendering.

## Camera Ray

Rendering starts by constructing one ray for each pixel. The ray has an origin at the camera and a normalized direction through the corresponding point on the virtual viewport.

:::code-tabs ray-marching-camera-ray
```rust
let direction = (
    camera_forward
        + camera_right * screen_coord.x * viewport_half_width
        + camera_up * screen_coord.y * viewport_half_height
).normalize();

let ray = Ray::new(camera_position, direction);
```

```wgsl
let direction = normalize(
    cameraForward
        + cameraRight * screenCoord.x * viewportHalfWidth
        + cameraUp * screenCoord.y * viewportHalfHeight
);

let ray = Ray(cameraPosition, direction);
```

```glsl
vec3 direction = normalize(
    cameraForward
        + cameraRight * screenCoord.x * viewportHalfWidth
        + cameraUp * screenCoord.y * viewportHalfHeight
);

Ray ray = Ray(cameraPosition, direction);
```
:::

Changing the field of view changes the viewport dimensions. Changing the aspect ratio changes its width relative to its height.

## Scene Distance Field

The renderer needs a function that returns the signed distance from any point to the nearest scene surface. Primitive SDFs are combined into a single scene field. For a normal union, the smallest distance wins:

:::code-tabs ray-marching-scene-sdf
```rust
fn scene_sdf(point: Vec3) -> f32 {
    let sphere_dist = sphere_sdf(point);
    let box_dist = box_sdf(point);
    sphere_dist.min(box_dist)
}
```

```wgsl
fn sceneSdf(point: vec3f) -> f32 {
    let sphereDist = sphereSdf(point);
    let boxDist = boxSdf(point);
    return min(sphereDist, boxDist);
}
```

```glsl
float sceneSdf(vec3 point) {
    float sphereDist = sphereSdf(point);
    float boxDist = boxSdf(point);
    return min(sphereDist, boxDist);
}
```
:::

The scene sample can also carry a material identifier or color. The material associated with the smallest distance is then used when the ray hits the surface.

## Marching the Ray

The ray starts at a near distance from the camera. At each iteration, the renderer samples the scene SDF and advances by the returned distance:

:::code-tabs ray-marching-loop
```rust
let mut traveled = near_clip;

for _ in 0..max_steps {
    let point = ray.origin + ray.direction * traveled;
    let distance = scene_sdf(point);

    if distance < hit_epsilon {
        return Hit { point, traveled };
    }

    traveled += distance.max(min_step);

    if traveled > max_distance {
        break;
    }
}

return Miss;
```

```wgsl
var traveled = nearClip;

for (var i = 0; i < maxSteps; i++) {
    let point = ray.origin + ray.direction * traveled;
    let distance = sceneSdf(point);

    if (distance < hitEpsilon) {
        return RayHit(point, traveled, true);
    }

    traveled += max(distance, minStep);

    if (traveled > maxDistance) {
        break;
    }
}

return RayHit(vec3f(0.0), traveled, false);
```

```glsl
float traveled = nearClip;

for (int i = 0; i < MAX_STEPS; i++) {
    vec3 point = ray.origin + ray.direction * traveled;
    float distance = sceneSdf(point);

    if (distance < hitEpsilon) {
        return RayHit(point, traveled, true);
    }

    traveled += max(distance, minStep);

    if (traveled > maxDistance) {
        break;
    }
}

return RayHit(vec3(0.0), traveled, false);
```
:::

## Why Distance-Sized Steps Work

At a sampled point, an exact SDF gives the distance to the closest surface. A sphere with that radius can be placed around the point without intersecting any geometry. The ray can therefore advance to the edge of that sphere without skipping a surface.

Far from geometry, the distance is large and the ray advances quickly. Near geometry, the distance becomes small and the steps automatically become more precise.

This guarantee depends on the field being an exact distance or a conservative lower bound. Displacement, aggressive domain deformation, and some smooth operations can produce values that are no longer safe step lengths. If the field overestimates the distance, the ray can step through thin geometry.

## Hit and Miss Conditions

Ray marching stops for one of three reasons:

- The distance becomes smaller than `hit_epsilon`, so the point is treated as a surface hit.
- The traveled distance exceeds `max_distance`, so the ray cannot see relevant geometry.
- The loop reaches `max_steps`, preventing an infinite or excessively expensive march.

`hit_epsilon` is a numerical tolerance rather than an exact intersection. A smaller value improves surface precision but usually requires more steps. A large `min_step` improves progress through problematic fields but can overshoot surfaces, so it should be used carefully.

## Surface Normal

An SDF does not directly store a surface normal. The normal is estimated from the gradient of the distance field near the hit point. Central differences sample both sides of each axis:

:::code-tabs ray-marching-normal
```rust
let normal = Vec3::new(
    scene_sdf(point + Vec3::new(epsilon, 0.0, 0.0))
        - scene_sdf(point - Vec3::new(epsilon, 0.0, 0.0)),
    scene_sdf(point + Vec3::new(0.0, epsilon, 0.0))
        - scene_sdf(point - Vec3::new(0.0, epsilon, 0.0)),
    scene_sdf(point + Vec3::new(0.0, 0.0, epsilon))
        - scene_sdf(point - Vec3::new(0.0, 0.0, epsilon)),
).normalize();
```

```wgsl
let normal = normalize(vec3f(
    sceneSdf(point + vec3f(epsilon, 0.0, 0.0))
        - sceneSdf(point - vec3f(epsilon, 0.0, 0.0)),
    sceneSdf(point + vec3f(0.0, epsilon, 0.0))
        - sceneSdf(point - vec3f(0.0, epsilon, 0.0)),
    sceneSdf(point + vec3f(0.0, 0.0, epsilon))
        - sceneSdf(point - vec3f(0.0, 0.0, epsilon))
));
```

```glsl
vec3 normal = normalize(vec3(
    sceneSdf(point + vec3(epsilon, 0.0, 0.0))
        - sceneSdf(point - vec3(epsilon, 0.0, 0.0)),
    sceneSdf(point + vec3(0.0, epsilon, 0.0))
        - sceneSdf(point - vec3(0.0, epsilon, 0.0)),
    sceneSdf(point + vec3(0.0, 0.0, epsilon))
        - sceneSdf(point - vec3(0.0, 0.0, epsilon))
));
```
:::

The normal can then be used by a lighting model such as Lambert or Phong shading.

## Shadow Rays

After a surface is hit, the same marching technique can be used along a secondary ray toward a light. If geometry is reached before the light, the light is occluded.

:::code-tabs ray-marching-shadow
```rust
let shadow_origin = surface_point + surface_normal * bias;
let to_light = light_position - shadow_origin;
let shadow_ray = Ray::new(shadow_origin, to_light.normalize());
let visibility = march_shadow(shadow_ray, to_light.len());
```

```wgsl
let shadowOrigin = surfacePoint + surfaceNormal * bias;
let toLight = lightPosition - shadowOrigin;
let shadowRay = Ray(shadowOrigin, normalize(toLight));
let visibility = marchShadow(shadowRay, length(toLight));
```

```glsl
vec3 shadowOrigin = surfacePoint + surfaceNormal * bias;
vec3 toLight = lightPosition - shadowOrigin;
Ray shadowRay = Ray(shadowOrigin, normalize(toLight));
float visibility = marchShadow(shadowRay, length(toLight));
```
:::

The normal offset prevents the secondary ray from immediately hitting the surface that created it. Binary visibility produces hard shadows. Tracking how closely the ray passes nearby geometry can approximate a soft penumbra.

## Accuracy and Performance

Ray-marching quality and cost are controlled by several related choices:

- `max_steps` limits the number of SDF evaluations per ray.
- `max_distance` limits how far the renderer searches.
- `hit_epsilon` controls when a sample is close enough to a surface.
- `min_step` prevents extremely small steps but can cause overshoot.
- The complexity of the scene SDF determines the cost of every step.

There is no single ideal configuration. Small detailed geometry needs tighter tolerances, while large simple scenes can use fewer steps and larger limits. Debugging missed surfaces usually starts by checking whether the field remains a conservative distance bound and whether the minimum step or hit tolerance is too large.
