# Phong Lighting

Phong lighting is a simple model for making a surface react to light. It adds three pieces together: ambient light, diffuse light, and specular reflection.

After a ray hits a surface, the renderer estimates a normal from nearby SDF samples and uses that normal for lighting.

## Surface Hit

Each pixel starts with a camera ray. Ray marching moves along that ray until the SDF distance is close enough to zero:

:::code-tabs ray-hit
```rust
let hit = ray_march(ray, settings, |point| sample_scene(point));
```

```wgsl
let hit = rayMarch(ray, settings);
```

```glsl
RayHit hit = rayMarch(ray, settings);
```
:::

The hit point gives the position on the surface. The material color comes from the SDF sample.

## Surface Normal

Lighting needs a surface normal: a direction pointing away from the surface. Mesh renderers usually get normals from vertices or triangles. An SDF scene estimates the normal by sampling the distance field around the hit point:

:::code-tabs normal
```rust
let normal = estimate_normal_tetrahedral(hit.point, epsilon, |point| {
    sample_scene(point).dist
});
```

```wgsl
let normal = estimateNormalTetrahedral(hit.point, epsilon);
```

```glsl
vec3 normal = estimateNormalTetrahedral(hit.point, epsilon);
```
:::

The normal points in the direction where the distance field increases fastest.

## Ambient Light

Ambient light is the constant base light in the scene. It prevents surfaces from becoming completely black when they face away from the point light:

:::code-tabs ambient
```rust
let ambient = material_color * ambient_color * ambient_intensity;
```

```wgsl
let ambient = materialColor * ambientColor * ambientIntensity;
```

```glsl
vec3 ambient = materialColor * ambientColor * ambientIntensity;
```
:::

It is not physically precise. It is a practical approximation for indirect light.

## Diffuse Light

Diffuse light depends on the angle between the light direction and the surface normal:

:::code-tabs diffuse
```rust
let diffuse_strength = normal.dot(light_dir).max(0.0);
let diffuse = material_color * light_color * light_intensity * diffuse_strength;
```

```wgsl
let diffuseStrength = max(dot(normal, lightDir), 0.0);
let diffuse = materialColor * lightColor * lightIntensity * diffuseStrength;
```

```glsl
float diffuseStrength = max(dot(normal, lightDir), 0.0);
vec3 diffuse = materialColor * lightColor * lightIntensity * diffuseStrength;
```
:::

When the surface faces the light, the dot product is close to `1.0`. When it turns away, the value approaches `0.0`.

## Specular Reflection

Specular reflection creates the shiny highlight. The light direction is reflected around the surface normal and compared with the view direction:

:::code-tabs specular
```rust
let reflected = (light_dir * -1.0).reflect(normal);
let specular_strength = reflected.dot(view_dir).max(0.0).powf(shininess);
let specular = specular_color * light_color * light_intensity * specular_intensity * specular_strength;
```

```wgsl
let reflected = reflect(-lightDir, normal);
let specularStrength = pow(max(dot(reflected, viewDir), 0.0), shininess);
let specular = specularColor * lightColor * lightIntensity * specularIntensity * specularStrength;
```

```glsl
vec3 reflected = reflect(-lightDir, normal);
float specularStrength = pow(max(dot(reflected, viewDir), 0.0), shininess);
vec3 specular = specularColor * lightColor * lightIntensity * specularIntensity * specularStrength;
```
:::

`shininess` controls the size of the highlight. Higher values make it smaller and sharper. `specular_intensity` controls how bright the highlight is.
