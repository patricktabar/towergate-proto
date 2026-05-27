# Towergate — Evolution 01: Terminal HUD Overlay

**Project:** `towergate` (v0.1.0)  
**Engine:** [Bevy 0.18.1](https://bevyengine.org/) (Rust, edition 2024)  
**Coverage:** Commits `8c561ad` → `b4ac03c`  
**Previous baseline:** `docs/architecture.md` (as of commit `8c561ad`)

---

## 1. Overview

This evolution transforms Towergate from a headless logic testbed into a visible, windowed application. It introduces the first graphical layer — a terminal-styled HUD overlay that renders real-time game state on screen — and completes the migration from `MinimalPlugins` to `DefaultPlugins`, enabling the full Bevy render pipeline (sprites, UI, text, camera).

Two commits span this release:

| Commit | Message | Purpose |
|--------|---------|---------|
| `a52e17c` | Add architecture documentation | `docs/architecture.md` capturing the project state as of `8c561ad` |
| `b4ac03c` | Add terminal HUD overlay and window config | Window setup, camera, and the first visual UI layer |

The architecture documentation itself was authored in this window, which means `docs/architecture.md` is now the stable reference for the baseline, and this document captures what came next.

---

## 2. Dependency Surface

No dependencies were added or removed. The existing `Cargo.toml` already included the feature flags required for this evolution:

| Existing Feature   | Used by this evolution                                    |
|--------------------|-----------------------------------------------------------|
| `bevy_winit`       | `WindowPlugin` → primary window creation                  |
| `bevy_render`      | `ClearColor`, `Camera2d`                                  |
| `bevy_core_pipeline` | `Camera2d` pipeline                                     |
| `bevy_sprite`      | Indirectly loaded by `DefaultPlugins`                     |
| `bevy_text`        | `Text` / `TextFont` / `TextColor` in UI                   |
| `bevy_ui`          | `Node`, `BackgroundColor`, `UiRect` layout                |
| `bevy_state`       | `StatesPlugin` (now auto-included via `DefaultPlugins`)   |
| `multi_threaded`   | ECS parallelism                                           |

The key change is that `DefaultPlugins` now implicitly enables these features at the plugin level rather than cherry-picking `MinimalPlugins` + `InputPlugin`. The `StatesPlugin` (previously imported explicitly via `use bevy::state::app::StatesPlugin`) is now pulled in automatically by `DefaultPlugins` — so the explicit import and plugin addition were removed.

---

## 3. Module-by-Module Breakdown

### 3.1 `main.rs` — From Headless to Windowed

#### Before (`8c561ad`)

```rust
App::new()
    .add_plugins((MinimalPlugins, InputPlugin))
    .add_plugins(StatePlugin)
    .add_plugins(NetworkPlugin)
    .add_plugins(AgentPlugin)
    .run();
```

#### After (`b4ac03c`)

```rust
App::new()
    .add_plugins(DefaultPlugins.set(WindowPlugin {
        primary_window: Some(Window {
            title: "TOWERGATE // Network Intrusion Terminal".to_string(),
            resolution: (1024, 768).into(),
            resizable: false,
            ..default()
        }),
        ..default()
    }))
    .insert_resource(ClearColor(Color::BLACK))
    .add_plugins(StatePlugin)
    .add_plugins(NetworkPlugin)
    .add_plugins(AgentPlugin)
    .add_systems(Startup, setup_terminal_camera)
    .run();
```

#### What changed

| Aspect | Before | After |
|--------|--------|-------|
| Plugin set | `MinimalPlugins` + `InputPlugin` | `DefaultPlugins` (includes render, UI, text, state, winit, etc.) |
| Window config | Bevy default (tiny, resizable, generic title) | 1024×768, non-resizable, `"TOWERGATE // Network Intrusion Terminal"` |
| Background | Default light grey | `ClearColor(Color::BLACK)` |
| Camera | None | `Camera2d` spawned in `setup_terminal_camera` |
| Plugins order | State → Network → Agent | DefaultPlugins → State → Network → Agent + Startup camera |

The `setup_terminal_camera` function is a simple one-liner that spawns a `Camera2d` bundle. In Bevy 0.18, `Camera2d` automatically configures its required components (`Transform`, `Visibility`, `Camera`, `Camera2d`), so no additional setup is needed.

```rust
fn setup_terminal_camera(mut commands: Commands) {
    commands.spawn(Camera2d);
}
```

---

### 3.2 `game_state.rs` — Terminal HUD Implementation

This is the centerpiece of the evolution. Three additions were made to the state module:

#### Marker Components

```rust
#[derive(Component)]
pub struct CpuDisplayMarker;

#[derive(Component)]
pub struct AlertDisplayMarker;
```

These zero-sized marker types are attached to specific UI text entities. They serve as precise query targets, avoiding fragile lookups by entity index, name string, or parent-child traversal. This is a defensive design pattern: if the HUD layout changes (e.g. elements are reordered), the marker queries remain stable.

#### `StatePlugin` Changes

| Before | After |
|--------|-------|
| `app.add_plugins(StatesPlugin)` | Removed (auto-included in `DefaultPlugins`) |
| `insert_resource(CyberResources::default())` | Kept |
| — | `add_systems(Startup, spawn_terminal_hud)` |
| — | `add_systems(Update, refresh_terminal_hud)` |

#### `spawn_terminal_hud` — Layout Structure

The HUD is built with Bevy's flexbox-based UI (`bevy_ui`). The layout tree is:

```
Root Node (100% x 100%, Column flex, 20px padding)
├── Header Banner (100% x 50px, SpaceBetween, dark green bg)
│   ├── Text: "CORE_UPLINK // TOWERGATE_OS" (green)
│   ├── Text: "CPU: 4/4" (bright green)  ← CpuDisplayMarker
│   └── Text: "ALERT DETECT: 0%" (red)   ← AlertDisplayMarker
└── Workspace Panel (100% x 90%, 15px top margin)
```

Design choices:
- **Column flex layout** allows the workspace panel to naturally expand below the header as the viewport grows.
- **Dark green background** (`Color::linear_rgb(0.0, 0.05, 0.0)`) on the header gives a terminal aesthetic without overwhelming the workspace area.
- **Green/red color coding** for CPU and alert text follows terminal conventions (safe vs. danger).
- **20px outer padding** prevents UI elements from touching window edges — consistent with terminal emulator behavior.

#### `refresh_terminal_hud` — Reactive Update Pattern

```rust
fn refresh_terminal_hud(
    resources: Res<CyberResources>,
    mut params: ParamSet<(
        Option<Single<&mut Text, With<CpuDisplayMarker>>>,
        Option<Single<&mut Text, With<AlertDisplayMarker>>>,
    )>,
) {
    if resources.is_changed() {
        if let Some(mut text) = params.p0() {
            text.0 = format!("CPU: {}/{}", resources.available_cpu_cycles, resources.total_cpu_cycles);
        }
        if let Some(mut text) = params.p1() {
            text.0 = format!("ALERT DETECT: {}%", resources.security_alert_level);
        }
    }
}
```

Key design decisions:

- **`ParamSet`** — Bevy normally forbids multiple `Single<&mut T>` queries in the same system because they would alias the same component type (`Text`). `ParamSet` works around this by running each parameter group sequentially, pausing borrows between groups. This is safe because `CpuDisplayMarker` and `AlertDisplayMarker` are disjoint component sets.

- **`Option<Single<...>>`** — Wrapping `Single` in `Option` prevents a runtime panic if the corresponding UI entity hasn't been spawned yet (e.g. during a brief frame at startup where `spawn_terminal_hud` hasn't run but the update system fires). This is a defensive pattern — under normal conditions the entities always exist.

- **`resources.is_changed()`** — The system only mutates text strings when `CyberResources` has actually been modified by the breach mechanic. This avoids unnecessary string allocations every frame.

---

## 4. Architecture Decisions

### 4.1 `DefaultPlugins` vs `MinimalPlugins`

**Context:** The original code used `MinimalPlugins` + `InputPlugin` to keep the compile surface small during early logic development.

**Decision:** Switch to `DefaultPlugins` configured with a custom `WindowPlugin`.

**Rationale:**
- `DefaultPlugins` is the standard Bevy entry point for any project that renders visuals.
- It includes `StatesPlugin`, `UiPlugin`, `TextPlugin`, `RenderPlugin`, `CameraPlugin` and all other subsystems required by the HUD.
- The custom `WindowPlugin.set(...)` override replaces only the window configuration while keeping the default behavior for all other sub-plugins.
- Compile time concern is secondary once the project reaches a visual stage — the extra features are now required, not optional.

### 4.2 Marker Components Over Entity-Based Queries

**Context:** The HUD needs to update two specific `Text` entities on every resource change.

**Decision:** Use dedicated zero-sized marker components (`CpuDisplayMarker`, `AlertDisplayMarker`) with `Single` queries.

**Rationale:**
- **Entity references** (storing `Entity` IDs in a resource) risk dangling references if the UI is rebuilt.
- **Name-based queries** are stringly-typed and slower.
- **Parent-child traversal** couples the update logic to the layout tree.
- Markers are compile-time checked, performant, and decoupled from layout structure.

### 4.3 `ParamSet` for Same-Type Mutations

**Context:** Both HUD elements use `&mut Text`, which would normally be an aliasing conflict in Bevy's query system.

**Decision:** Use `ParamSet<(...)>` to interleave access to the two `Single<&mut Text>` queries.

**Rationale:**
- `ParamSet` is Bevy's native solution for multiple mutable accesses to the same component type — provided the queries have disjoint `With<>` filters.
- This is safer and more idiomatic than splitting the system into two separate systems or using raw `World` access.

---

## 5. Code Styles & Patterns Observed

The diff reveals a consistent authorial voice across the codebase:

| Pattern | Example | Signature |
|---------|---------|-----------|
| **Inline doc comments** | `/// Decoupled update system utilizing native 0.18 Option<Single> parameters.` | Mixed-length, opinionated, descriptive |
| **`println!` logging** | `println!("[SUCCESS] {} compromised!", ...)` | Bracket-prefixed tags (`[SYS]`, `[ACTION]`, `[SUCCESS]`, `[FAILED]`, `[ALERT]`) |
| **Explicit defensive `Option` wrapping** | `Option<Single<&mut Text, With<CpuDisplayMarker>>>` | Prefer safe-query over unwrap |
| **Layout with `..default()`** | `Node { ..., ..default() }` | Bevy 0.18's required struct initialization pattern |
| **Named marker components** | `CpuDisplayMarker`, `AlertDisplayMarker` | PascalCase, `Marker` suffix |

---

## 6. Project Structure Update

```
towergate/
├── .cargo/
│   └── config.toml
├── src/
│   ├── main.rs              # Windowed App with Camera2d
│   ├── game_state.rs        # + Terminal HUD (spawn + refresh)
│   ├── network.rs           # Unchanged
│   └── agent.rs             # Unchanged
├── docs/
│   ├── architecture.md      # Baseline (as of 8c561ad)
│   └── evolution-01-terminal-hud-overlay.md  # ← This file
├── Cargo.toml
└── Cargo.lock
```

---

## 7. Testing Strategy

No tests were added or modified in this evolution. The existing unit tests in `agent.rs` continue to validate the breach mechanics independently of the rendering layer.

The HUD is exercised implicitly through Bevy's system scheduling — if `spawn_terminal_hud` panics (e.g. due to a malformed layout node), the `cargo run` binary will crash on startup. A future improvement could add snapshot or integration tests for the UI.

---

## 8. Git History — Release Narrative

| Commit | What happened |
|--------|---------------|
| `a52e17c` | Authored `docs/architecture.md` documenting the project up to `8c561ad`. |
| `b4ac03c` | Switched to `DefaultPlugins`, configured 1024×768 window, spawned `Camera2d`. Added terminal HUD with marker components and `ParamSet`-based reactive refresh. |

---

## 9. Known Gaps & Future Work

### New gaps introduced

- **No interactions in the workspace panel** — The HUD header is rendered, but the main workspace area (90% of the screen) is an empty flex node. The network nodes, targeting UI, and game log have no visual representation yet.
- **CPU/ALERT display only updates on `is_changed()`** — If `CyberResources` is mutated outside the breach system (e.g. by a future passive tick system), the HUD will react. But if `is_changed()` returns false due to stale change detection, the display may lag. This is unlikely with the current single-source mutation pattern but worth monitoring.
- **No Game Over screen** — The alert display can reach 100%, but there is no visual transition to a terminal state.

### Previously noted gaps (unchanged)

- `NodeInfiltration` and `GameOver` states remain inert.
- Network nodes are still hardcoded.
- Single-node targeting unchanged.
- No persistence or audio.

### Natural next steps

1. Render network nodes as interactive elements in the workspace panel.
2. Implement the `GameOver` state transition and a corresponding screen.
3. Add a targeting mechanic (clickable nodes, highlight on hover).
4. Collapse the `StatesPlugin` explicit wiring now that `DefaultPlugins` auto-includes it.

---

## 10. Build & Run

```sh
cd towergate
cargo run           # Opens a 1024×768 window with the terminal HUD
cargo test          # Unit tests unchanged, still pass
cargo check         # Fast compile-time check
```

> You can verify the HUD in action: launch the game, press Space to trigger a breach attempt, and watch the CPU and ALERT DETECT values update in real time in the top-right header area.