---
name: new-plottery-project
description: Create a new Plottery project (a Rust generative-art generator for pen plotters) from an artwork idea and get a first rendered sketch fast, then iterate. Use when the user wants a new plottery project, wants to sketch, try or prototype a plotter artwork idea, or start a new drawing generator, even if they don't say "project".
---

# New Plottery project: idea to first sketch

The goal is a first sketch of the user's idea that renders, quickly. Rough is fine. The user iterates
from there in the editor, with you changing the code. Don't polish before they have seen it.

This skill only says what to do and where to look. The details live in the sources it points to;
read them there.

## 1. Pin down the idea

Restate the idea in one or two sentences: what is drawn, the composition, what varies. Make the open
choices yourself and say which ones you made. Ask only if the idea is too vague to draw anything, and
then together with the question in step 2, so the user answers once.
What the user will want to tune while looking at it becomes the first params.

## 2. Decide where it goes

**Already in a new project:** if the working directory is inside a Plottery project (a `*.plottery`
file in it or a parent folder) that is still the untouched template, the user made it for this idea.
Use it as it is, with its name, and skip `plottery new` in step 4. Say so in one line. Untouched means
its `<name>/src/generate.rs` still draws the template's circle spiral (compare with
`project/cargo_project_template/src/generate.rs` in the Plottery checkout) and its git repository has
no commits yet. A project with work in it is not to be built over.

**Otherwise** suggest a full path `<parent>/<name>` and ask the user to confirm it or name another:

- **Parent folder:** a root from `roots` in the editor's `app_state.json`, or the category subfolder of
  one that fits the idea, so the project shows up in the editor. The file's location is in
  `editor/src/model/app_state.rs` (on macOS `~/Library/Application Support/de.tobiasgrothmann.plottery/`).
- **Name:** snake_case and descriptive, it is also the crate name.
- If the path already exists, suggest another name instead of deleting anything.

## 3. Find starting points

Reusing a working piece is faster than writing it again. Look here, and tell the user what you build on:

- **The user's projects:** the folders under `roots` in `app_state.json` (see step 2). A project is a
  folder with a `*.plottery` file. Per project, `<name>/src/generate.rs` shows how it draws and its
  thumbnail what: `previews/<name>-*.png` in the editor's cache folder (on macOS
  `~/Library/Caches/de.tobiasgrothmann.plottery/`). Grep their sources for a technique or lib type to
  see it used in practice.
- **Plottery's examples:** `examples/` in the Plottery checkout. Each `generate.rs` opens with a
  comment saying what it shows, and `preview.png` is its render.
- **The template:** `project/cargo_project_template/src/generate.rs`, what every new project starts with.
- **The library:** the "What Plottery already has" table in the new project's `AGENTS.md`.

The Plottery checkout is at `$PLOTTERY_HOME`, or at the `path` dependency in an existing project's
`Cargo.toml`.

Reuse by copying functions or modules into the new project, not by copying a project folder: a folder
carries its name, git repository and editor state.

## 4. Create the project

Skip this if you use the project you are in. Otherwise run `plottery new` at the path from step 2;
`plottery new --help` has the syntax. For the library source, use whatever the user's other projects
use. Look at their `Cargo.toml`.

**Then read the project's `AGENTS.md` completely** and follow it from here on: layout, the CLI
loop, params, conventions and when work counts as done.

## 5. Write the first sketch

Replace the template in `generate.rs`:

- A one-line comment above `Params` saying what the project draws, like the examples have.
- Only `seed`, `paper` and the two to five params that matter most for the idea. More come later.
- The simplest version that shows the composition. Shortcuts are fine (straight lines before splines,
  a fixed count before a density field), but split it into small functions per element so each part
  can be refined on its own.
- If the drawing comes about over iterations (growth, a simulation, relaxation), call `snapshot` in
  that loop from the start, as `AGENTS.md` says under Conventions. It costs nothing in normal runs and
  lets the user make a video with `plottery animate`. Leave it out when nothing is built up in steps.

Build and render with the CLI as `AGENTS.md` describes, look at every png, and stop after a few rounds,
once it reads as the idea.

## 6. Hand it over

- `plottery run` once: the project is new, so it overwrites nothing of the user's, and the editor tile
  gets a thumbnail. After that, follow `AGENTS.md` on when not to use it.
- Offer `plottery open`.
- Tell the user briefly: what it draws, what it builds on, the params worth playing with, what is
  simplified, and the obvious next steps.
- Don't commit unless the user asks.
