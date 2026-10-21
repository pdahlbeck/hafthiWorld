Adds the first small farming slice from #1: **order sowing → collect/carry finite seed → sow → grow → order reaping → carry four sacks to storage**. O/R and native strategy buttons issue intent; the same persistent worker performs the work in the settlement seen by both cameras. Crops and seed/grain sacks persist as entities. HUD counts distinguish stored/carried/used seed and ground/carried/stored grain.

This is a stacked PR based on #4 (`codex/gotland-visuals-performance`), which remains open. Review this diff against that branch; merge #4 first, then retarget this PR to main.

One plot and one finite crop cycle are included. Growth follows simulation time, independently of the presentation sun; repeated or premature orders cannot mint resources. Existing timber priorities remain, and field traffic uses the southern approach to avoid the house. The crop mesh/yields/durations are illustrative, with no species-specific historical claims. Seasons, food consumption, livestock, households, replanting and saves remain later work. No dependencies are added.

Validation: `cargo fmt --check`, `cargo check --locked`, **21 tests**, Clippy with warnings denied, and debug/release builds passed. Tests exercise seed use, complete harvest/transport, rejected duplicates, mixed farming/timber routes, frame partitions, focus/cursor guards and persistent entities through camera changes. A release Linux software-Vulkan run verified sowing, a ripe plot in both viewpoints and all four sacks delivered (zero left on the field or carried); screenshots and the complete crop smoke procedure are included in README.

The user has already accepted the prior playable visual slice on their Mac. No further benchmark matrix or new Mac performance claim is part of this PR.
