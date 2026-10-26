# hafthiWorld

Rust + Bevy 0.20 settlement simulation inspired by Viking Age Gotland. The second scoped milestone from [issue #1](https://github.com/pdahlbeck/hafthiWorld/issues/1) adds a playable resource chain: **order tree felling → carry logs to storage → carry stored timber to a construction site → finish a timber rack**. Strategy and first person observe the same running ECS world. Milestone 3 adds an original procedural coastal visual pass and portable performance instrumentation. The following small slices add a crop/seed loop, a two-sheep fleece chain and a first owned household pantry and a finite shore barter offer, using the same worker and storage. The founder, optional coastal helper, three finite trees and material-backed timber rack remain the same shared simulation in both views. A fresh session is solo; **N** invites the second resident and assigns coastal work, as described below. Earlier smoke tests assume N is omitted.

## Forestry and timber priorities

This version adds **two steps together**: replanting/regrowth and timber planning.
Select a felled plot with **1/2/3**, then press **X**. The founder must plant it;
the sapling takes two nonwinter growing seasons to mature. **F** then harvests
new timber without restoring old burned or installed wood. **M** cycles Balanced,
Building first and Winter reserves, and shows each plot's progress and timber needs.
See the [controls, allocation rules and limits](docs/forestry-and-timber.md).

## Second farmstead

Press **G** to order the marked farmstead west of the original house. It costs
**two stored timber units**, physically delivered by the founder, then 12 seconds
of assembly work. When it is complete, press **G** again to invite its resident.
Together with the optional **N** coastal helper, the settlement can now have
three residents in two households. **H2** in the inspector, or clicking the new
house, shows its own food and firewood. The new resident fetches real shared
supplies for household 2; H/K keep controlling the original household.

The initial six timber units are shared with the rack and winter fuel. If they
are spent, replant the stumps and harvest another crop before building. Nothing
refunds burned wood or duplicates goods for the new house.
See [second-household controls, validation and limits](docs/second-household.md).

## Current resident review

The runtime people now use MIT-licensed Rocketbox head/hand anatomy and skin
maps with project-authored costume, an 80-joint skeleton and four native Bevy
animation clips. Each worker blends from its own saved task/clock; contact IK
corrects stance feet and reaches towards actual cargo. Space and F5/F9 preserve
paused and mid-transition poses; older saves load with default blend state.
The anatomical neck extends inside the collar and shares its upper-chest deformation.
**Tab** enters facing the founder in a fresh session; **F4** hides the HUD.
See [actual captures, provenance and remaining limits](docs/textured-residents.md).
All people still share one face; full skin SSS, new behavior AI, landscape
quality and the finished-game visual target remain unresolved.

## Current graphics review

High now uses Bevy's physical atmosphere, sky-driven environment illumination,
photographic exposure, ACES and contact shadows. Original textured reed roofing
and alpha-cut pine branches replace the flat roof and triangular needle fans;
terrain has broad material variation rather than a uniformly speckled surface.
Use `cargo run --release --locked -- --quality high`; **F4** hides the HUD.
Twilight keeps daylight exposure until the sun is below the horizon, with night
gain limited to five stops to prevent the reported sunset whiteout. **T** pauses
the sun and **Y** advances three hours for checking the cycle.
Low keeps the lighter lighting pipeline. See [renderer review](docs/physical-renderer.md)
for architecture, controls and remaining visual limitations. These changes do
not establish the supplied reference image's finished-game quality.

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
| F2 | Toggle full settlement statistics | Same control, also with mouse released |
| I | Cycle inspection: home, residents, boat, closed | Same control, also with mouse released |
| F3 | Hide/show performance panel | Same panel |
| Space | Pause/resume the settlement | Same control, also with mouse released |
| F5 / F9 | Save / load session | Same controls, also with mouse released |
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
| H | Order stored grain or salted fish supplied to the home | Same order while mouse is captured |
| V | Order one stored sack per harvest saved as seed | Same order while mouse is captured |
| J | Order two stored wool per salt pouch at the shore visitor | Same order while mouse is captured |
| K | Order stored timber delivered as household firewood | Same order while mouse is captured |
| P | Order the finite four-fish coastal catch | Same order while mouse is captured |
| L | Order two raw fish salted with one real salt pouch | Same order while mouse is captured |
| N | Invite one coastal worker to the existing household | Same order while mouse is captured |
| G | Order second farmstead; once built, invite its resident | Same order while mouse is captured |
| X | Order replanting at the selected felled plot | Same order while mouse is captured |
| M | Cycle timber priority and open its inspector | Same order while mouse is captured |
| U | Request collection of the shore visitor's four paid wool bundles | Same order while mouse is captured |
| B | Order the fixed timber rack (two logs) | Same order while mouse is captured |
| Escape | Quit gracefully | Release mouse; press again to return to strategy |

The default HUD is compact. Press **F2** for the full counters used in the older smoke checks below. Clicking a resident, the house or a visible boat in strategy opens its inspector; tree selection and production keys retain their existing meaning.

The first visit spawns at a safe meadow position. Later visits retain the walking position and look direction. Strategy focus, orbit angle and zoom survive first-person visits. Walking stays on the meadow and slides around the house and limestone footprint. Losing window focus releases the mouse; returning to the window requires a click to resume navigation. Releasing the mouse or changing camera mode does not pause simulation. Close the window to quit from either mode.

## Harvest, store, build

Three trees contain **two log units each**. Nothing is harvested automatically. Click a crown in strategy view, click a numbered button, or press **1–3** to select one (the gold ground marker shows selection). Press **F** or **Harvest**. The worker walks to that tree, spends three simulation seconds felling it, and leaves a stump and two physical logs. Those same log entities are carried individually to the stockpile. Repeating the order cannot regenerate a tree or create more logs.

Press **B** or **Build rack** to order the single predefined timber rack beside the stockpile. It can be ordered before any wood exists: it waits. The worker picks up **stored** logs, carries each to the site and installs it. Each pickup removes that log from available storage. After two deliveries and three seconds of assembly, the crossbeam appears and the rack is complete. No resources are deducted merely for placing an order, and repeated orders never charge twice. One tree supplies this rack; harvesting all three leaves four log units in storage after construction.

Among timber jobs, the worker prioritizes supplying an ordered rack from available storage, then loose logs, then queued trees. Already-started work finishes before the next task is chosen. The HUD exposes the persistent worker ID, task, selected tree, time and material locations. Every unit is in exactly one state: **in tree, on ground, carried, stored, installed, owned home fuel or burned**; their sum stays six. Installed units are represented by the rack structure rather than duplicate log meshes. Camera switching does not spawn replacement entities or reset tasks, orders or stocks.

The timber chain has one founder by default, three fixed trees, one timber material and one fixed building site. The crop slice below shares that same worker and adds separate seed/grain accounting. It does not include free placement, advanced pathfinding, arbitrary worker assignment, durability, further construction recipes or multiple save slots. Routes are authored to stay clear of the existing house/limestone footprint; the small rack is an open prototype frame without its own collision. Restarting starts a fresh session; F9 restores the explicit F5 save. Tree yield, rack cost, dimensions and work durations are technical/art choices, not historical measurements.

## Architecture

- `art.rs`: shared rounded timber edges, shaped clothing/anatomy, continuous boat shell/deck/rails, bowed thatch and irregular matte foliage. These are rendering meshes, not simulation/collision geometry.
- `world.rs`: grass/pines/farmstead, daylight/sky/fog and shared shore/obstacle queries.
- `coast.rs`: continuous shore/meadow/shelf meshes, layered limestone and tiled PBR surfaces.
- `resident.rs` / `art_assets.rs`: first skinned resident study, saved-clock bone poses and embedded runtime art.
- `performance.rs`: low/high render settings, rolling CPU wall-frame statistics and a timed CSV benchmark. It never changes settlement resources or work.
- `camera.rs`: explicit camera-mode resource, independent strategy/first-person poses, collision-aware movement, focus and cursor policy. It never mutates simulation.
- `calendar.rs`: an accelerated four-season world calendar, sowing windows and exact nonwinter crop-growth time across long updates/year boundaries. It has no camera or presentation-clock dependency.
- `simulation.rs`: fixed-step worker tasks, finite tree/log states, construction state, field labour/transport and clock. The founder handles all jobs by default. N adds one coastal specialist; the founder then handles timber, crops, wool, barter, grain and firewood while the helper handles fish. Coastal jobs retain one owner through their full transport cycle; all world clocks advance once per tick.
- `farming.rs`: repeatable crop cycles, explicit seed reservation and cumulative grain accounting, plus persistent field/crop/sack visuals. It reads the shared settlement/worker state; it never advances work in rendering.
- `livestock.rs`: deterministic bounded grazing, finite fleece state, the three-sided wooden pen and persistent sheep/wool visuals. Work and transport still belong to the shared worker.
- `fishing.rs`: finite raw/preserved fish state, salting recipe and persistent net-frame/table/fish visuals.
- `coastal_work.rs`: a child module of simulation implementing physical catch, ingredient, output and home-food transport using the same worker and one-cargo rule.
- `hearth.rs`: owned firewood demand that advances only during calendar winter, a three-log home supply target and a persistent door-side wood rack. It consumes only actually delivered owned logs; transport belongs to the worker.
- `household.rs`: one household ID, its owned pantry and common grain/fish ration clock and a persistent visible storage rack by the house door. The worker is assigned to that household; supply movement belongs to the common worker task machine. Pure work progression has no camera/rendering/input dependency.
- `trade.rs`: a visitor with two finite salt pouches, an explicit two-wool price, physical handover and persistent visitor/salt visuals. Payment and cargo movement run in the common worker task machine.
- `inspection.rs`: read-only live resident/home/boat panels, conservative foreground-aware strategy picking, native inspection navigation and compact/detail HUD state. UI choices are not part of the saved economy.
- `orders.rs`: selection, tree ray picking, keyboard/native UI button orders and focus/mode guards. It records intent; it never moves workers or grants material.
- `presentation.rs`: persistent worker/tree/log entities, axe motion, selection marker, construction parts, native Bevy 0.20 buttons and HUD reading shared state in `Update`. It does not advance the economy.
- `shipping.rs`: one finite outbound wool-collection voyage; waiting/arrival/loading/departure and original boat visuals read the shared settlement.
- `session.rs`: versioned full simulation snapshots, checked cargo/roster, atomic local save replacement, restore and simulation pause. Camera poses stay live.
- `main.rs`: plugin composition and window setup.

`FixedUpdate` advances material flow regardless of camera mode. `Update` presents the current state. Environment geometry and clothing are authored for this project; the residents’ head and hands adapt a documented CC0 anatomical base (see `assets/README.md`). Historical layouts, costumes and logistics timing are artistic/technical interpretations, not archaeological claims. Several small interactive props remain simplified prototypes.

## Validation

```sh
cargo fmt --check
cargo check --locked
cargo test --locked
cargo clippy --locked -- -D warnings
cargo build --locked
```

Eighty-two tests cover the complete material chain, waiting for missing timber, idempotent orders, finite yields, frame partitioning, authored traversable routes, preserved entity identities across camera changes, clickable-button activation, keyboard orders in both modes, input suppression on focus loss/mouse release, retained camera poses and existing collision. Checks also cover shore/pine collision, daylight, benchmark arguments and slow-frame percentiles. Crop tests cover seed use, premature/repeated orders, physical delivery, mixed timber/farming routes, frame partitions, focus guards and persistent field/cargo entities across camera changes. Sheep tests add deterministic grazing, held animals, fixed drop positions, finite fleeces, mixed workloads, pen collision and persistent animal/wool identities. Repeat-crop tests add three complete harvests, cumulative food/seed balance, live-stock overwrite guards, no recovery of consumed seed, and reused crop/sack entities across camera changes. Barter tests cover no-order/missing-payment waits, partial physical payment, finite offers, wool/salt conservation, single cargo, safe mixed-work routes, frame partitions and persistent visitor/cargo entities in both views. Five calendar/palette tests add exact season/year boundaries, partitioned nonwinter growth, winter crop/seed preservation, queued spring sowing, worker-independent runtime clocks, camera/sun independence and reused seasonal materials. Six firewood tests add owned-log consumption, winter-only/frame-partitioned demand, actual capped deliveries, no-order/nonmember guards, shared timber/building/crop/trade workloads and persistent log identities through two winters. The no-worker calendar test also verifies winter demand continues without a worker. Nine coastal-food tests add finite/duplicate catch guards, real ingredient requirements, staged salt/raw/output transport, unavailable-source waits, one shared grain/fish ration clock, member-only supply, frame partitions, mixed material/season/winter workloads, and persistent fish/salt/worker entities in both views. Five crew tests add guarded one-time recruitment, coastal-only assignment, handoff during catch/ingredient transport/salting/home transport, unchanged world/household/winter clocks, actual-source concurrent multi-chain work, carrier-specific projection for all five cargo kinds and persistent actors/cargo across camera changes. Native focus/cursor guards include J/K/P/L/N. Household tests cover owned-food consumption, waiting supply orders, physical delivery, ration timing/shortages independent of worker count, nonmember guards, mixed-work routes and persistent home/cargo identities.

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

Among production tasks, timber delivery/building, loose logs and ordered trees retain their existing priority; grain hauling and field work follow. An ordered home firewood top-up has first priority when timber is stored and the home stack has room. When V is ordered, saving one seed sack follows. When H is ordered and grain is already stored, the worker then supplies the home before accepting another production task. A current task completes before another is accepted. Farm travel uses the southern stockpile approach rather than crossing the house. Crop growth advances with the fixed simulation even when the worker is busy elsewhere or either camera is active. **T/Y change presentation sunlight only** and never speed up, pause or reset crops. Without V, only the initial sow/harvest cycle is available. With saved seed, O can order the next sowing after harvest; restarting resets the session.

This is one field and one founder by default, with illustrative grain geometry and arbitrary seed/yield units. The crop slice does not implement species-specific cultivation, multiple households, free field placement or saved games. The household slice below adds abstract food use from delivered grain. The separate sheep slice below adds only bounded grazing and fleece transport. The short growth/work durations are testing choices, not agricultural or historical measurements. Earlier milestone-3 benchmark results remain a record of that earlier scene; no new hardware performance claim is made for the crop slice.

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

Each sheep supplies **two wool units once**, four total per session. Each unit is exactly one of **in fleece, on ground, carried, stored or owned by the trader**. C does not immediately award inventory, cannot interrupt a cargo delivery, and cannot reclip an already shorn animal or duplicate resources. By default there is one worker: current tasks finish, timber/farm priorities remain ahead of sheep work, and simulation continues through Tab/T/Y. Restarting starts a fresh session; F9 restores the explicit F5 save. No wool regrowth, feeding, health, breeding, lambs, milk, meat or spinning is implemented. The later barter step below adds one fixed wool exchange. These animals are generic interpretations, not a claim to reconstruct a particular historic or modern Gotland sheep breed; yields and work times are technical choices.

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

The household's common ration clock starts **only after the first real grain or salted-fish delivery**. It uses one pantry unit every **40 simulation seconds**, with the HUD showing remaining pantry units, total food used, missed ration intervals and time to the next use. If the pantry is empty at a due interval, the household reports **needs food**; stock never becomes negative, and no substitute material is eaten. Grain still on the field, in the shared store or carried by the worker cannot be consumed. Tab and T/Y do not reset or speed up this clock. The toolbar wraps into two rows so H remains clickable; the reserved UI area does not select trees through the buttons.

This is a foundation for household ownership and demand, not a complete family simulation. There is one home, a founder and an optional coastal helper, with no full family population model or autonomous household economy, mortality, work penalties, cooking, milling, baking, trade or interiors. Grain units stand in for prepared food; direct grain-to-ration use and the 40-second period are technical abstractions, not historical diets or household sizes. Without V, the field supplies four total grain units and food eventually runs out. The seed-reservation step below enables more crop cycles; further households remain later work. Without H/V, the earlier crop/wool/timber smoke tests retain their original stock outcomes.

Screenshots: [waiting supply order](docs/screenshots/household-supply-order.png), [delivered pantry and used food](docs/screenshots/household-pantry.png), [same counters in first person](docs/screenshots/household-first-person.png), and [empty pantry after use](docs/screenshots/household-empty.png).

### Household smoke test

1. In a fresh session, press H twice. Verify zero pantry units and zero used food; no inventory should appear while the supply order waits.
2. Press O to sow. At **ripe**, press R. Follow sacks from the field to the shared store and then, one at a time, to the door-side pantry. Seed is not used as household food.
3. Switch Tab while food is carried. The worker ID/home membership, sack and both inventory counts must persist. Pause the presentation sun with T; crop and food clocks continue.
4. After the first delivery, wait forty simulation seconds. A delivered unit disappears and **food used** increases. Across the whole harvest, grain on ground + carried + shared store + owned pantry + consumed must total four once reaping is complete.
5. After all four units are used, watch **needs food** and missed intervals increase without negative inventory or extra sacks. H cannot refill the pantry without a real source. Current timber/crop/sheep jobs still work; mixed workloads and the return path to the shared store are covered by automated tests.

## Coastal visual pass (milestone 3)

The small clearing now has a faceted limestone shore, opaque Baltic color bands and broken animated wave lines, wind-blown batched grass, a background pine group, a timber building with boards/door/gables/pitched roof, distance haze and warm late-afternoon sunlight. Terrain stays flat at the worker paths; camera walking follows the same irregular shore mesh with a dry-land setback and trunk clearance. House/limestone collision and all authored logistics routes remain in place. Decorative vegetation does not create harvestable resources.

**T** pauses the presentation sun; **Y** advances three hours so dawn, dusk and night can be inspected without waiting. New sessions begin at 10:00. One full visual day lasts 60 minutes of unclamped game time, giving 20 minutes before sunset. Night uses an artistic cool directional light and ambient floor matched to the camera exposure so paths, people and the coast remain visible. This clock changes lighting/sky/haze only; it does not change the economy or reset when switching cameras. See [visual references and interpretation limits](docs/visual-references.md), [strategy screenshot](docs/screenshots/milestone3-strategy.png) and [first-person screenshot](docs/screenshots/milestone3-first-person.png). The world remains a small stylized diorama; this does not implement the eventual 2×2 km map, weather. The separate gameplay calendar below adds a first seasonal simulation.

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

Future work: Gotland visual refinement, more construction recipes, independent households, winter food/fodder demands, more trade and Baltic shipping. Follow [AGENTS.md](AGENTS.md) and the scoped PR sequence in issue #1. Do not copy proprietary game assets or claim historical certainty without evidence.

A project license has not yet been selected. All rights reserved until the repository owner chooses one.

## Saved seed and repeat sowing

Press **V** or **Save seed** to place a standing seed-reservation order. Order it before the first harvested sack is supplied to the home. The worker waits for actual grain in the shared store, walks there and spends four simulation seconds preparing **one sack into one batch of four seed units**. The grain sack disappears only when this work completes; the stored seed sack becomes visible again. This conversion is an illustrative game ratio, not a historical yield or a claim that grain units and seed units have equal mass.

After an eligible firewood top-up, seed reservation takes priority over new food deliveries and production jobs, but never interrupts existing work or recovers food already sent to the pantry. V is idempotent and stays active across harvests; it saves **at most one sack from each harvest**. H can supply the other three sacks. Reserved seed cannot be eaten by the household. No resources appear just because an order was issued.

Once the field is harvested and a seed batch is stored, press **O** again. The worker carries and uses that actual batch. Growth resets to zero and the crop grows through the same 30-second cycle; sowing/reaping remain manual orders. Existing grain in the field, shared store, cargo or pantry stays intact while the new crop grows. **R is rejected until the crop is ripe and all previous grain has been consumed or saved as seed.** Wait for those stocks to clear, then press R again. This bounded prototype reuses the same four sack entities only after the old contents are spent, rather than overwriting inventories or accumulating hidden entities. A ripe crop does not rot while it waits.

The HUD shows total **harvests**, total **seed batches saved**, and whether seed saving is ordered. Across every completed harvest, `4 × harvests = live grain (field + carried + shared store + pantry) + total food used − fish meals used + seed batches saved`. Each saved sack becomes one sowing batch; the separate seed counters show the current batch stored, carried or used. After three complete cycles with V/H, twelve produced sacks account for nine meals and three saved seed batches. Without V, the previous four-meal finite cycle still behaves as before. Ordering V after every sack has already reached the home cannot recover those sacks; restarting is the only reset.

This remains one field and one home, with a founder and optional coastal helper. It adds no automatic sowing/harvesting, general stockpile capacities, seed-saving cancellation, seed-quality loss, crop varieties or historical agricultural calendar. A food shortage can still occur while work or the next harvest waits; there are no starvation/work penalties. The fixed four-sack limit and accelerated timings are prototype rules.

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

J is idempotent and remains ordered until both offers are exhausted. The current four-wool flock can buy at most **two salt pouches**. Salt and traded wool do not respawn; repeated C/J cannot create further goods. Every wool bundle remains in exactly one of fleece/ground/carried/stored/trader, totaling four, and salt offered + paid + carried + stored + at salting + consumed always totals two. Without L, the last two counts remain zero. The number of fully paid pouches equals `floor(trader wool / 2)`. Without J, the previous sheep loop still leaves four wool bundles in storage and both salt pouches with the visitor.

Firewood top-ups, seed saving and home food supply keep priority over starting a new trade. With those demands satisfied, available barter precedes new production work. Current tasks and deliveries always finish, and the worker returns to the stockpile before accepting other jobs; the shore route stays clear of the house, rock and rack. Camera changes and sunlight controls do not alter orders, price, stocks or cargo.

This is one fixed offer and one stationary visitor, with no currency, negotiations, generic market, ship movement, replenishment or saved games. The visitor is separate from the resident workforce and does not join household labor. Salt is brought-in merchandise with an unspecified origin; there is no local extraction or production. The coastal-food step below can consume stored salt to process raw fish into household rations. It does not season grain meals or change their ration timing. The 2:1 exchange rate, pouch sizes, costume and cloth display are illustrative game choices, not documented Gotlandic prices or merchant practices around 900 CE. The visitor and display are small nonblocking props, like the current worker.

Screenshots: [waiting for wool](docs/screenshots/barter-waiting.png), [one completed exchange](docs/screenshots/barter-first-exchange.png), [salt in transit in first person](docs/screenshots/barter-salt-carried.png), and [both finite exchanges completed](docs/screenshots/barter-finished.png).

### Barter smoke test

1. Start fresh. Verify **offered 2 / paid 0 / carried 0 / stored 0**, and no wool at the visitor.
2. Click J twice before shearing. Nothing should be delivered and no salt should be granted. Press C and follow the existing wool bundles into shared storage.
3. When two bundles are stored, watch one carried to the visitor. After only one handover, both salt pouches must remain offered. The other wool bundle is then carried separately.
4. After the second handover, watch paid salt move to carried and then stored. Press Tab during transport: the worker ID, cargo and inventories persist. Switch back; the strategy pose is retained.
5. Let both exchanges finish. Verify **trader wool 4**, **salt stored 2**, all other wool/salt transport counts zero and both sheep shorn. The visitor's display is now empty of salt; four wool bundles remain there.
6. Repeat J/C. No additional wool or salt should appear. In a separate session without J, C should still leave four wool in shared storage. H/V/O/R and timber construction remain available; mixed-work/material invariants are covered by tests.


## First seasonal calendar

The HUD now shows **year, season, seconds until the next season and the sowing window**. A new session starts in **year 1, spring**. Each season lasts **120 simulation seconds**, in spring → summer → autumn → winter order; an accelerated year lasts 480 seconds. This calendar advances once per world fixed tick even with no worker. Tab, released mouse capture, camera motion and T/Y do not pause, skip or reset it. T/Y still operate only the separate presentation sun; this prototype does not synchronize a solar year with the visual day.

**New sowing work starts only in spring or summer.** O can still queue a valid sow order in autumn/winter: seeds stay stored and the worker can take other jobs while waiting. When spring opens, the order becomes eligible under the normal fuel/seed/food/trade/timber priorities. Already started sowing/seed transport finishes even if autumn begins during the trip; it is never cancelled halfway and seed is never refunded or duplicated.

A planted crop grows at its existing rate in spring, summer and autumn. **Winter pauses growth**, preserving its phase, progress and planted seed. At the next spring, growth resumes from that progress. Long updates integrate only the actual nonwinter time, including updates across multiple seasons/years. A ripe crop can be reaped in any season if its previous grain slots are resolved; it does not rot. Food consumption, timber, wool, trade and cargo transport keep running through winter, with all existing inventory/ownership rules intact.

Meadow and grass materials shift from spring green through a drier summer and warm autumn to muted winter tones. The existing meshes and material handles are reused; no landscape/cargo/inhabitant entities are respawned at a season change. Pines remain evergreen. This is a stylized palette, with no snow, ice, changing day length, weather, seasonal mortality, fodder, sheep regrowth or crop failure yet. Spring/summer sowing, overwinter survival and the accelerated 120-second seasons are game rules for a generic crop, not a reconstruction of Gotlandic agronomy around 900 CE.

Screenshots: [spring calendar](docs/screenshots/calendar-spring.png), [same clock in first person after a sun change](docs/screenshots/calendar-first-person.png), [autumn palette](docs/screenshots/calendar-autumn.png), and [queued autumn sowing while timber work proceeds](docs/screenshots/calendar-queued-sowing.png).

### Seasonal smoke test

1. Start fresh. Verify year 1 spring, sowing open and approximately 120 seconds remaining. Press T to freeze the sun and O to sow. The calendar and crop still progress.
2. Switch Tab and return; time, worker and inventory persist. At simulation 120 seconds, summer begins and the meadow/grass palette changes. Y shifts the sun without changing the season.
3. In a fresh session, leave the field unplanted until autumn (240 seconds). Press O twice: the sow order waits, all four seed units remain stored and no crop grows. Other ordered timber/sheep/trade jobs can proceed.
4. At winter (360 seconds), sowing remains queued. At year 2 spring (480 seconds), the worker can collect that same seed batch and sow without a second order.
5. For an existing growing crop, winter holds its progress and spring resumes it; the automated tests check this boundary exactly. Ripe harvests and home rations remain possible in winter, with no season-driven inventory gifts or reset.


## Winter firewood

Press **K** or **Supply firewood** to place an idempotent standing supply order for Home 1. It waits for **stored timber** and keeps at most **three owned logs** in the new wood rack beside the door. It does not automatically fell a tree: use **1–3 / F** for the existing harvesting jobs. The worker picks up one stored log, carries it along the southern approach, unloads at the house, and returns to shared storage before choosing another task. Its location changes **stored → carried → home fuel**, using the same existing log entity. A nonmember cannot deliver fuel to this home.

The household uses **one delivered owned log every 40 calendar-winter seconds**. Demand starts at winter, even if no wood has been delivered; it does not wait indefinitely for a first delivery. Spring/summer/autumn pause the fuel clock completely. Long updates use the calendar's actual winter time across year boundaries. Food and work clocks keep their existing rules; Tab/T/Y and worker count do not affect fuel timing. Shared stored logs, carried logs, installed building material and another home's logs cannot be burned.

At a due interval with fuel, one owned log becomes **burned** and its mesh hides while its entity ID persists. At an empty due interval, **missed** increments without negative inventory or eating another resource. The HUD reports home fuel, target, burned units, missed intervals and the time until the next use in *winter seconds*. Outside winter it reads **winter demand paused**; during winter an empty stack reads **needs firewood**. This is demand/inventory feedback, with no simulated room temperature, actual fireplace/fire effect, health damage or work penalty.

When K is ordered and there is room, a firewood top-up precedes new seed, food, trade and production jobs; current work/deliveries finish first. The cap leaves surplus in shared storage until winter creates room. Fuel competes with building materials: timber already delivered home is unavailable to the rack, and installed timber cannot be reclaimed as fuel. For a fresh all-three-tree run with B/K, the six units end up as **two installed, three home fuel and one shared surplus** before winter. During winter the last surplus can refill the home stack. Across two winters, four logs can burn and two fuel intervals are missed; the installed two remain intact. Without K, earlier timber smoke tests still leave four stored logs after construction, and winter shortages alone do not change those stocks.

Tree yields remain finite: there is no regrowth, extra fuel source, splitting/drying step, charcoal production, free delivery, cancellation or transfer back from the home pile. Raw timber units stand in for prepared firewood; the 40-second rate, three-log target and visible rack are technical/artistic choices, not documented medieval fuel quantities, thermal behavior or Gotlandic hearth architecture. The rack is a small nonblocking prop; no building/interior/fire safety reconstruction is claimed.

Screenshots: [waiting firewood order](docs/screenshots/fuel-waiting.png), [three home logs plus two installed and one shared](docs/screenshots/fuel-stocked.png), [same counts in first person](docs/screenshots/fuel-first-person.png), and [winter burning and refill](docs/screenshots/fuel-winter.png).

### Winter-fuel smoke test

1. Start fresh. Click K twice before felling: the order waits, home fuel stays zero and no wood appears. All six units remain in trees.
2. Order B and all three trees with 1/F, 2/F, 3/F. Follow real logs into shared storage, then one at a time to the home rack. The carried log is not owned fuel until unloading.
3. Switch Tab during delivery. The same worker and log persist. During spring/summer/autumn the **burned** counter stays zero. Before winter and after work finishes, verify installed 2 + home fuel 3 + shared 1 = 6.
4. At winter (simulation 360 seconds), the countdown starts. At 400 seconds, a delivered log disappears and burned increases to one; the standing order can replenish from the shared surplus. After the first winter, burned 3 + home fuel 1 + installed 2 = 6.
5. During year 2 winter, only that last log can burn. The remaining two due intervals report shortages. No installed timber, grain, seed, wool or salt is consumed as fuel. The automated mixed-chain test checks this full two-winter result and every sampled material/route balance.


## Coastal food: catch → salt → supply home

This package links three playable steps using the existing resident worker, finite traded salt and common household pantry. **P / Fish** places a standing order for this session's **four-fish catch allowance**. The worker walks to the net-frame marker on the southwest shore, spends eight simulation seconds catching one fish, picks it up, carries it back and stores it. Every fish has one persistent entity: uncaught fish are hidden, then the actual caught fish is visible at the shore, in transit and at storage. Repeating P cannot reset the allowance or duplicate fish; no fishing population or regrowth model is claimed.

**L / Salt fish** places a standing processing order. It waits for **two stored raw fish and one actually stored salt pouch**. Salt still offered/paid at the visitor or carried from a trade is unavailable. Obtain salt using **C** for wool and **J** for the existing barter chain. Fishing and salting can be ordered before salt exists; they wait rather than granting ingredients. With no L, caught fish remain raw in storage.

The worker carries the salt pouch to the new table near shared storage, then brings each raw fish separately. Ingredients remain visible at the table during **six seconds of salting work**. Only after that work completes does the salt change to **consumed** and the two fish become salted output. Each output fish is then carried separately back to shared storage. Salt is never simultaneously in shared storage, in cargo and at the table, and one worker never carries multiple ingredients at once. A batch finishes transport before a fresh unrelated job is chosen. Raw fish are blue-gray; salted fish have warm brown bodies.

**H / Supply home** now also transports stored salted fish, one at a time, to the owned pantry. Fish at the salting table or in transit cannot be consumed; raw fish are not a ration in this prototype. The household uses **one shared 40-second ration clock** for both grain and fish, starting on its first actual delivery of either. Each due interval consumes one owned unit, preferring grain when both are available; fish do not create an extra ration timer. The HUD shows total food used and the fish subset. `grain meals used = total food used − fish meals used`, so the previous grain/seed balance remains auditable. A fish-only household can start and eat without planting a field.

Four raw fish plus the two finite traded salt pouches yield **two processed batches and at most four fish meals**. Fish across uncaught/raw/processing/preserved/carried/home/eaten stages total four. Processed fish (table + preserved cargo + stored + home + eaten) equal `2 × completed salting batches`; consumed salt equals completed batches, at most two. Used fish and salt meshes hide but retain their entity IDs. Repeated P/L/H, Tab or T/Y cannot replenish fish, salt or eaten food.

The HUD's **raw** count combines fresh fish at the shore and raw shared storage; **processing** means raw fish at the table; **preserved** combines salted output at the table and shared storage. The distinct **carried**, **home** and **eaten** counts cover later locations. These buckets sum to four with uncaught. Salt now also shows **table** and **used**. Ingredient/output transport and its exact location are visible in the worker's task line.

In the default solo mode, task priorities are: eligible home firewood top-up, seed saving, grain home supply, salted-fish home supply, ready salting batch, available barter, timber work, crop work, sheep work, then new fishing trips. Current tasks always finish first; a salting batch delivers all its output before switching jobs. Supply requires membership in the destination home. Routes use clear southern approaches and an offset work position beside the table. The frame/table are small nonblocking visual props, like the existing racks. Fishing/processing continue through winter; the existing crop and firewood season rules are unchanged.

This is one authored shore worksite, four finite fish and one two-fish recipe. It adds no boats, specific species, catch seasons, ecosystem, spoilage, drying/cooking, food quality/temperature, disease or safety simulation. Salted output is the only implemented fish ration; this does **not** imply that fresh fish was historically inedible or that salting takes six seconds. Recipe ratios, gear, colors, catch limit and durations are original technical/art abstractions, not claims about Viking Age Gotland's fishing methods, diets or salt use. Imported salt retains an unspecified origin. Restarting starts a fresh session; F9 restores the explicit F5 save.

Screenshots: [actual catching](docs/screenshots/coastal-catch.png), [four raw fish waiting for salt](docs/screenshots/coastal-raw-stock.png), [table ingredients during salting](docs/screenshots/coastal-salting.png), [first home delivery](docs/screenshots/coastal-home.png), [same pantry in first person](docs/screenshots/coastal-first-person.png), and [four meals completed](docs/screenshots/coastal-finished.png).

### Coastal-food smoke test

1. Start fresh. Click P and L twice, then H. The worker catches four fish individually, but with no salt there must be no salted output, consumed salt, pantry food or used meals. The processing order waits.
2. Press C and J. Follow the wool to the visitor and the salt back to shared storage; offered/paid/carried salt alone must not start a batch.
3. Once one pouch is stored and two raw fish exist, follow salt and then both fish separately to the table. Before the six-second job finishes, the pouch is still at the table and no preserved output exists. On completion, used salt increases by one and exactly two fish change stage.
4. Follow both output fish back to storage, then H deliveries to the home. Switch Tab during ingredient or meal transport; entities, counts, membership and the one ration clock persist.
5. Let both batches and home meals finish. Verify fish eaten 4, salt used 2, trader wool 4, no live raw/preserved/cargo/home fish, and total food used 4 in this fish-only session. Repeated P/L/H cannot create more goods. Ration shortages may grow once food is exhausted.
6. In a separate session without H, preserved fish stay stored; without L, fish stay raw. With grain and fish delivered, only one owned unit is eaten per 40 seconds, grain first. Automated tests additionally mix two crops, timber construction, winter firewood and the complete real-source fish chain.


## Two residents and coastal work allocation

Press **N** or **Invite helper** once to add a blue-clad coastal worker to the **same Home 1** as the green-clad founder. The helper walks from the shore approach to the shared store before taking work. Repeated N cannot add a third resident, replenish stocks, reset clocks or replace the founder. A fresh session remains solo until N, preserving the earlier smoke-test behavior.

The founder keeps timber/construction, crops/seeds, wool, barter, grain delivery and winter firewood. The helper takes **P fishing, L salting and H salted-fish delivery**. The existing global orders still request work; role assignment decides which resident performs it. The helper prioritizes fish home supply, then ready salting batches, then new catches. Both can work/move/carry simultaneously, with at most one cargo per resident. The visitor remains a separate trader and never joins the workforce. Idle residents stand at separate store-side positions.

If N is ordered while the founder is already doing a coastal job, that job stays with the founder through its complete catch/delivery, processing batch/output transport or home delivery/return. The helper waits for that job to finish, then takes future coastal work. A shared coastal-job owner prevents both residents selecting the same fish or ingredients during this handoff. The founder never abandons cargo at recruitment. Barter still belongs to the founder, so fish processing waits for actual salt returned to shared storage while the helper may catch/store raw fish in parallel.

Each resident has a stable ID, home membership, role, position, task and cargo. The HUD shows the founder and the helper separately and reports which resident currently holds coastal work. Actor bodies and each carried log/grain/wool/salt/fish follow their correct resident in both camera modes, rather than using a single-worker rendering shortcut. Cargo and the original resident retain their entity IDs at recruitment and later camera changes; only the new resident/model are created once.

Calendar, crop growth, household meals and winter fuel still advance **once per world fixed tick**, not once per resident. Both belong to one abstract household: its existing 40-second meal/fuel rules remain unchanged. This step adds labor capacity, not demographic nutrition, wages or individual hunger. It has a free one-time invitation, two fixed roles and no uninvite, manual reassignment, arbitrary worker pool, independent second home, path congestion or resident collision/health/mortality simulation. Colors, arrival and role split are fictional game choices, not a reconstruction of Gotlandic household size or labor customs.

Screenshots: [solo catch already underway](docs/screenshots/crew-solo-job.png), [recruitment with the original job still held by the founder](docs/screenshots/crew-handoff.png), [log and fish carried by different residents](docs/screenshots/crew-parallel-cargo.png), [both residents in first person](docs/screenshots/crew-first-person.png), and [finite economy completed with two residents](docs/screenshots/crew-finished.png).

### Crew smoke test

1. Start solo. Press P, wait until the founder begins walking/fishing, then click N twice. Verify one blue-clad helper appears, the founder ID persists, and the crew count becomes two.
2. While the founder still holds the first coastal job, the helper walks in and waits; no second fish is caught from the same slot. After the founder stores that catch and completes the job, future coastal work transfers to the helper.
3. Order trees/building or C/J while P/L/H remain ordered. Observe the green founder doing farm/barter work and the blue helper fishing at the same time. Salt still has to be traded and returned; allocation grants no goods.
4. Switch Tab while either carries cargo. Both residents, tasks and their own carried goods persist. Watch a salt pouch follow the founder when coming from trade, then follow the helper when transported to the salting table.
5. Let fish processing and meals finish. Finite totals are still four fish, two used salt and four trader-owned wool. With two crop harvests, total food used can reach ten (six grain + four fish); world time and ration periods must not double.
6. Repeat in a fresh session without N: the original founder still performs the entire chain. Automated tests also invite during raw-ingredient hauling, salting and fish home delivery and verify full material/cargo balances across two winters.

## Save, restore and pause

**Space** pauses/resumes settlement work, recruitment, grazing, crop growth, seasons, food and winter fuel clocks together. The status panel above the order bar says **PAUSED** or **RUNNING**. Camera movement, Tab, order queuing and the separate T/Y presentation sunlight remain usable. Orders issued while paused begin when resumed. Focus is required for session keys; first-person mouse capture is not.

**F5** writes one local save slot, `hafthi-save.json`, in the working directory (the repository root when using the command above). It replaces the previous successful save. **F9** loads that slot and always leaves the simulation **paused**; press Space when ready. No automatic loading or saving occurs at startup/exit. Missing, truncated, damaged or unsupported-version saves show an error in the status panel and leave the current session untouched.

The snapshot includes both residents' stable domain IDs, roles, positions, tasks, partial work timers, carried item slots, exclusive coastal ownership, pending orders, every material location, processing/crop progress, calendar and household/fuel timers, counters, and presentation sunlight. Loading retains the current camera view, selected tree and render quality. Existing resident ECS entities are reused by domain ID; the helper and its visible body are removed/created as needed when moving between solo and two-resident saves. Bevy entity numbers are runtime details and need not survive restarting the application.

The versioned JSON envelope contains a checksum of its serialized payload to detect accidental damage; hand-editing saves is unsupported. Writes use a temporary sibling file, flush it and rename it over the slot only after successful encoding/writing. A pre-existing `.tmp` file causes an explicit failure and leaves the previous slot untouched; after an interrupted save, remove that stale temporary file before trying F5 again. The slot and temporary file are gitignored. This is a single desktop slot, with no migration promise for future format versions, cloud sync or save browser.

Manual check:

1. Press P, N, C, J, L and H; optionally queue timber/crops too. During a visible delivery, press Space. Check that time, food timers, positions and cargo stop while Tab and camera movement still work.
2. Press F5. Resume with Space and let jobs progress. Press F9: the earlier time, tasks, inventories and carried item return, paused. Press Space and watch the delivery continue.
3. Quit and restart with `cargo run --release --locked`, then F9. Verify the same saved cargo/crew and resume. A new launch otherwise starts fresh.
4. Save a solo session, invite the helper, then F9. Verify the helper disappears and one founder remains. Repeat loading a two-resident save and verify exactly two residents, without duplicate bodies.

Six snapshot tests check uninterrupted versus restored continuation throughout busy two-resident chains, exact timer/cargo round trips, atomic file failure and damage/version rejection, pause with live camera changes, focused session input, and both domain/visual roster restoration including helper children.

## First outbound wool collection

This package connects **arrival → physical loading → departure** to the existing wool/salt trade. **U / Boat** requests one collection voyage. The boat waits off scene until **all four wool bundles have actually been delivered to the shore visitor and both paid salt pouches have been carried back** (or subsequently used for salting). Missing fleece, stored wool or incomplete payment cannot summon a usable cargo. C and J remain the orders for obtaining/trading wool; U grants no wool, salt, payment or extra harvest.

The boat approaches an authored southwestern landing over **20 simulation seconds**, then waits without a deadline. The resident assigned to coastal work carries each of the visitor's existing bundles from the visitor pile to the dry-land handover point, places it aboard, and returns to the visitor before choosing more work. The founder performs this if solo; an invited coastal helper takes it after completing any current coastal job. Home fish supply and salting retain priority. Carrying uses the same worker cargo slot and the same four persistent wool meshes. They move from **trader → carried → aboard → exported**. This is forwarding the trader's already-paid goods, with no second sale or reward.

Only after **all four bundles are aboard** does the boat depart, taking another 20 simulation seconds. During departure, the actual cargo travels with the hull. At the end the hull and exported bundles hide; their entities remain. Repeating U never creates another boat, replenishes fleece or resets the two-pouch offer. The boat status panel reports waiting/arrival/loading/departure/completion and aboard/exported counts. All movement and material transitions use the fixed simulation clock, so Space pauses sailing and loading; Tab leaves them running. F5/F9 preserve the voyage's exact elapsed time, cargo locations and any resident mid-delivery. Existing PR #16 saves load with an unrequested voyage; the save format version stays 1 for this additive field.

The handover plank and mooring posts are authored scenery, not a costed building recipe. Residents stop on the existing walkable shore; they do not board a moving boat or walk on water. There is no shipbuilding, simulated sailor labor/rowing/wind physics, passenger transport, navigable route map, repeat trade, imported cargo or new household in this slice. The visitor remains the shore-side contact after departure. Vessel dimensions, route, free collection, capacity and timings are fictional gameplay/art choices, not a reconstruction of a named Gotlandic ship, harbour, wool cargo or trade route circa 900 CE.

Manual check:

1. On a fresh session click **U: Boat** twice before ordering any wool. Verify the boat waits for trade, wool/salt counts do not change, and no extra boat appears.
2. Order C and J; optionally N, P, L and H. Watch four wool bundles reach the visitor and the two real salt pouches return. Only then does the boat approach. Zoom out with E or pan toward the southwest with the arrow keys to follow it.
3. Follow the coastal resident between the visitor and southwestern handover point. Each carried bundle disappears from the visitor pile and appears aboard, one at a time. Switch Tab during loading; the same resident and cargo remain.
4. Press Space during sailing/loading, F5, resume and later F9. Verify the earlier boat position and carried/aboard wool return paused. Resume to see departure only after aboard = 4, followed by exported = 4. Repeat U and verify no new goods or voyage.

Two shipping tests cover the wait/full-load/idempotence rules and the full real shearing/barter/fish/food/export chain with solo and two-resident crews, traversable paths, actual carrier/boat cargo projection, pause during both sailing legs, and persistent mesh entities. Existing save-continuation tests now include the voyage, and the legacy-save test verifies the default voyage for older snapshots. Input guards include native U activation, both camera modes and focus/mouse-release suppression.

## Boat crew and sail handling

The collection boat now carries **two visible visiting sailors** throughout its arrival, loading and departure. One stands by a steering oar; both are permanent children of the boat and travel with it. They are distinct from the founder/coastal helper and do not add household members, consume household rations, recruit ashore or take logistics jobs. Their tunics, bodies and rigging are original procedural geometry.

A curved, two-sided square sail is fully deployed offshore, lowers during the **last three seconds of arrival**, stays furled during loading, then opens during the **first three seconds of departure**. The small furled cloth shrinks as the hanging sail opens, attached to the same yard. The pose is derived directly from the saved voyage phase and elapsed simulation time; it cannot flutter on a separate wall clock while paused or restart its animation after F9. Existing version-1 saves need no new fields. The same boat, crew, sail and cargo entities remain in both camera modes and hide together when the voyage ends.

This is a visual crew/rigging pass. Route and 20-second sailing durations remain scripted; wind, sail force, animated sailor work, turning maneuvers and independent crew needs are future work. Three-second sail handling and the two-person crew are illustrative choices, not historical measurements or a reconstructed Gotlandic rig/crew complement.

To check: use C/J/U (optionally N), follow arrival until the sail lowers, and watch it open after all four bundles are loaded. Press Space and F5/F9 midway through raising/lowering: the cloth should retain its exact height. Tab shows the same two sailors and cargo from the shore. One additional ECS test verifies sail poses across both sailing legs, docking and completion, serialized voyage restoration, pause/camera independence, two persistent visiting crew entities parented to the boat, and unchanged household worker count/material state.

## Live settlement inspection and compact overview

The initial overview now shows the camera mode, year/season, world time, resident count, owned food, shared timber and selected tree, plus the latest order feedback. **F2** expands/collapses the original detailed counters. The inspector on the right shows one selected subject; **I** cycles **home → founder → helper (if present) → boat → closed**, then back to home. These two display controls work while the window is focused in either camera mode, including released first-person mouse capture. They do not pause the simulation or issue economic orders.

In strategy, click a resident, the house or a visible boat to inspect it. Selection uses conservative bounding footprints and the nearest eligible object; an intervening tree retains tree selection. Hidden/completed boats cannot be picked from the scene, but their manifest is still available with I or the **Boat** inspector button. Native **Home / R1 / R2 / Boat / Close** buttons navigate/close the inspector in strategy; R2 is dim and ignored until the helper exists. Keyboard digits 1–3 still select trees. First-person mouse capture keeps its navigation behavior; I provides inspection without a competing click action.

- **Resident:** actual role/home, readable current job, the specific item currently carried (including raw versus salted fish and seed batch), and the resident holding the coastal job.
- **Home:** only its own pantry grain/fish and firewood, food/fuel consumption and missed-demand counters, ration timing (or waiting for first delivery) and winter fuel status. Visiting sailors are not counted as household residents.
- **Boat:** current voyage phase, visiting sailors aboard/off scene, and each of the four original wool bundles' current location, with stored/used salt counts. The list updates during actual loading and export; it grants no materials.

Inspection remains available during Space pause and follows the same world across Tab/F9. It retains a resident's domain ID rather than a transient scene entity. Loading a solo save while inspecting the helper returns the inspector to the home. UI selection and F2 layout are local display choices and are not stored in the save file. Inspector and native-button clicks cannot fall through to tree picking. This is a read-only small-scene inspector, with approximate selection footprints rather than arbitrary mesh-depth selection, camera following, contextual orders or a new economy system.

Manual check:

1. Start fresh: verify the compact overview and Home panel, with one resident and no delivered food/fuel. Use I through founder/boat/closed/home; absent helper is skipped. Toggle F2 twice and verify counters/current orders remain unchanged.
2. Click the founder or house in strategy; use native Boat/Home/R1 buttons and Close. Click a tree and order F as before. Click inside the panel and verify the selected tree does not change. R2 becomes available after N.
3. During fishing/salting/barter/loading, inspect the actual carrier and boat. Verify the described item and each wool slot agree with the visible cargo. Tab keeps the same selection and I changes it in first person; normal click-to-recapture still works.
4. Pause and use I/F2: the world stays paused. Load a solo save while R2 is inspected; verify the panel falls back to Home without an orphaned resident. Lose window focus and verify inspection keys/buttons are ignored and do not replay on refocus.

Three new tests cover nearest subject/tree occlusion and hidden-boat picking; focus, native buttons, panel-click claims, closed-panel behavior, camera modes, helper disappearance and no mutation of paused simulation state; and owned-home/cargo/manifest descriptions from actual model slots. Existing material, save, crew, sail and input tests remain in the full suite.

## Sculpted coastal graphics pass

The most visible box primitives have been replaced by original authored mesh generators: a continuous curved wooden boat with inside/outside shell, shaped deck, rounded gunwales and swept stems; shaped tunics, rounded limbs/boots, belts, hands, hair and simple faces for residents/visitor/sailors; rounded timber wall courses, doorway frame, a bowed thatch surface with strand/color variation, uneven eave fringe and ridge; and irregular clustered foliage. Shared furniture/rack/plank meshes now have softened timber edges. Matte cloth, wood and foliage materials reduce the former plastic shine. Vertex colors give roof, hull, deck and foliage local variation without external textures.

The same existing residents, cargo slots, boat, sail handling, save data, authored routes and collision footprints drive these new models. Geometry does not grant stock or change work speed, food demand or boat capacity. F9 loads older version-1 sessions and immediately shows their current work/cargo with the new art. First-person and strategy inspect the same simulation entities. Roof overhangs and foliage are visual, not new collision walls; movement retains the existing clearances.

Both quality levels use the new main silhouettes. High uses 70 eave tufts per side and five bough clusters per pine layer; Low uses 35 and three, respectively, alongside the existing grass/shadow settings. Meshes/materials are shared where practical, including the eave tufts, boughs and softened furniture cube. Finite meshes use unit normals and consistent triangle winding; the new geometry test also checks that the roof and deck face upward. Linux software rendering verifies functionality at both settings; macOS/Metal performance requires desktop testing after this increase in scene detail.

There are no external models, image assets, textures or new dependencies. This remains a stylized painterly prototype, with further landscape/animation/material work possible. Horizontal log courses, roof fibers, hull lines, clothing/hair and tree shapes are artistic interpretations, not verified reconstructions of a particular Gotlandic building, vessel, costume or botanical survey circa 900 CE. See `docs/visual-references.md` for the evidence/interpretation boundary.

To compare, load an existing boat/loading save and inspect the house and boat in strategy, then Tab to view the sailors and hull from the shore. Follow an actual delivery: the item must remain with the same resident. Test C/J/U and Space/F5/F9 as before. Captures in `docs/screenshots/sculpt-*.png` include matched Low views and High close-ups.

## Resident motion

The founder and invited coastal helper now swing opposite legs and arms while walking, use task-specific arm motions for felling, field work, shearing, fishing and salting, and hold their arms towards their actual carried timber, grain, wool, salt, fish or seed batch. The axe is a child of the right arm and hides while carrying goods. Cargo continues to use the same material slots and world positions; animation cannot create or deliver resources.

`src/motion.rs` computes poses from the resident's current task, cargo, heading and saved `SimulationClock`, with shoulder/hip pivots authored in `art.rs`. `presentation.rs` applies those poses to the existing resident entities. Space freezes all resident motion, Tab observes the same pose in either view, and F9 restores the saved clock and heading. Heading now comes from fixed-step simulation displacement rather than render-frame differences. Version-1 saves accept an optional `facing` field; older saves default to zero until their next movement. Visiting sailors and the shore trader keep their existing poses.

These are simple procedural limb swings and bounded arm reaches, without elbow/knee joints, terrain foot placement or full inverse kinematics. Work gestures and gait timing are artistic interpretations. No new assets or dependencies are required. Three additional tests cover opposite leg swings, cargo-directed arms, old saves, and exact resident pose/heading restoration across pause and camera changes.

To test: order a tree with **1**, **F**, follow the worker through walking, felling and log delivery, then try **C**, **J** or **P/L** for other work/cargo poses. Invite the helper with **N** to watch simultaneous work. Pause mid-delivery, press **F5**, resume, then **F9**: the same job, cargo, body direction and limb pose should return paused. Check both views with **Tab**.

## Coastal art study: textured surfaces and one skinned resident

This review slice establishes an editable asset workflow beyond the earlier primitive-based models. The founder now uses a Blender-authored glTF skin with project-authored clothing/rig and a CC0 MakeHuman anatomical head: a 15-joint skeleton with weighted clothing, elbow/knee/ankle articulation, an adult silhouette, facial landmarks, cuffs/neckline/seams, boots, belt and pouch. `resident.rs` replaces the founder's old render parts only when the complete embedded model is ready, reattaches the existing axe to the hand, and poses the rig from the same saved worker/task/cargo/heading/clock. The founder's simulation entity, job and material inventory persist. Helper, trader and sailors retain their previous art for this first model study.

`coast.rs` replaces the stacked flat shore discs and uniform boulder necklace with a continuous tessellated meadow/gravel/submerged shelf, broken angular limestone beds and clustered stones. The playable centre stays level; this is not a new height-aware navigation system. Soil, limestone and timber use original tileable base-color, normal and roughness maps. Vertex colors add broad meadow variation and visual path wear. Grass uses curved tapered blades with local color variation; pine sprays leave gaps between needles, with tapered trunks and visible bough supports. High/Low retain their shadow difference and use up to 450/160 grass patches before placement exclusions. Texture detail and the new character currently share one resolution/LOD across tiers.

All runtime art is embedded in the binary, so launching the built executable does not require copying a separate assets folder. No new Rust dependency is needed. Editable `.blend`, deterministic texture generation, export scripts and an independent validator for glTF geometry, normals, joint indices and weights live under `tools/art/`; see [asset provenance and regeneration](assets/README.md).

**This remains a quality study, not finished production graphics.** The environment is still a compact diorama with a level playable centre. Roof/water/props and the remaining people need further art work. The new skin has procedural bone poses rather than polished animation clips, facial animation, finger articulation, full cargo IK or foot planting. The costume and appearance are artistic interpretations. Evaluate the captures and desktop build before extending this model to everyone.

Test the default High setting and `--quality low`: inspect the shore and founder at walking distance with Tab, order **1/F** and follow felling/log delivery, pause with Space, save with F5, resume and load with F9. Check heading, knees/arms, cargo and the axe after load and camera changes. Existing material flow/inspection/save controls remain the same. Graphical Linux software-Vulkan checks establish rendering functionality, not Apple Metal performance.

## Continuous coastal landscape renderer

The reference-driven renderer pass replaces the isolated island with a **120 × 108 m coast and hinterland**: shared height-field terrain/navigation, independently culled terrain chunks, hills blending into a gravel beach, deterministic woodland/shrubs, dry stone boundaries and limestone stacks. First-person height follows the terrain and strategy panning reaches the new surroundings. Existing worker routes, household/material stocks, landing and boat water level retain their clearing coordinates.

An embedded forward-PBR water shader adds animated wave normals, shallow/deep color, a foam fringe and sky Fresnel shading above the actual seabed. Surface mip chains use linear-light color averaging, normalized normals and anisotropic filtering. Static gravel/wall/grass groups and shared crown variants limit per-object overhead. High adds HDR/TAA/SSAO; Low uses simpler camera effects and lower terrain/vegetation density. The home has timber framing, muted infill, stone footing and weathered thatch. The authored skin now also replaces helper, visitor and sailor bodies; each actual resident animates from their own saved work state. Visitors keep standing poses.

**F4 toggles the HUD** for visual review. It changes UI visibility only; Space/F5/F9, Tab and the economy remain available. The initial strategy angle places the coast on the right. These changes build the rendering foundation towards the supplied image; distinct character/costume art, polished animation, roof/prop detail and further material/composition work remain. See [renderer architecture, limits and desktop test sequence](docs/rendering-target.md).

## Current resident renderer: textured anatomy and native clips

The runtime people now use MIT-licensed Rocketbox facial/hand anatomy and skin
textures with project-authored clothing, an 80-joint skeleton and four native
Bevy animation clips. Each worker blends from its own saved task/clock; contact
IK corrects stance feet and reaches for actual cargo. Space and F5/F9 preserve
paused and mid-transition poses. Visiting people breathe without gaining worker
roles. Older saves load; new version-one payloads add optional animation state.

This supersedes the earlier 15-joint/procedural resident descriptions above.
It is an art/animation replacement, not a completed landscape overhaul, full
skin SSS or new behavior/navigation AI. All people still share one face. See
[provenance, architecture, limitations and authoring steps](docs/textured-residents.md).

Run with `cargo run --release --locked -- --quality high` (the `--` before game
options is required). Tab enters first person; Space pauses; F4 hides the HUD.

The resident proportion correction shortens the exposed neck by 4 cm while
preserving facial shape, eyes/hair alignment and the neck/collar connection.
A slightly fuller tunic waist softens the pinched torso silhouette. See the
[matched asset portraits and limits](docs/textured-residents.md#proportion-correction).

The wrist correction continues the anatomical hands into the sleeves with
weighted forearm skin, removing the visible gap at each cuff. Export validation
checks both connections through the animation clips and crossfades; see the
[before/after portraits](docs/textured-residents.md#connected-hands-and-sleeves).

## Repeat coastal work and annual trade

The first coastal batch still uses the existing orders: **C** shear, **J** barter,
**P** fish, **L** preserve, **H** supply home and **U** collect the paid wool.
After that batch, **U** can authorize another complete coastal round instead of
leaving the coast permanently exhausted. The boat panel explains what blocks it.

A later round requires a later gameplay year, spring/summer, a full gameplay
year of fleece regrowth after each sheep's shearing, a completed shipment, all
four fish eaten at this home, both salt pouches consumed, all four wool bundles
exported, and no outstanding coastal worker delivery. It never deletes existing
stock to make room. Until all conditions hold, pressing U changes no inventory.

Once accepted, U queues shearing, fishing, barter, salting and wool collection for
the next batch. Home food supply keeps its existing standing order. New fish
start uncaught, salt starts with the visitor, and wool starts on the sheep; the
same actual workers must perform and carry every step. Coat size gradually
regrows with world time. The same visual entities are reused for the next batch.
The boat panel counts completed voyages across rounds, and fish meal and salting
counters remain cumulative. Nothing repeats without U authorizing the next round.

Regrowth pauses with Space, persists through Tab/F5/F9 and is independent of
T/Y sunlight. Older saves default the new counters safely; pre-feature shorn
sheep have no recorded growth delay. The next accepted round records the current
year, so loading a late save does not grant missed years' worth of cargo.
These yields, regrowth timing and a spring/summer trade window are gameplay
choices, not archaeological claims. This adds repeat coastal work. The following second-farmstead slice extends
that foundation to two households and up to three residents.

To test: complete the first real C/J/P/L/H/U chain, wait for fleece growth and a
later spring/summer, then press U. Follow the next round in either view, save
while wool is carried and reload. Expect four additional fish meals, two salting
batches and one additional completed voyage after the work finishes. The three-
round simulation regression checks twelve actual fish meals and six batches.
