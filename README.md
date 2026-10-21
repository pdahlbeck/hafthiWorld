# hafthiWorld

Rust + Bevy 0.20 settlement simulation inspired by Viking Age Gotland. The second scoped milestone from [issue #1](https://github.com/pdahlbeck/hafthiWorld/issues/1) adds a playable resource chain: **order tree felling → carry logs to storage → carry stored timber to a construction site → finish a timber rack**. Strategy and first person observe the same running ECS world. Milestone 3 adds an original procedural coastal visual pass and portable performance instrumentation. The next scoped slice adds one finite, player-ordered crop cycle using the same worker and storage. The single worker, three finite trees and material-backed timber rack remain the same shared simulation in both views.

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
| T / Y | Pause/resume sunlight / advance three hours | Same presentation controls |
| F3 | Hide/show performance panel | Same panel |
| W/A/S/D | Orbit | Walk |
| Q/E or mouse wheel | Zoom | — |
| Arrow keys | Pan | — |
| Hold right mouse + drag | Orbit | — |
| Mouse movement | — | Look while captured |
| Left Shift | — | Run |
| Left click | Select a tree crown/stump, or use the order buttons | Capture mouse after release/focus loss |
| 1 / 2 / 3 | Select tree | Select tree while mouse is captured |
| F | Order selected tree felled | Same order while mouse is captured |
| O / R | Order sowing / reap the ripe field | Same orders while mouse is captured |
| B | Order the fixed timber rack (two logs) | Same order while mouse is captured |
| Escape | Quit gracefully | Release mouse; press again to return to strategy |

The first visit spawns at a safe meadow position. Later visits retain the walking position and look direction. Strategy focus, orbit angle and zoom survive first-person visits. Walking stays on the meadow and slides around the house and limestone footprint. Losing window focus releases the mouse; returning to the window requires a click to resume navigation. Releasing the mouse or changing camera mode does not pause simulation. Close the window to quit from either mode.

## Harvest, store, build

Three trees contain **two log units each**. Nothing is harvested automatically. Click a crown in strategy view, click a numbered button, or press **1–3** to select one (the gold ground marker shows selection). Press **F** or **Harvest**. The worker walks to that tree, spends three simulation seconds felling it, and leaves a stump and two physical logs. Those same log entities are carried individually to the stockpile. Repeating the order cannot regenerate a tree or create more logs.

Press **B** or **Build rack** to order the single predefined timber rack beside the stockpile. It can be ordered before any wood exists: it waits. The worker picks up **stored** logs, carries each to the site and installs it. Each pickup removes that log from available storage. After two deliveries and three seconds of assembly, the crossbeam appears and the rack is complete. No resources are deducted merely for placing an order, and repeated orders never charge twice. One tree supplies this rack; harvesting all three leaves four log units in storage after construction.

The worker prioritizes supplying an ordered rack from available storage, then loose logs, then queued trees. Already-started work finishes before the next task is chosen. The HUD exposes the persistent worker ID, task, selected tree, time and material locations. Every unit is in exactly one state: **in tree, on ground, carried, stored or installed**; their sum stays six. Installed units are represented by the rack structure rather than duplicate log meshes. Camera switching does not spawn replacement entities or reset tasks, orders or stocks.

The timber chain has one worker, three fixed trees, one timber material and one fixed building site. The crop slice below shares that same worker and adds separate seed/grain accounting. It does not include free placement, advanced pathfinding, multiple workers, replanting, durability, construction recipes, households or saved games. Routes are authored to stay clear of the existing house/limestone footprint; the small rack is an open prototype frame without its own collision. Restarting resets the session. Tree yield, rack cost, dimensions and work durations are technical/art choices, not historical measurements.

## Architecture

- `world.rs`: deterministic irregular shore, shallow sea bands and wave lines, shared grass patches with wind, pine group, timber farm silhouette, sunlight/sky/fog and shared shore/obstacle queries.
- `performance.rs`: low/high render settings, rolling CPU wall-frame statistics and a timed CSV benchmark. It never changes settlement resources or work.
- `camera.rs`: explicit camera-mode resource, independent strategy/first-person poses, collision-aware movement, focus and cursor policy. It never mutates simulation.
- `simulation.rs`: fixed-step worker tasks, finite tree/log states, construction state, field labour/transport and clock. One persistent worker handles both timber and farming.
- `farming.rs`: one finite crop cycle, seed/grain accounting and persistent field/crop/sack visuals. It reads the shared settlement/worker state; it never advances work in rendering. Pure work progression has no camera/rendering/input dependency.
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

Twenty-one tests cover the complete material chain, waiting for missing timber, idempotent orders, finite yields, frame partitioning, authored traversable routes, preserved entity identities across camera changes, clickable-button activation, keyboard orders in both modes, input suppression on focus loss/mouse release, retained camera poses and existing collision. Checks also cover shore/pine collision, daylight, benchmark arguments and slow-frame percentiles. Crop tests cover seed use, premature/repeated orders, physical delivery, mixed timber/farming routes, frame partitions, focus guards and persistent field/cargo entities across camera changes.

Linux software-Vulkan smoke captures for this milestone are in `docs/screenshots/milestone2-*.png`. They document functionality; macOS/Metal performance still needs desktop testing.

### Manual smoke test (macOS or Linux desktop)

1. Start the app. The HUD shows six units **in trees**, zero stored/installed, an idle worker and a rack that is not ordered. Wait: no material should appear automatically.
2. Click the **Tree 1** and **Harvest** buttons (or press **1**, **F**). Order **Build rack** immediately, before any timber reaches storage. The rack must wait for timber.
3. Observe the worker reach the selected tree and swing the axe. After felling, its crown disappears, the stump remains and two logs appear. The worker picks up one; it disappears from the ground and is visibly carried.
4. Press **Tab** while carrying. Verify the same worker ID, task and material counts. Follow the worker on foot. It unloads at the stockpile, later picks up stored timber and carries it to the rack. Two installed posts and then the finished crossbeam demonstrate delivery and assembly.
5. Return with Tab: the original strategy pose returns. Try ordering **F/B** from first person while the mouse is captured. Escape releases the mouse; F/B should then do nothing. Focus loss must release the mouse and suppress orders until recaptured.
6. Reissue harvest for the felled tree and build for the completed rack. No additional logs or costs should appear. Select the other two trees and order harvest; after all work finishes, **stored = 4, installed = 2**, the sum of all locations remains six, and the worker idles.
7. Existing walking/house/limestone/shore collision, camera pan/orbit/zoom and graceful Escape/window-close exit should continue to work.

## First crop cycle (next scoped milestone)

A single tilled plot lies southeast of the house. Press **O** or **Sow field**. The existing worker collects the four initial seed units from the stockpile, carries their sack around the house via the southern approach, and spends four simulation seconds sowing. Seeds move from **stored → carried → used** exactly once. Until that actual work completes, the field cannot grow.

The crop then grows for **30 simulation seconds**, visibly rising and changing from green to gold. It does not reap itself. Wait for **ripe** in the HUD and press **R** or **Reap crop**; premature orders are rejected with feedback. After walking back and four seconds of reaping, the worker leaves four finite grain sacks at the field and carries each separately to the stockpile. Each is either **unproduced, on the field, carried or stored**. The HUD shows those material counts separately from timber. No grain is credited merely for issuing an order, and repeating O/R cannot recreate seed, regrow the field or duplicate grain.

Timber delivery/building, loose logs and ordered trees retain their existing priority; grain hauling and field work follow. A current task completes before another is accepted. Farm travel uses the southern stockpile approach rather than crossing the house. Crop growth advances with the fixed simulation even when the worker is busy elsewhere or either camera is active. **T/Y change presentation sunlight only** and never speed up, pause or reset crops. One sow/harvest cycle is available per session; restarting resets it.

This is one field and one worker, with illustrative grain geometry and arbitrary seed/yield units. It does not yet implement species-specific cultivation, seed saving/replanting, food consumption, seasons, livestock, households, free field placement or saved games. The short growth/work durations are testing choices, not agricultural or historical measurements. Earlier milestone-3 benchmark results remain a record of that earlier scene; no new hardware performance claim is made for the crop slice.

Screenshots: [ripe plot in strategy](docs/screenshots/milestone4-ripe-strategy.png), [the same plot in first person](docs/screenshots/milestone4-ripe-first-person.png), and [completed delivery](docs/screenshots/milestone4-stored.png).

### Crop smoke test

1. Start a fresh session. Verify four seed units stored, no grain produced and an unplanted field. Wait: nothing should grow automatically.
2. Press O twice. Follow the same worker carry the seed sack, sow, and return through the southern approach. The initial stock moves to carried, then used; it must not be charged twice.
3. Press R before the crop is ripe: no harvest should start. Pause sunlight with T, advance its hour with Y and switch cameras with Tab. Crop growth must continue, with the same worker/entity IDs and stocks.
4. At ripe, press R twice. Observe reaping, ground sacks, one carried sack at a time, and four sacks eventually stored. Repeated O/R after completion must not add resources.
5. Repeat while also ordering tree felling and the rack. Timber must still produce six total units, install two and store four after all three trees are harvested; grain remains exactly four in storage. The automated test also checks every sampled route stays walkable.

## Coastal visual pass (milestone 3)

The small clearing now has a faceted limestone shore, opaque Baltic color bands and broken animated wave lines, wind-blown batched grass, a background pine group, a timber building with boards/door/gables/pitched roof, distance haze and warm late-afternoon sunlight. Terrain stays flat at the worker paths; camera walking follows the same irregular shore mesh with a dry-land setback and trunk clearance. House/limestone collision and all authored logistics routes remain in place. Decorative vegetation does not create harvestable resources.

**T** pauses the presentation sun; **Y** advances three hours so dawn, dusk and night can be inspected without waiting. One full visual day lasts 12 minutes of unclamped game time. This clock changes lighting/sky/haze only; it does not change the economy or reset when switching cameras. See [visual references and interpretation limits](docs/visual-references.md), [strategy screenshot](docs/screenshots/milestone3-strategy.png) and [first-person screenshot](docs/screenshots/milestone3-first-person.png). The world remains a small stylized diorama; this does not implement the eventual 2×2 km map, weather or seasonal simulation.

```sh
cargo run --release --locked -- --quality high
cargo run --release --locked -- --quality low
```

High (default) requests 180 grass patch positions and directional shadows; low requests 60 and disables those shadows. Positions inside obstacles or around the stockpile approach are omitted. Each patch has 28 crossed blades in one shared mesh; rocks, foliage and building boards also share mesh/material handles for Bevy's batching and normal frustum culling. Only grass transforms and wave-line transforms animate. This is a small bounded scene, not a general distance-LOD system. Standard Bevy raster rendering is used on Metal/Vulkan; no Solari/ray-tracing requirement is introduced.

**F3** toggles a panel with a rolling 240-frame window, mean frame time, weighted FPS and nearest-rank p95. The text refreshes at most four times per second. These are CPU wall intervals between frames, including scheduling/presentation/GPU waits, **not GPU timestamp measurements**.

### Reproducible performance runs

On Apple Silicon, run natively from a graphical macOS session; keep power mode, display resolution and thermal conditions fixed and stop other heavy jobs. The script builds release once, runs both quality levels and both fixed camera poses, saves hardware/build metadata, CSV samples and renderer logs, and fails if any run fails:

```sh
bash scripts/benchmark.sh /tmp/hafthi-metal-run1
```

The default is three repeats of each case, 10 seconds warmup followed by 30 seconds sampling at 1280×800, fixed 16:00 lighting, idle settlement and requested `AutoNoVsync`. The driver may still limit presentation. Leave the window foreground, do not resize or interact during sampling, and inspect renderer logs to confirm the adapter/backend. Report mean, p50, p95, p99 and sample count per run; keep cold/warm driver-cache conditions distinct. Compare matching cases across commits rather than quoting the HUD as a benchmark.

```sh
# One individual run; a missing/unwritable output path fails with a nonzero exit.
cargo run --release --locked -- --quality high --benchmark 30 \
  --benchmark-camera first-person --benchmark-output /tmp/first-person.csv
# Shorter matrix, useful for validating the measurement pipeline:
HAFTHI_BENCHMARK_REPEATS=1 HAFTHI_BENCHMARK_SECONDS=20 \
  bash scripts/benchmark.sh /tmp/hafthi-short-run
```

Benchmarking freezes the sun, skips warmup frames, exports every sampled wall-frame interval and exits after flushing the CSV. It exercises idle rendering, not the worst-case future settlement workload. Linux cloud software-Vulkan results and their limitations are recorded in [the performance report](docs/performance/milestone3.md). **Apple Silicon/Metal measurements remain unrun**; the portable measurement path is implemented, but this Linux host cannot validate macOS performance.

For the visual smoke check, inspect both camera modes, wind/water movement, T/Y and the performance panel; then repeat the seven-step harvest/store/build smoke test above. Changing light or quality must not mint resources or replace simulation entities.

## Direction and license

Future work: resource orders and construction, Gotland visual refinement, farming/households, seasons, barter and Baltic shipping. Follow [AGENTS.md](AGENTS.md) and the scoped PR sequence in issue #1. Do not copy proprietary game assets or claim historical certainty without evidence.

A project license has not yet been selected. All rights reserved until the repository owner chooses one.
