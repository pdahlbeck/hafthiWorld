## Collect the shore visitor's paid wool by boat

The completed wool/salt trade previously left all four paid bundles at the visitor indefinitely. U / Boat now requests one outbound collection: wait for the real trade and returned salt, approach the southwestern landing, carry the visitor's existing wool aboard one bundle at a time, and depart only with all four aboard. No extra materials or payment are created.

The boat uses a fixed simulation route and two 20-second sailing legs. The coastal resident (founder if solo, helper if invited) uses the common exclusive job ownership and cargo slot, after current work/home supply/salting. The four existing wool meshes follow the actual carrier, then the moving hull; exported cargo hides only after departure. A status panel exposes voyage phase and aboard/exported counts, and the native order bar includes U in both-mode input guards.

Space pauses sailing/loading. F5/F9 restore voyage progress and in-flight cargo exactly. The new settlement field defaults for existing PR #16 version-1 saves; checksum handling and file validation remain intact. The boat, mast/furled cloth, handover plank and mooring posts are original procedural scenery. This is not a reconstructed Gotlandic vessel/harbour or a costed harbour-building system. There is no shipbuilding, crew/sail physics, passenger travel, repeated trade, incoming cargo or second sale of the same wool. Historical interpretation limits and controls are documented.

## Validation

- `cargo fmt --check`, `cargo check --locked`, `cargo clippy --locked -- -D warnings`, debug and release builds pass.
- `cargo test --locked`: **77 passed**. Two new tests cover finite request/wait/full-load/departure rules and the complete solo/two-resident shearing → barter → fish/home → shipping loop. Checks include walkable routes, actual cargo projections, pause during both sailing legs and persistent mesh entities.
- Extended existing tests cover native U and focus/cursor guards, exact saved-versus-uninterrupted voyage continuation, and loading legacy saves without a voyage field.
- Graphical Linux software-Vulkan smoke check: native U request before trade, real two-resident barter and fish chain, pause/save/load during arrival and departure, restored cargo in both cameras, full-load departure and repeated U after completion. Screenshots in `docs/screenshots/shipping-*.png`; this is functional verification, not macOS/Metal performance measurement.

Stacked on #16 (`codex/session-save-pause`). Related to #1. No new dependencies.

![One original wool bundle loaded](https://github.com/pdahlbeck/hafthiWorld/blob/codex/coastal-wool-shipment/docs/screenshots/shipping-loading.png)

![Four bundles departing](https://github.com/pdahlbeck/hafthiWorld/blob/codex/coastal-wool-shipment/docs/screenshots/shipping-departure.png)

![Same restored voyage in first person](https://github.com/pdahlbeck/hafthiWorld/blob/codex/coastal-wool-shipment/docs/screenshots/shipping-first-person.png)
