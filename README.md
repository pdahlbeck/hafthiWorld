# hafthiWorld

Rust + Bevy 0.20 settlement simulation inspired by Viking Age Gotland. The first scoped milestone from [issue #1](https://github.com/pdahlbeck/hafthiWorld/issues/1) adds strategy/first-person views of **one running ECS simulation**. The existing meadow, shallow sea, limestone block, timber building and lighting remain the coastal visual prototype; inhabitants and logs are explicit geometric placeholders.

## Run

Install current stable Rust using [rustup](https://rustup.rs/) (edition 2024; tested here with Rust 1.99.1):

```sh
cargo run --release --locked
```

macOS uses Metal; Linux needs a working graphics driver/display and Bevy's native ALSA, udev, X11/Wayland and xkbcommon development libraries. No server, assets download or credentials are required to play. No macOS performance measurements were made in the Linux cloud environment.

## Controls

| Input | Strategy | First person |
| --- | --- | --- |
| Tab | Enter first person | Restore the previous strategy view |
| W/A/S/D | Orbit | Walk |
| Q/E or mouse wheel | Zoom | — |
| Arrow keys | Pan | — |
| Hold right mouse + drag | Orbit | — |
| Mouse movement | — | Look while captured |
| Left Shift | — | Run |
| Left click | — | Capture mouse after release/focus loss |
| Escape | Quit gracefully | Release mouse; press again to return to strategy |

The first visit spawns at a safe meadow position. Later visits retain the walking position and look direction. Strategy focus, orbit angle and zoom survive first-person visits. Walking stays on the meadow and slides around the house and limestone footprint. Losing window focus releases the mouse; returning to the window requires a click to resume navigation. Releasing the mouse or changing camera mode does not pause simulation. Close the window to quit from either mode.

## Shared simulation proof

One worker moves **six existing logs** from the supply pad at `(-5, -2.5)` to the stockpile at `(-2.5, 3.5)`. The worker walks to the supply, loads one log, carries it, unloads, and repeats until the finite supply is exhausted (roughly 75 simulation seconds). The carried log is taken from supply before appearing in storage: `supply + carried + stored = 6` throughout. The same six log entities change position; camera switching never spawns replacement workers/logs or resets stocks/tasks. The HUD exposes simulation time, worker entity ID, task, carried state and both stocks in either view.

This is a deterministic transport proof, not tree harvesting, player orders, construction, a complete economy or saved games. Simulation persists between modes during a session; restarting the app resets it. Those next steps belong to later PRs in issue #1.

## Architecture

- `world.rs`: the existing coastal scene, shared meadow bounds and obstacle queries.
- `camera.rs`: explicit camera-mode resource, independent strategy/first-person poses, collision-aware movement, focus and cursor policy. It never mutates simulation.
- `simulation.rs`: fixed-step ECS worker/task state, finite stocks and simulation clock. Pure transport logic has no camera/rendering/input dependency.
- `presentation.rs`: shared worker/log meshes and HUD reading simulation state in `Update`; no economy progression is performed here.
- `main.rs`: plugin composition and window setup.

`FixedUpdate` advances material flow regardless of camera mode. `Update` presents the current state. All geometry is original placeholder geometry; historical layouts, costumes and logistics timing are artistic/technical interpretations, not archaeological claims.

## Validation

```sh
cargo fmt --check
cargo check --locked
cargo test --locked
cargo clippy --locked -- -D warnings
cargo build --locked
```

Tests cover material conservation, load-before-store ordering, finite task completion, equivalent elapsed time with different step sizes, worker identity/progress through repeated camera changes, exact restoration of strategy pose, retained visitor position, cursor release on focus loss/Escape, and house/rock/shore collision including large displacements.

Linux software-Vulkan smoke captures: [strategy](docs/screenshots/milestone1-strategy.png) and [first person](docs/screenshots/milestone1-first-person.png). These demonstrate the same worker ID and continuing material flow, not a visual-quality or Apple Silicon performance benchmark.

### Manual smoke test (macOS or Linux desktop)

1. Start the app in strategy mode. Orbit, pan and zoom to a distinctive view. Note the HUD's worker ID, supply and simulation time.
2. Watch the worker collect a log (about six seconds). Verify supply decreases while carrying, and storage increases only after unloading.
3. Press Tab during transport. Verify the same worker ID/task/stocks and advancing simulation time. Walk with WASD and look with the mouse. Follow the worker between the two pads.
4. Press Escape: the cursor becomes visible and walking/look stop while the worker continues. Click to capture it again. Switch to another app and back; the pointer must be free until clicked.
5. Press Tab: the exact earlier strategy view returns. Repeat switching during loading, walking and unloading; neither stocks nor the task restart.
6. Try walking into the house, limestone block and sea. Movement should slide or stop at the boundary. Diagonal walking should not be faster.
7. After roughly 75 simulation seconds, all six logs must be at the stockpile and the worker must stop. Switch modes again: the count remains six. Quit with Escape from strategy or by closing the window.

## Direction and license

Future work: resource orders and construction, Gotland visual refinement, farming/households, seasons, barter and Baltic shipping. Follow [AGENTS.md](AGENTS.md) and the scoped PR sequence in issue #1. Do not copy proprietary game assets or claim historical certainty without evidence.

A project license has not yet been selected. All rights reserved until the repository owner chooses one.
