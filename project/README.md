# Plottery Project

Part of [**Plottery**](https://github.com/TobiasGrothmann/plottery), a generative art engine for pen-plotters.

Creates, builds and runs **Plottery** projects, the host side used by the editor and the CLI. A project is a regular cargo package:

<img alt="folder structure" src="https://github.com/user-attachments/assets/d0066b16-5b75-4563-b625-88cc7d206a77" />

Create one with the [editor](https://github.com/TobiasGrothmann/plottery/tree/main/editor) or `plottery new` from the [CLI](https://crates.io/crates/plottery_cli).

What a project's own code uses, its parameters and `run_project`, is [`plottery_runtime`](https://docs.rs/plottery_runtime). Complete example projects with rendered previews are in [`examples/`](https://github.com/TobiasGrothmann/plottery/tree/main/examples).

## Build and run

```rust,no_run
use plottery_project::{LibSource, Project};

fn main() -> anyhow::Result<()> {
    let project = Project::new("sketches".into(), "my_sketch");
    project.generate_to_disk(LibSource::CratesIO, true)?;

    project.build(true)?;
    let params = project.run_get_params(true)?;
    let layer = project.run(true, &params)?;
    project.write_svg("my_sketch.svg".into(), &layer)
}
```

