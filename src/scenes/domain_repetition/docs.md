# Domain Repetition

Domain repetition creates repeated structure by transforming the input coordinate before evaluating an SDF. Instead of storing every copy as a separate object, the renderer folds each sample point into a local lattice cell and evaluates the shape in that local space.

For a normal SDF, the point is evaluated directly:

:::code-tabs shape-sdf
```rust
let dist = shape_sdf(point);
```

```wgsl
let dist = shapeSdf(point);
```

```glsl
float dist = shapeSdf(point);
```
:::

Domain repetition changes the point first. The world-space point is split into two values: the local point inside the nearest lattice cell, and the cell index.

:::code-tabs lattice-cell
```rust
let cell_index = (point / spacing).round();
let local_point = point - cell_index * spacing;
let dist = shape_sdf(local_point);
```

```wgsl
let cellIndex = round(point / spacing);
let localPoint = point - cellIndex * spacing;
let dist = shapeSdf(localPoint);
```

```glsl
vec3 cellIndex = round(point / spacing);
vec3 localPoint = point - cellIndex * spacing;
float dist = shapeSdf(localPoint);
```
:::

The same SDF is evaluated in local space, but because every world-space point is folded into a cell, the result looks like an infinite lattice of repeated shapes.

## Cell Index

The local point controls the shape. The cell index identifies which repeated copy the point belongs to.

That index is useful for stable per-cell variation. For example, it can generate a different color for each copy:

:::code-tabs cell-color
```rust
let color = Color::rgb(
    0.5 + 0.5 * (cell_index.x + 1.0).sin(),
    0.5 + 0.5 * cell_index.y.sin(),
    0.5 + 0.5 * cell_index.z.sin(),
);
```

```wgsl
let color = vec3f(
    0.5 + 0.5 * sin(cellIndex.x + 1.0),
    0.5 + 0.5 * sin(cellIndex.y),
    0.5 + 0.5 * sin(cellIndex.z),
);
```

```glsl
vec3 color = vec3(
    0.5 + 0.5 * sin(cellIndex.x + 1.0),
    0.5 + 0.5 * sin(cellIndex.y),
    0.5 + 0.5 * sin(cellIndex.z)
);
```
:::

The same idea can drive size, material, or animation. Since the index is derived from the lattice position, each repeated copy gets deterministic variation without storing object instances.
