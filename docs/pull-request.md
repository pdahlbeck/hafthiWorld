Tree harvesting and timber-rack construction are now player-ordered tasks in the same simulation used by both cameras. Select one of three trees by clicking its crown, a numbered UI button or keys 1–3; order harvest with F or the Harvest button. The worker walks over, fells it, and carries its two finite logs individually to storage.

B or Build rack orders one predefined two-log construction. It waits when storage is empty. The worker removes stored logs only when picking them up, carries each to the site, installs the posts and assembles the final beam. Every material unit has exactly one location (in tree, ground, carried, stored or installed). Duplicate orders cannot regrow trees, create logs or charge twice. The six log entities, tree entities and worker remain persistent across camera changes.

This implements **PR 2 — basic resource loop** from #1. Scope is one worker, three trees, one material and one fixed timber-rack site. No new dependencies, large art pass, free placement, general pathfinding or economy expansion. Existing coastal geometry, strategy/first-person controls, cursor policy and collision remain functional. The small open rack has no separate collision footprint; geometry, yields and timings are technical/artistic placeholders.

Validation with Rust 1.99.1 / Bevy 0.20:

- `cargo fmt --check`
- `cargo check --locked`
- `cargo test --locked`: **11 passed**
- `cargo clippy --locked -- -D warnings`
- `cargo build --locked`

Tests cover material conservation, waiting for timber, complete transport/install/assembly, idempotence, all three authored routes, frame partitioning, native-button order delivery, focus/mode guards, and persistent scene/simulation entity identities through camera changes. Linux software-Vulkan desktop smoke testing covers tree selection, order buttons, shared state and completed construction; macOS/Metal performance was not measured.

README includes controls, architecture and a repeatable manual smoke test. Screenshots: [orders](https://github.com/pdahlbeck/hafthiWorld/blob/codex/resource-loop/docs/screenshots/milestone2-orders.png), [completed construction](https://github.com/pdahlbeck/hafthiWorld/blob/codex/resource-loop/docs/screenshots/milestone2-complete.png).
