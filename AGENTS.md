# AGENTS.md

## Project Overview

**unholy-force** — Rust game project built with `macroquad` (rendering) + `egui` (UI). 
Uses ECS via `hecs`, wave function collapse for procedural floor generation (`simple-tiled-wfc`), 
and a node-graph editor (`egui-snarl`) for floor flow configuration.

## Build & Run

```sh
cargo build
cargo run
```

No separate test suite is currently configured.

## Project Structure

```
src/
  main.rs                  — Entry point
  assets.rs                — Asset loading
  graphics.rs              — Rendering utilities
  screen_utils.rs          — Screen/layout helpers
  app/
    mod.rs                 — App state machine
    app_stage.rs           — Stage definitions
    main_menu_stage.rs     — Main menu
    editor_stage/          — Configuration editors (egui-based)
      parameter_config_editor.rs
      tag_config_editor.rs
      floor_config_editor.rs
      floor_part_editor.rs
      floor_part_adjacency_config_editor.rs
      item_config_editor.rs
      unit_config_editor.rs
      image_widgets.rs
    game_stage/
      mod.rs               — Gameplay stage
      floor_generator.rs   — Procedural floor generation (WFC)
      grid_math.rs         — Grid math utilities
  game_config/
    mod.rs                 — Config module root
    parameters.rs          — Game parameters
    floors.rs              — Floor definitions
    floor_parts.rs         — Floor part definitions
    floor_part_adjacency.rs — Adjacency rules for WFC
    floor_flow_graph.rs    — Floor flow node graph
    units.rs               — Unit definitions
    items.rs               — Item definitions
    effects.rs             — Effect definitions
  effect_mechanics/
    mod.rs                 — Effect/mechanics system
```

## Conventions

- Language: **Rust** (edition 2024)
- UI framework: **egui** via `egui-macroquad`
- Config format: **JSON5** (`json5` crate + `serde`)
- Follow existing code style in each module; no explicit linter is configured
- Keep editor stages under `src/app/editor_stage/`, gameplay under `src/app/game_stage/`
- Game data types live in `src/game_config/`
