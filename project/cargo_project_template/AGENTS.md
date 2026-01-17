# AGENTS.md - Plottery Project

This is a **Plottery Project** - a generative art generator for pen-plotters.

## How It Works

1. Define parameters in `Params` struct with `#[derive(PlotteryParams)]`. These are exposed and editable in the Plottery Editor.
2. The function `generate(params: Params) -> Layer` is repeatedly called to generate the artwork whenever the user triggers it or the parameters are changed.
3. Run via Plottery Editor, CLI, or `cargo run svg`

## Key Types

All from `plottery_lib::*`:

Some examples:
- `V2` - 2D vector. Use `V2::new(x, y)`, `V2::polar(angle, radius)`, `V2::a4()` for paper sizes
- `Angle` - Use `Angle::from_degrees()`, `Angle::from_rotations()`, `Angle::rand()`
- `Layer` - Recursive container for shapes. Use `push()`, `push_many()`, `push_layer()`
- `Shape` - Enum of `Circle`, `Rect`, `Path`
- `Frame` - Great for a layout with margins. Use `inner_rect()` for drawable area, `outer_rect()` is used to cut the paper at.

## Parameters

```rust
#[derive(PlotteryParams)]
pub struct Params {
    #[value(10.0)]          // Default value (required)
    #[range(0.0, 100.0)]    // Optional min/max for editor slider
    pub my_param: f32,      // there are many more types that can be exposed to the editor (for example, `i32`, `Curve2D`, ...)
}
```

## Common Patterns

**Shapes:** `Circle::new(center, radius)`, `Rect::new(bl, tr)`, `Path::new_from(vec![points])`
**Transforms:** `shape.translate(v2).rotate(angle).scale(factor)`

Use idiomatic rust - iterators, builder types, and functional programming.

## Resources

- [plottery_lib docs](https://docs.rs/plottery_lib)
- [Plottery GitHub](https://github.com/TobiasGrothmann/plottery)
