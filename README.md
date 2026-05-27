# Towergate

**A turn-based hacking / infiltration game built with [Bevy 0.18.1](https://bevyengine.org/) (Rust).**

In Towergate you play as Icebreaker, a hacker agent who scans virtual subnets, picks targets, and breaches their firewalls with limited CPU cycles. Each success grants access to a network node; each failure raises the security alert level and brings the trace closer to you.

The project is developed incrementally — every commit adds a well-defined slice of gameplay. See the [docs](docs/) for the full architecture and evolution log.

---

## Current State

Towergate is in early but functional development. You can:

- Open a 1024×768 terminal-styled window
- See a live HUD with CPU cycles and security alert level
- Press **Space** to attempt a breach on the first network node
- Watch the HUD update in real time on success or failure

![Screenshot placeholder](docs/evolution-01-terminal-hud-overlay.md)

---

## Quick Start

```sh
cargo run      # Launch the game
cargo test     # Run unit tests
cargo check    # Fast compile check (preferred during iteration)
```

---

## Project Structure

```
towergate/
├── src/
│   ├── main.rs              # Application entrypoint (window, camera, plugins)
│   ├── game_state.rs        # State machine, player resources, terminal HUD
│   ├── network.rs           # Network topology (NetworkNode component)
│   └── agent.rs             # HackerAgent, breach mechanics, unit tests
├── docs/
│   ├── architecture.md      # Baseline architecture commentary
│   └── evolution-01-terminal-hud-overlay.md  # First feature evolution
├── .agents/
│   └── skills/              # Zed agent skills (release-doc, etc.)
├── Cargo.toml
└── README.md
```

---

## Documentation

| Doc | Description |
|-----|-------------|
| [`docs/architecture.md`](docs/architecture.md) | Full architecture breakdown — vision, modules, design decisions, limitations |
| [`docs/evolution-01-terminal-hud-overlay.md`](docs/evolution-01-terminal-hud-overlay.md) | First evolution: headless → windowed app with terminal HUD |
| `docs/evolution-02-*.md` | Next (when ready) |

---

## Tech Stack

- **Engine:** Bevy 0.18.1 (ECS, state, UI, render, sprite, text)
- **Language:** Rust (edition 2024)
- **Rendering:** 2D camera with `bevy_ui` flexbox layout
- **Linker:** `lld` (Linux) / `zld` (macOS) — configured in `.cargo/config.toml`

---

## License

All rights reserved unless otherwise specified.