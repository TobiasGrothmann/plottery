# Plottery CLI

Part of [**Plottery**](../README.md), a generative art engine for pen-plotters.

Creates, builds and renders Plottery projects. Install with `cargo install plottery_cli`, or on macOS together with the editor via the [install script](../README.md#install).

Create a new project in the folder `sketches/my_sketch`:
```sh
plottery new sketches my_sketch
```

Inside a project folder (or any folder below it):
```sh
plottery build                                    # compile, show warnings
plottery params                                   # the params with the editor's saved values
plottery params --set size=12                     # preview an override
plottery render /tmp/a.png /tmp/a.svg --set size=12  # render, print length, travel and bounds
plottery run                                      # like the editor's Run button
plottery open                                     # open the project in the editor
```

Project commands also take the project folder as an argument, e.g. `plottery run ~/art/labyrinth`. `plottery --help` lists all commands and `plottery <command> --help` their options.
