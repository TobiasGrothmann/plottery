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
choices yourself and say which ones you made. Ask only if the idea is too vague to draw anything.
What the user will want to tune while looking at it becomes the first params.

## 2. Find starting points

Reusing a working piece is faster than writing it again. Look here, and tell the user what you build on:

- **The user's projects:** the folders under `roots` in the editor's `app_state.json`. Its location
  is in `editor/src/model/app_state.rs` (on macOS `~/Library/Application Support/de.tobiasgrothmann.plottery/`).
  A project is a folder with a `*.plottery` file. Per project, `resources/latest_editor_thumb.png`
  shows what it draws and `<name>/src/generate.rs` how. Grep their sources for a technique or lib type
  to see it used in practice.
- **Plottery's examples:** `examples/` in the Plottery checkout. Each `generate.rs` opens with a
  comment saying what it shows, and `preview.png` is its render.
- **The template:** `project/cargo_project_template/src/generate.rs`, what every new project starts with.
- **The library:** the "What Plottery already has" table in the new project's `AGENTS.md`.

The Plottery checkout is at `$PLOTTERY_HOME`, or at the `path` dependency in an existing project's
`Cargo.toml`.

Reuse by copying functions or modules into the new project, not by copying a project folder: a folder
carries its name, git repository and editor state.

## 3. Create the project

Run `plottery new --help` for the syntax. The choices:

- **Name:** snake_case and descriptive, it is also the crate name.
- **Parent folder:** a root from `app_state.json`, or the category subfolder of one that fits, so the
  project shows up in the editor. If nothing fits, ask.
- **Library source:** whatever the user's other projects use. Look at their `Cargo.toml`.
- If the project already exists, pick another name instead of deleting anything.

**Then read the new project's `AGENTS.md` completely** and follow it from here on: layout, the CLI
loop, params, conventions and when work counts as done.

## 4. Write the first sketch

Replace the template in `generate.rs`:

- A one-line comment above `Params` saying what the project draws, like the examples have.
- Only `seed`, `paper` and the two to five params that matter most for the idea. More come later.
- The simplest version that shows the composition. Shortcuts are fine (straight lines before splines,
  a fixed count before a density field), but split it into small functions per element so each part
  can be refined on its own.

Build and render with the CLI as `AGENTS.md` describes, look at every png, and stop after a few rounds,
once it reads as the idea.

## 5. Hand it over

- `plottery run` once: the project is new, so it overwrites nothing of the user's, and the editor tile
  gets a thumbnail. After that, follow `AGENTS.md` on when not to use it.
- Offer `plottery open`.
- Tell the user briefly: what it draws, what it builds on, the params worth playing with, what is
  simplified, and the obvious next steps.
- Don't commit unless the user asks.
