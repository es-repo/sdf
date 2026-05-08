# Signed Distance Fields

A signed distance field, or SDF, stores geometry as a distance function instead of as pixels or triangles. For any point in space, the function returns how far that point is from the closest surface.

- Negative distance means the point is inside the shape.
- Positive distance means the point is outside the shape.
- Zero means the point is exactly on the boundary.

That single value is enough to draw a shape:

:::code-tabs sdf-fill
```rust
let dist = shape.dist(&coord);

if dist < 0.0 {
    shape.color
} else {
    Color::BLACK
}
```

```wgsl
let dist = shape_sdf(coord);

if dist < 0.0 {
    return shape_color;
}

return background_color;
```

```glsl
float dist = shapeSdf(coord);

if (dist < 0.0) {
    fragColor = shapeColor;
} else {
    fragColor = backgroundColor;
}
```
:::

## Primitives

Each primitive implements the same idea with different geometry. A circle is the distance from the point to the center, minus the radius:

:::code-tabs circle-sdf
```rust
center.dist(&point) - radius
```

```wgsl
length(point - center) - radius
```

```glsl
length(point - center) - radius
```
:::

A rectangle first moves the point into the rectangle's local space, then measures distance to the box:

:::code-tabs rectangle-sdf
```rust
let p = (point - center).rotate(-rotation);
let d = p.abs() - half_size;

let outside = d.max(Vec2::new(0.0, 0.0)).len();
let inside = d.x.max(d.y).min(0.0);

outside + inside
```
:::

The triangle uses the same signed-distance contract, but its distance is built from the closest point on each edge plus side tests to determine whether the point is inside.

## Combining Shapes

SDFs are useful because shapes combine with simple operations. A hard union takes the smaller distance:

:::code-tabs sdf-union
```rust
let dist = circle_dist.min(rectangle_dist);
```

```wgsl
let dist = min(circle_dist, rectangle_dist);
```

```glsl
float dist = min(circleDist, rectangleDist);
```
:::

The smaller value wins because it represents the closest surface. If either shape contains the point, at least one distance is negative, so the union is also negative.

## Rounding

Rounding is also a distance operation. Subtracting a radius from the SDF expands the shape outward and rounds sharp SDF corners:

:::code-tabs sdf-rounding
```rust
fn dist_round(&self, point: &Vec2<f32>, radius: f32) -> f32 {
    self.dist(point) - radius
}
```
:::

The scene animates that radius so the primitives softly grow and shrink while keeping the same SDF-based rendering logic.
