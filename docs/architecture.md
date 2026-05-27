# Towergate — Architecture & Implementation Commentary

**Project:** `towergate` (v0.1.0)  
**Engine:** [Bevy 0.18.1](https://bevyengine.org/) (Rust, edition 2024)  
**Repository:** [github.com/patricktabar/towergate](https://github.com/patricktabar/towergate)

---

## 1. Vision

Towergate is a turn-based hacking / infiltration game built with Bevy. The player takes the role of a hacker agent ("Icebreaker") who scans a virtual subnet, picks targets, and attempts to breach their firewalls using limited CPU cycles. Successes grant access to network nodes; failures raise the security alert level and inch the player closer to a Game Over.

The game is developed incrementally: each commit adds a well-defined slice of gameplay mechanics. This document captures the architecture as of commit `8c561ad`.

---

## 2. Project Structure

```
towergate/
├── .cargo/
│   └── config.toml          # Linker configuration (lld/zld for fast linking)
├── src/
│   ├── main.rs              # Application entrypoint
│   ├── game_state.rs        # State machine & player resources
│   ├── network.rs           # Network topology (nodes)
│   └── agent.rs             # Hacker agent entity & breach logic
├── docs/
│   └── architecture.md      # This document
├── Cargo.toml               # Dependency manifest
└── Cargo.lock               # Locked dependency versions
```

Each source file is a self-contained Bevy **plugin** with its own types, systems, and (where applicable) tests — a clean domain-driven decomposition.

---

## 3. Core Dependency: Bevy 0.18.1

The game is built on Bevy, a modern ECS (Entity-Component-System) game engine for Rust. The `Cargo.toml` explicitly selects optional feature flags to keep the compile surface minimal:

| Feature             | Purpose                                        |
|---------------------|------------------------------------------------|
| `std`               | Standard library (required by `bevy_ecs`)      |
| `bevy_winit`        | Window creation and event loop                 |
| `bevy_render`       | Core rendering pipeline                        |
| `bevy_core_pipeline`| 2D/3D camera pipelines                         |
| `bevy_sprite`       | 2D sprite rendering                            |
| `bevy_text`         | Terminal / text rendering                      |
| `bevy_ui`           | UI elements                                    |
| `bevy_state`        | Finite state machine (states, `in_state`)      |
| `keyboard`          | Keyboard input (`ButtonInput<KeyCode>`)        |
| `multi_threaded`    | Parallel ECS system execution                  |
| `x11`               | X11 backend for windowing                      |

> **Note:** The project currently uses `MinimalPlugins` + `InputPlugin` rather than the full `DefaultPlugins`, so there is no visible game window yet. This is intentional — the team is building game logic first and will layer on rendering once the core loop is solid.

### Bevy 0.18 → 0.18.1 Migration

Commit `8c561ad` upgraded Bevy from 0.18 to 0.18.1, which introduced a new [observable / message-based event API](https://bevyengine.org/news/bevy-0-18-1/#message-based-events). The following changes were made:

- `Event` / `EventWriter<T>` / `EventReader<T>` → `Message` / `MessageWriter<T>` / `MessageReader<T>`
- `.add_event::<T>()` → `.add_message::<T>()`
- `.send(event)` → `.write(event)`
- `Query<&HackerAgent>` → `Single<&HackerAgent>` (guaranteed single-entity query, panics otherwise)
- Rust edition bumped to 2024

---

## 4. Module-by-Module Breakdown

### 4.1 `main.rs` — Application Entrypoint

```rust
App::new()
    .add_plugins((MinimalPlugins, InputPlugin))
    .add_plugins(StatePlugin)
    .add_plugins(NetworkPlugin)
    .add_plugins(AgentPlugin)
    .run();
```

Responsibilities:
- Assembling the Bevy `App` with three game-domain plugins layered on top of minimal engine infrastructure.
- The plugin ordering is intentional: `StatePlugin` must be added first so that `GamePhase` states exist before systems from other plugins try to `run_if(in_state(...))`.

---

### 4.2 `game_state.rs` — Game Phases & Resources

This module defines the game's **finite state machine** and **global resources**.

#### `GamePhase` (State)

```rust
enum GamePhase {
    NetworkMapping,    // Scanning the architecture (default)
    NodeInfiltration,  // Actively breaching a specific firewall
    GameOver,
}
```

| Phase            | Description                                           |
|------------------|-------------------------------------------------------|
| `NetworkMapping` | Initial phase. Player can scan and breach nodes.      |
| `NodeInfiltration` | Reserved for deeper interactions with a breached node. |
| `GameOver`       | Terminal state (reached when `security_alert_level >= 100`). |

Currently only `NetworkMapping` has active systems; the other two states exist as scaffolding.

#### `CyberResources` (Resource)

```rust
struct CyberResources {
    total_cpu_cycles: u32,        // Always 4
    available_cpu_cycles: u32,    // Decremented per breach attempt
    security_alert_level: u32,    // Reaching 100 = Game Over
}
```

- Each breach attempt consumes 1 CPU cycle.
- A failed breach adds 25 to `security_alert_level`.
- When `security_alert_level` reaches 100, the game should transition to `GameOver` (not yet implemented).

#### `StatePlugin`

```rust
app.add_plugins(StatesPlugin)
   .init_state::<GamePhase>()
   .insert_resource(CyberResources::default());
```

`StatesPlugin` is Bevy 0.18's required plugin for the state API. `init_state` registers `GamePhase` with its `Default` variant (`NetworkMapping`).

---

### 4.3 `network.rs` — Network Topology

Defines what the player interacts with.

#### `NetworkNode` (Component)

```rust
struct NetworkNode {
    ip_address: String,
    firewall_strength: u32,
    is_compromised: bool,
}
```

Each `NetworkNode` is an Entity in the ECS with this component and a `Name` component for display.

#### `spawn_initial_subnet`

Runs at `Startup`. Currently spawns two static nodes:

| Name              | IP Address    | Firewall Strength |
|-------------------|---------------|-------------------|
| Gateway Router    | 192.168.1.1  | 10                |
| Mainframe Core    | 10.0.4.23    | 10                |

Both are `is_compromised: false` by default. This function is the seed of what will eventually become a procedural network generation system.

---

### 4.4 `agent.rs` — Player Agent & Breach Mechanics

The largest module, containing the player's avatar, input handling, breach execution, and unit tests.

#### `HackerAgent` (Component)

```rust
struct HackerAgent {
    decryption_suite: u32,  // Attack power; currently 15
}
```

Spawned once at startup as `Player_Icebreaker`. Uses `Single<&HackerAgent>` (a Bevy 0.18.1 query that guarantees exactly one matching entity) in the breach system — a safe design choice since there is always exactly one player.

#### Event Flow

```
[Space] pressed
    → listen_for_input checks ButtonInput<KeyCode>
    → picks first NetworkNode entity
    → writes BreachAttemptEvent { target }

execute_breach reads the event
    → checks available_cpu_cycles > 0
    → decrements cycles
    → compares agent.decryption_suite vs node.firewall_strength
    → on success: node.is_compromised = true
    → on failure: security_alert_level += 25
```

Both systems run only during `GamePhase::NetworkMapping`, gated via `.run_if(in_state(GamePhase::NetworkMapping))`.

> **Design note:** In the 0.18.1 migration, the two systems had their `run_if` clauses moved from a shared tuple to individual system-level `.run_if()`. This is functionally equivalent but more explicit and easier to refactor when systems are later split across different states.

#### `BreachAttemptEvent` (Message)

```rust
#[derive(Message)]
struct BreachAttemptEvent { target: Entity }
```

Registered via `app.add_message::<BreachAttemptEvent>()`. This is an **observable message** (Bevy 0.18.1's replacement for `Event`). Messages are well-suited here because:

- Multiple consumers could react to a breach (sound effects, animations, state transitions).
- The message API supports labelled observers, making future system hookup more flexible.

#### Resource Consumption Model

```
available_cpu_cycles = 4 (initially)
  ↓ per breach: -1
  └─ if == 0: breach is blocked, event is skipped
```

```
security_alert_level = 0 (initially)
  ↓ per failed breach: +25
  └─ at 100: game should transition to GameOver (not yet implemented)
```

This creates a meaningful risk/reward tension: the player can attempt at most 4 actions per "round," and each failure costs a quarter of the alert budget.

---

## 5. Testing Strategy

Tests live inside `agent.rs` as `#[cfg(test)] mod tests`. They use Bevy's `World` directly (no `App`), which gives fast, deterministic, headless unit tests.

| Test                              | What it verifies                                                |
|-----------------------------------|----------------------------------------------------------------|
| `test_breach_success`             | Agent with `decryption_suite: 15` beats `firewall: 10`, CPU consumed |
| `test_breach_fails_when_firewall_too_strong` | Agent with `decryption_suite: 5` loses to `firewall: 100`, alert +25 |
| `test_insufficient_cpu_cycles_blocks_breach` | `available_cpu_cycles: 0` prevents breach, node stays safe         |

These tests manually simulate the resource mutations that `execute_breach` would perform, without needing the full Bevy scheduler. This is a pragmatic choice for early development — it validates the core algorithm without coupling to systems scheduling.

---

## 6. Git History — Development Narrative

| Commit | Message | What happened |
|--------|---------|---------------|
| `9f8d869` | "first commit" | Project scaffold, `Cargo.toml`, `.cargo/config.toml`, initial Bevy dependency |
| `be8f9a4` | "Add game state and network plugins" | `game_state.rs` and `network.rs` created. State machine, `CyberResources`, `NetworkNode` component, initial subnet spawn |
| `f10c77d` | "Add hacker agent and breach mechanics" | `agent.rs` created. `HackerAgent`, `BreachAttemptEvent`, input/breach systems, unit tests |
| `0041dde` | "Add Bevy app with state, network, agent plugins" | `main.rs` written wiring all three plugins into the `App` |
| `8c561ad` | "Upgrade Bevy to 0.18.1 and adapt to new API" | Migrated from `Event`/`EventWriter`/`EventReader` → `Message`/`MessageWriter`/`MessageReader`, `Query<&T>` → `Single<&T>`, Rust edition 2024 |

The commit history tells a clean story:

1. **Scaffold** — Dependency setup with linker optimizations.
2. **State + Data** — Establish the game loop's skeleton (phases) and core data types.
3. **Agent + Interaction** — Add the player entity, input handling, and the central breach mechanic.
4. **Integration** — Wire everything together in `main.rs`.
5. **Upgrade** — Stay current with Bevy's latest API, cleaning up verbosity and adopting `Single` / `Message`.

---

## 7. Current Limitations & Future Work

### Known Gaps

- **No rendering** — `MinimalPlugins` is used, so there is no game window. All output is via `println!`. A visual layer (2D sprites, UI overlays) is the obvious next step.
- **`NodeInfiltration` and `GameOver` are inert** — The state machine has three phases but only `NetworkMapping` is wired. Transition logic (`security_alert_level >= 100` → `GameOver`) is missing.
- **Hardcoded subnet** — The two network nodes are spawned statically. Procedural generation (randomized IPs, variable firewall strengths, topology graphs) would make the game replayable.
- **Single-node targeting** — `listen_for_input` always targets the *first* `NetworkNode` in the query. There is no targeting mechanic (cursor, list selection, etc.).
- **No persistence / save** — Game state is entirely in-memory.
- **No audio** — No sound effects or music.

### Natural Next Steps

1. Render the subnet as a node graph using `bevy_sprite` + `bevy_render`.
2. Implement `GameOver` transition and `NodeInfiltration` phase.
3. Add procedural network generation with varying firewall strengths.
4. Add a targeting UI (clickable nodes, highlight on hover).
5. Expand breach outcomes (data exfiltration, node defense, side-effects).

---

## 8. Build & Run

```sh
cd towergate
cargo run           # Run the game (headless for now — watch stdout)
cargo test          # Run unit tests
cargo check         # Fast compile-time check (preferred during iteration)
```

> The `.cargo/config.toml` configures `lld` (Linux) / `zld` (macOS) / `lld-link.exe` (Windows) as linkers, and sets `opt-level = 3` for all dependencies in dev mode — a standard Bevy performance trick to keep frame-times low even during development.