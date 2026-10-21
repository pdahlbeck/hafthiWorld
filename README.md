# hafthiWorld

Rust + Bevy 0.20 settlement simulation inspired by Viking Age Gotland. The second scoped milestone from [issue #1](https://github.com/pdahlbeck/hafthiWorld/issues/1) adds a playable resource chain: **order tree felling → carry logs to storage → carry stored timber to a construction site → finish a timber rack**. Strategy and first person observe the same running ECS world. Milestone 3 adds an original procedural coastal visual pass and portable performance instrumentation. The following small slices add a crop/seed loop, a two-sheep fleece chain and a first owned household pantry and a finite shore barter offer, using the same worker and storage. The single worker, three finite trees and material-backed timber rack remain the same shared simulation in both views.

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
| C | Order both sheep clipped | Same order while mouse is captured |
| H | Order stored grain supplied to the home | Same order while mouse is captured |
| V | Order one stored sack per harvest saved as seed | Same order while mouse is captured |
| J | Order two stored wool per salt pouch at the shore visitor | Same order while mouse is captured |
| B | Order the fixed timber rack (two logs) | Same order while mouse is captured |
| Escape | Quit gracefully | Release mouse; press again to return to strategy |

The first visit spawns at a safe meadow position. Later visits retain the walking position and look direction. Strategy focus, orbit angle and zoom survive first-person visits. Walking stays on the meadow and slides around the house and limestone footprint. Losing window focus releases the mouse; returning to the window requires a click to resume navigation. Releasing the mouse or changing camera mode does not pause simulation. Close the window to quit from either mode.

## Harvest, store, build

Three trees contain **two log units each**. Nothing is harvested automatically. Click a crown in strategy view, click a numbered button, or press **1–3** to select one (the gold ground marker shows selection). Press **F** or **Harvest**. The worker walks to that tree, spends three simulation seconds felling it, and leaves a stump and two physical logs. Those same log entities are carried individually to the stockpile. Repeating the order cannot regenerate a tree or create more logs.

Press **B** or **Build rack** to order the single predefined timber rack beside the stockpile. It can be ordered before any wood exists: it waits. The worker picks up **stored** logs, carries each to the site and installs it. Each pickup removes that log from available storage. After two deliveries and three seconds of assembly, the crossbeam appears and the rack is complete. No resources are deducted merely for placing an order, and repeated orders never charge twice. One tree supplies this rack; harvesting all three leaves four log units in storage after construction.

Among timber jobs, the worker prioritizes supplying an ordered rack from available storage, then loose logs, then queued trees. Already-started work finishes before the next task is chosen. The HUD exposes the persistent worker ID, task, selected tree, time and material locations. Every unit is in exactly one state: **in tree, on ground, carried, stored or installed**; their sum stays six. Installed units are represented by the rack structure rather than duplicate log meshes. Camera switching does not spawn replacement entities or reset tasks, orders or stocks.

The timber chain has one worker, three fixed trees, one timber material and one fixed building site. The crop slice below shares that same worker and adds separate seed/grain accounting. It does not include free placement, advanced pathfinding, multiple workers, durability, further construction recipes or saved games. Routes are authored to stay clear of the existing house/limestone footprint; the small rack is an open prototype frame without its own collision. Restarting resets the session. Tree yield, rack cost, dimensions and work durations are technical/art choices, not historical measurements.

## Architecture

- `world.rs`: deterministic irregular shore, shallow sea bands and wave lines, shared grass patches with wind, pine group, timber farm silhouette, sunlight/sky/fog and shared shore/obstacle queries.
- `performance.rs`: low/high render settings, rolling CPU wall-frame statistics and a timed CSV benchmark. It never changes settlement resources or work.
- `camera.rs`: explicit camera-mode resource, independent strategy/first-person poses, collision-aware movement, focus and cursor policy. It never mutates simulation.
- `simulation.rs`: fixed-step worker tasks, finite tree/log states, construction state, field labour/transport and clock. One persistent worker handles timber, farming, sheep, food supply and barter transport.
- `farming.rs`: repeatable crop cycles, explicit seed reservation and cumulative grain accounting, plus persistent field/crop/sack visuals. It reads the shared settlement/worker state; it never advances work in rendering.
- `livestock.rs`: deterministic bounded grazing, finite fleece state, the three-sided wooden pen and persistent sheep/wool visuals. Work and transport still belong to the shared worker.
- `household.rs`: one household ID, its owned pantry/ration clock and a persistent visible storage rack by the house door. The worker is assigned to that household; supply movement belongs to the common worker task machine. Pure work progression has no camera/rendering/input dependency.
- `trade.rs`: a visitor with two finite salt pouches, an explicit two-wool price, physical handover and persistent visitor/salt visuals. Payment and cargo movement run in the common worker task machine.
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

Forty-four tests cover the complete material chain, waiting for missing timber, idempotent orders, finite yields, frame partitioning, authored traversable routes, preserved entity identities across camera changes, clickable-button activation, keyboard orders in both modes, input suppression on focus loss/mouse release, retained camera poses and existing collision. Checks also cover shore/pine collision, daylight, benchmark arguments and slow-frame percentiles. Crop tests cover seed use, premature/repeated orders, physical delivery, mixed timber/farming routes, frame partitions, focus guards and persistent field/cargo entities across camera changes. Sheep tests add deterministic grazing, held animals, fixed drop positions, finite fleeces, mixed workloads, pen collision and persistent animal/wool identities. Repeat-crop tests add three complete harvests, cumulative food/seed balance, live-stock overwrite guards, no recovery of consumed seed, and reused crop/sack entities across camera changes. Barter tests cover no-order/missing-payment waits, partial physical payment, finite offers, wool/salt conservation, single cargo, safe mixed-work routes, frame partitions and persistent visitor/cargo entities in both views. Native focus/cursor guards include J. Household tests cover owned-food consumption, waiting supply orders, physical delivery, ration timing/shortages independent of worker count, nonmember guards, mixed-work routes and persistent home/cargo identities.

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

A single tilled plot lies southeast of the house. Press **O** or **Sow field**. The existing worker collects the four initial seed units from the stockpile, carries their sack around the house via the southern approach, and spends four simulation seconds sowing. Each seed batch moves from **stored → carried → used** exactly once. Until that actual work completes, the field cannot grow.

The crop then grows for **30 simulation seconds**, visibly rising and changing from green to gold. It does not reap itself. Wait for **ripe** in the HUD and press **R** or **Reap crop**; premature orders are rejected with feedback. After walking back and four seconds of reaping, the worker leaves four finite grain sacks at the field and carries each separately to the stockpile. Each sack slot is either **unproduced, on the field, carried, stored, in the home, consumed or saved as seed**. The HUD shows those material counts separately from timber. No grain is credited merely for issuing an order, and repeating O/R cannot recreate seed or duplicate grain. Another sowing requires a real saved seed batch, as described below.

Among production tasks, timber delivery/building, loose logs and ordered trees retain their existing priority; grain hauling and field work follow. When V is ordered, saving one seed sack has first priority. When H is ordered and grain is already stored, the worker then supplies the home before accepting another production task. A current task completes before another is accepted. Farm travel uses the southern stockpile approach rather than crossing the house. Crop growth advances with the fixed simulation even when the worker is busy elsewhere or either camera is active. **T/Y change presentation sunlight only** and never speed up, pause or reset crops. Without V, only the initial sow/harvest cycle is available. With saved seed, O can order the next sowing after harvest; restarting resets the session.

This is one field and one worker, with illustrative grain geometry and arbitrary seed/yield units. The crop slice does not implement species-specific cultivation, seasons, multiple households, free field placement or saved games. The household slice below adds abstract food use from delivered grain. The separate sheep slice below adds only bounded grazing and fleece transport. The short growth/work durations are testing choices, not agricultural or historical measurements. Earlier milestone-3 benchmark results remain a record of that earlier scene; no new hardware performance claim is made for the crop slice.

Screenshots: [ripe plot in strategy](docs/screenshots/milestone4-ripe-strategy.png), [the same plot in first person](docs/screenshots/milestone4-ripe-first-person.png), and [completed delivery](docs/screenshots/milestone4-stored.png).

### Crop smoke test

1. Start a fresh session. Verify four seed units stored, no grain produced and an unplanted field. Wait: nothing should grow automatically.
2. Press O twice. Follow the same worker carry the seed sack, sow, and return through the southern approach. The initial stock moves to carried, then used; it must not be charged twice.
3. Press R before the crop is ripe: no harvest should start. Pause sunlight with T, advance its hour with Y and switch cameras with Tab. Crop growth must continue, with the same worker/entity IDs and stocks.
4. At ripe, press R twice. Observe reaping, ground sacks, one carried sack at a time, and four sacks eventually stored. Repeated O/R after completion must not add resources.
5. Repeat while also ordering tree felling and the rack. Timber must still produce six total units, install two and store four after all three trees are harvested; grain remains exactly four in storage. The automated test also checks every sampled route stays walkable.

## First sheep and wool chain

Two original stylized sheep occupy the small three-sided wooden pen northeast of the house. They move and lower their heads using deterministic **simulation time**, in a small bounded grazing pattern. The pen's south side is open; its visible side/back rails block walking. They are the same persistent animal entities in both camera modes. Their grazing motion does not yet consume a grass/forage inventory.

Press **C** or **Shear sheep** to order both unshorn animals clipped. Ordered animals hold their current positions until the existing worker reaches them; nothing is teleported to a separate scene. The worker goes via the southern stockpile approach and the east side of the house, enters the open pen, spends four simulation seconds clipping, then carries the two visible wool bundles individually to storage. Loose wool is collected before starting the next sheep. The shorn sheep has a smaller/darker coat and resumes its grazing motion; dropped bundles stay at their own fixed locations.

Each sheep supplies **two wool units once**, four total per session. Each unit is exactly one of **in fleece, on ground, carried, stored or owned by the trader**. C does not immediately award inventory, cannot interrupt a cargo delivery, and cannot reclip an already shorn animal or duplicate resources. There is still one worker: current tasks finish, timber/farm priorities remain ahead of sheep work, and simulation continues through Tab/T/Y. Restarting resets the session. No wool regrowth, feeding, health, breeding, lambs, milk, meat or spinning is implemented. The later barter step below adds one fixed wool exchange. These animals are generic interpretations, not a claim to reconstruct a particular historic or modern Gotland sheep breed; yields and work times are technical choices.

Screenshots: [initial flock](docs/screenshots/sheep-flock.png), [shared storage in first person](docs/screenshots/sheep-wool-first-person.png), and [completed wool delivery](docs/screenshots/sheep-wool-stored.png).

### Sheep smoke test

1. Start fresh. Inspect the two grazing sheep and the open south approach. The HUD should show zero shorn, four wool units in fleece, and none on ground/carried/stored. Wait: no wool should appear automatically.
2. Press C twice. Follow the same worker around the house into the pen; queued sheep hold still. After clipping, its coat changes and two physical bundles appear. The animal resumes grazing without moving those dropped bundles.
3. Press Tab during pickup/transport. Confirm the same animals, worker ID, tasks and material counts. Observe one carried bundle at a time; storage increases only after unloading.
4. After both animals are clipped and all trips finish, expect **shorn = 2, fleece = 0, ground = 0, carried = 0, stored = 4**. Press C again: stock must stay four, and neither coat regrows.
5. Repeat while ordering timber and crops. No worker clone or extra cargo should appear, the pen fences should block walking, and all chains must use the clear southern/eastern approach. Automated tests cover this combined workload, persistent visual IDs, fixed drop positions and resource accounting.

## First household and food supply

The existing worker belongs to **Home 1**, the coastal farm. Press **H** or **Supply home** to place a standing supply order. It can be ordered before planting; it waits for actual **stored grain**, and does not create or spend seed, timber, wool or grain merely for placing the order. Repeating H does not queue duplicate deliveries. Stored food supply has priority over starting a new production job, while the current task and cargo delivery always finish first.

The same worker collects one stored sack at a time, carries it along the clear southern approach to the house door, unloads into the visible wooden pantry rack, and returns to the shared store. Its location changes **shared store → carried → owned home pantry → consumed**. It is never available in both inventories, and can only be used by its recorded household ID. The same persistent grain entities represent these locations; used sacks are hidden rather than replaced or respawned.

The household's ration clock starts **only after the first real delivery**. It uses one pantry unit every **40 simulation seconds**, with the HUD showing remaining pantry units, total food used, missed ration intervals and time to the next use. If the pantry is empty at a due interval, the household reports **needs food**; stock never becomes negative, and no substitute material is eaten. Grain still on the field, in the shared store or carried by the worker cannot be consumed. Tab and T/Y do not reset or speed up this clock. The toolbar wraps into two rows so H remains clickable; the reserved UI area does not select trees through the buttons.

This is a foundation for household ownership and demand, not a complete family simulation. There is one home and one resident worker, with no added family NPCs, autonomous household economy, mortality, work penalties, cooking, milling, baking, trade or interiors. Grain units stand in for prepared food; direct grain-to-ration use and the 40-second period are technical abstractions, not historical diets or household sizes. Without V, the field supplies four total grain units and food eventually runs out. The seed-reservation step below enables more crop cycles; further households remain later work. Without H/V, the earlier crop/wool/timber smoke tests retain their original stock outcomes.

Screenshots: [waiting supply order](docs/screenshots/household-supply-order.png), [delivered pantry and used food](docs/screenshots/household-pantry.png), [same counters in first person](docs/screenshots/household-first-person.png), and [empty pantry after use](docs/screenshots/household-empty.png).

### Household smoke test

1. In a fresh session, press H twice. Verify zero pantry units and zero used food; no inventory should appear while the supply order waits.
2. Press O to sow. At **ripe**, press R. Follow sacks from the field to the shared store and then, one at a time, to the door-side pantry. Seed is not used as household food.
3. Switch Tab while food is carried. The worker ID/home membership, sack and both inventory counts must persist. Pause the presentation sun with T; crop and food clocks continue.
4. After the first delivery, wait forty simulation seconds. A delivered unit disappears and **food used** increases. Across the whole harvest, grain on ground + carried + shared store + owned pantry + consumed must total four once reaping is complete.
5. After all four units are used, watch **needs food** and missed intervals increase without negative inventory or extra sacks. H cannot refill the pantry without a real source. Current timber/crop/sheep jobs still work; mixed workloads and the return path to the shared store are covered by automated tests.

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

Future work: Gotland visual refinement, more construction recipes, independent households, seasons, more trade and Baltic shipping. Follow [AGENTS.md](AGENTS.md) and the scoped PR sequence in issue #1. Do not copy proprietary game assets or claim historical certainty without evidence.

A project license has not yet been selected. All rights reserved until the repository owner chooses one.

## Saved seed and repeat sowing

Press **V** or **Save seed** to place a standing seed-reservation order. Order it before the first harvested sack is supplied to the home. The worker waits for actual grain in the shared store, walks there and spends four simulation seconds preparing **one sack into one batch of four seed units**. The grain sack disappears only when this work completes; the stored seed sack becomes visible again. This conversion is an illustrative game ratio, not a historical yield or a claim that grain units and seed units have equal mass.

Seed reservation takes priority over new home deliveries and production jobs, but never interrupts existing work or recovers food already sent to the pantry. V is idempotent and stays active across harvests; it saves **at most one sack from each harvest**. H can supply the other three sacks. Reserved seed cannot be eaten by the household. No resources appear just because an order was issued.

Once the field is harvested and a seed batch is stored, press **O** again. The worker carries and uses that actual batch. Growth resets to zero and the crop grows through the same 30-second cycle; sowing/reaping remain manual orders. Existing grain in the field, shared store, cargo or pantry stays intact while the new crop grows. **R is rejected until the crop is ripe and all previous grain has been consumed or saved as seed.** Wait for those stocks to clear, then press R again. This bounded prototype reuses the same four sack entities only after the old contents are spent, rather than overwriting inventories or accumulating hidden entities. A ripe crop does not rot while it waits.

The HUD shows total **harvests**, total **seed batches saved**, and whether seed saving is ordered. Across every completed harvest, `4 × harvests = live grain (field + carried + shared store + pantry) + total food used + seed batches saved`. Each saved sack becomes one sowing batch; the separate seed counters show the current batch stored, carried or used. After three complete cycles with V/H, twelve produced sacks account for nine meals and three saved seed batches. Without V, the previous four-meal finite cycle still behaves as before. Ordering V after every sack has already reached the home cannot recover those sacks; restarting is the only reset.

This remains one field, one worker and one home. It adds no automatic sowing/harvesting, general stockpile capacities, seed-saving cancellation, seed-quality loss, seasons, crop varieties or historical agricultural calendar. A food shortage can still occur while work or the next harvest waits; there are no starvation/work penalties. The fixed four-sack limit and accelerated timings are prototype rules.

Screenshots: [saved seed and remaining grain](docs/screenshots/seed-reserved.png), [same counters in first person](docs/screenshots/seed-first-person.png), [ripe crop protecting old pantry stock](docs/screenshots/seed-protected-pantry.png), and [second harvest with another saved seed batch](docs/screenshots/seed-second-harvest.png).

### Repeat-crop smoke test

1. Start fresh, press V twice and H, then O. No grain or replacement seed should appear before the real crop work.
2. Wait for ripe, then R. Follow the first sack into shared storage: the worker prepares seed before home deliveries. Verify four stored seed units, **seed batches saved 1**, and at most three sacks available for food.
3. Press O once the saved seed is stored. Follow the seed sack back to the field. Growth starts at zero while old pantry/cargo stocks remain; pressing O again cannot charge or duplicate seed.
4. Switch Tab during carrying/growth. The same worker, seed, crop and pantry counters persist. T/Y still affect sunlight only.
5. If ripe while old grain remains, press R and verify the refusal leaves stocks intact. After the three old units have been eaten, press R again. **Harvests 2** must yield four new sacks, then one saved batch and three further meals, without resetting cumulative food use. Repeat O/R for a third crop.
6. In a separate fresh session, omit V: H/O/R still deliver four meals, and O must be refused after they run out because no seed was saved.

## Shore barter: wool for salt

A blue-clad visitor stands on the southwest shore with **two pale salt pouches** on a cloth spread. Press **J** or **Barter wool/salt** to order a standing exchange. Press **C** to shear and store wool if needed. Nothing moves merely because J is ordered: a new exchange waits for **two actual stored wool bundles** and one remaining salt offer.

The same worker picks up one wool bundle from storage, carries it to the shore and hands it over. The bundle changes **stored → carried → trader**, using its existing entity. A single delivered bundle does not release salt. After the second physical handover, one of the visitor's existing pouches changes **offered → paid**. The worker then picks up that pouch, carries it back and unloads it into shared storage: **paid → carried → stored**. A paid pouch is still at the shore until pickup; it is not counted as stored while in transit. The next exchange starts only after the previous salt delivery finishes. The HUD shows all these states, including wool now owned by the visitor.

J is idempotent and remains ordered until both offers are exhausted. The current four-wool flock can buy at most **two salt pouches**. Salt and traded wool do not respawn; repeated C/J cannot create further goods. Every wool bundle remains in exactly one of fleece/ground/carried/stored/trader, totaling four, and salt offered + paid + carried + stored always totals two. The number of fully paid pouches equals `floor(trader wool / 2)`. Without J, the previous sheep loop still leaves four wool bundles in storage and both salt pouches with the visitor.

Seed saving and home food supply keep priority over starting a new trade. With those demands satisfied, available barter precedes new production work. Current tasks and deliveries always finish, and the worker returns to the stockpile before accepting other jobs; the shore route stays clear of the house, rock and rack. Camera changes and sunlight controls do not alter orders, price, stocks or cargo.

This is one fixed offer and one stationary visitor, with no currency, negotiations, generic market, ship movement, replenishment or saved games. The visitor is separate from the one resident worker and does not join household labor. Salt is brought-in merchandise with an unspecified origin; there is no local extraction or production. It is stored only and does not yet preserve food, season meals or affect household consumption. The 2:1 exchange rate, pouch sizes, costume and cloth display are illustrative game choices, not documented Gotlandic prices or merchant practices around 900 CE. The visitor and display are small nonblocking props, like the current worker.

Screenshots: [waiting for wool](docs/screenshots/barter-waiting.png), [one completed exchange](docs/screenshots/barter-first-exchange.png), [salt in transit in first person](docs/screenshots/barter-salt-carried.png), and [both finite exchanges completed](docs/screenshots/barter-finished.png).

### Barter smoke test

1. Start fresh. Verify **offered 2 / paid 0 / carried 0 / stored 0**, and no wool at the visitor.
2. Click J twice before shearing. Nothing should be delivered and no salt should be granted. Press C and follow the existing wool bundles into shared storage.
3. When two bundles are stored, watch one carried to the visitor. After only one handover, both salt pouches must remain offered. The other wool bundle is then carried separately.
4. After the second handover, watch paid salt move to carried and then stored. Press Tab during transport: the worker ID, cargo and inventories persist. Switch back; the strategy pose is retained.
5. Let both exchanges finish. Verify **trader wool 4**, **salt stored 2**, all other wool/salt transport counts zero and both sheep shorn. The visitor's display is now empty of salt; four wool bundles remain there.
6. Repeat J/C. No additional wool or salt should appear. In a separate session without J, C should still leave four wool in shared storage. H/V/O/R and timber construction remain available; mixed-work/material invariants are covered by tests.
