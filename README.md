# hafthiWorld

Rust + Bevy 0.20 settlement simulation inspired by Viking Age Gotland. The second scoped milestone from [issue #1](https://github.com/pdahlbeck/hafthiWorld/issues/1) adds a playable resource chain: **order tree felling → carry logs to storage → carry stored timber to a construction site → finish a timber rack**. Strategy and first person observe the same running ECS world. The coastal scene, trees, worker and rack remain explicit geometric prototypes; this is a logistics milestone, not the later art pass.

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
| Left click | Select a tree crown/stump, or use the order buttons | Capture mouse after release/focus loss |
| 1 / 2 / 3 | Select tree | Select tree while mouse is captured |
| F | Order selected tree felled | Same order while mouse is captured |
| B | Order the fixed timber rack (two logs) | Same order while mouse is captured |
| Escape | Quit gracefully | Release mouse; press again to return to strategy |

The first visit spawns at a safe meadow position. Later visits retain the walking position and look direction. Strategy focus, orbit angle and zoom survive first-person visits. Walking stays on the meadow and slides around the house and limestone footprint. Losing window focus releases the mouse; returning to the window requires a click to resume navigation. Releasing the mouse or changing camera mode does not pause simulation. Close the window to quit from either mode.

## Harvest, store, build

Three trees contain **two log units each**. Nothing is harvested automatically. Click a crown in strategy view, click a numbered button, or press **1–3** to select one (the gold ground marker shows selection). Press **F** or **Harvest**. The worker walks to that tree, spends three simulation seconds felling it, and leaves a stump and two physical logs. Those same log entities are carried individually to the stockpile. Repeating the order cannot regenerate a tree or create more logs.

Press **B** or **Build rack** to order the single predefined timber rack beside the stockpile. It can be ordered before any wood exists: it waits. The worker picks up **stored** logs, carries each to the site and installs it. Each pickup removes that log from available storage. After two deliveries and three seconds of assembly, the crossbeam appears and the rack is complete. No resources are deducted merely for placing an order, and repeated orders never charge twice. One tree supplies this rack; harvesting all three leaves four log units in storage after construction.

The worker prioritizes supplying an ordered rack from available storage, then loose logs, then queued trees. Already-started work finishes before the next task is chosen. The HUD exposes the persistent worker ID, task, selected tree, time and material locations. Every unit is in exactly one state: **in tree, on ground, carried, stored or installed**; their sum stays six. Installed units are represented by the rack structure rather than duplicate log meshes. Camera switching does not spawn replacement entities or reset tasks, orders or stocks.

This scope has one worker, three fixed trees, one material and one fixed building site. It does not include free placement, advanced pathfinding, multiple workers, replanting, durability, construction recipes, households or saved games. Routes are authored to stay clear of the existing house/limestone footprint; the small rack is an open prototype frame without its own collision. Restarting resets the session. Tree yield, rack cost, dimensions and work durations are technical/art choices, not historical measurements.

## Architecture

- `world.rs`: the existing coastal scene, shared meadow bounds and obstacle queries.
- `camera.rs`: explicit camera-mode resource, independent strategy/first-person poses, collision-aware movement, focus and cursor policy. It never mutates simulation.
- `simulation.rs`: fixed-step worker tasks, finite tree/log states, construction state and clock. Pure work progression has no camera/rendering/input dependency.
- `orders.rs`: selection, tree ray picking, keyboard/native UI button orders and focus/mode guards. It records intent; it never moves workers or grants material.
- `presentation.rs`: persistent worker/tree/log entities, axe motion, selection marker, construction parts, native Bevy 0.20 buttons and HUD reading shared state in `Update`. It does not advance the economy.
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

Eleven tests cover the complete material chain, waiting for missing timber, idempotent orders, finite yields, frame partitioning, authored traversable routes, preserved entity identities across camera changes, clickable-button activation, keyboard orders in both modes, input suppression on focus loss/mouse release, retained camera poses and existing collision.

Linux software-Vulkan smoke captures for this milestone are in `docs/screenshots/milestone2-*.png`. They document functionality; macOS/Metal performance still needs desktop testing.

### Manual smoke test (macOS or Linux desktop)

1. Start the app. The HUD shows six units **in trees**, zero stored/installed, an idle worker and a rack that is not ordered. Wait: no material should appear automatically.
2. Click the **Tree 1** and **Harvest** buttons (or press **1**, **F**). Order **Build rack** immediately, before any timber reaches storage. The rack must wait for timber.
3. Observe the worker reach the selected tree and swing the axe. After felling, its crown disappears, the stump remains and two logs appear. The worker picks up one; it disappears from the ground and is visibly carried.
4. Press **Tab** while carrying. Verify the same worker ID, task and material counts. Follow the worker on foot. It unloads at the stockpile, later picks up stored timber and carries it to the rack. Two installed posts and then the finished crossbeam demonstrate delivery and assembly.
5. Return with Tab: the original strategy pose returns. Try ordering **F/B** from first person while the mouse is captured. Escape releases the mouse; F/B should then do nothing. Focus loss must release the mouse and suppress orders until recaptured.
6. Reissue harvest for the felled tree and build for the completed rack. No additional logs or costs should appear. Select the other two trees and order harvest; after all work finishes, **stored = 4, installed = 2**, the sum of all locations remains six, and the worker idles.
7. Existing walking/house/limestone/shore collision, camera pan/orbit/zoom and graceful Escape/window-close exit should continue to work.

## Direction and license

Future work: resource orders and construction, Gotland visual refinement, farming/households, seasons, barter and Baltic shipping. Follow [AGENTS.md](AGENTS.md) and the scoped PR sequence in issue #1. Do not copy proprietary game assets or claim historical certainty without evidence.

A project license has not yet been selected. All rights reserved until the repository owner chooses one.
