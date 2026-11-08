# Plottery CLI

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
```

| Command | |
|---|---|
| `new <PATH> <NAME> [-l cratesio\|home\|PATH]` | create a project |
| `build` | build the project |
| `run` | build and run with the saved params, update the editor files |
| `params [--defaults] [--set NAME=VALUE]...` | print the params, one `--set` target per line |
| `render <OUT>... [--defaults] [--set NAME=VALUE]... [--size PX] [--update-editor]` | render to `.png` and `.svg` files |
| `set-home <PATH>` | set `PLOTTERY_HOME` for `new -l home` |
| `version` | print the version |

All project commands take `-p <PATH>`, a project folder or `.plottery` file. Without it the project
is found from the current folder upwards.

`--set` and `--defaults` only apply to that command. The editor's saved values change only with
`render --update-editor`, which saves what was rendered.
