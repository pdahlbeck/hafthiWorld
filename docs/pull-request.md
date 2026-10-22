## Give the collection boat visible sailors and sail handling

The boat previously travelled with no visible crew and a permanently furled sail. It now carries two visiting sailors and a steering oar, lowers a curved square sail over the last three seconds of arrival, keeps it furled while loading, and raises it over the first three seconds of departure.

Sail deployment is derived from the saved voyage phase/elapsed time, so Space freezes the pose and F9 restores it immediately. Crew/rigging are persistent children of the same boat, move with its actual cargo, and hide with it after completion. The visitors do not become household workers or consume household supplies. No save-schema or dependency changes are needed; existing saves work unchanged.

This is an original visual pass, with scripted sailing and illustrative crew/rig choices. It adds no wind/sail-force physics, independent sailor labor or historical vessel reconstruction. Controls and interpretation limits are documented.

## Validation

- Formatting, locked check/test, Clippy with warnings denied, debug and release builds.
- **78 tests**, including a new ECS check of serialized sailing poses, furl/raise endpoints and midpoints, docking/completion visibility, paused camera changes, persistent mesh/crew identities and unchanged material/worker state.
- Graphical Linux software-Vulkan inspection in both camera modes of docked crew, deployed sail and paused/restored sail handling. Screenshots in `docs/screenshots/rigging-*.png`. This is not a macOS/Metal performance measurement.

Stacked on #17 (`codex/coastal-wool-shipment`). Related to #1.


![Crew aboard while loading](https://github.com/pdahlbeck/hafthiWorld/blob/codex/boat-crew-sail/docs/screenshots/rigging-docked.png)

![Sail and visiting crew in first person](https://github.com/pdahlbeck/hafthiWorld/blob/codex/boat-crew-sail/docs/screenshots/rigging-sailing-first-person.png)

![Restored sail handling during arrival](https://github.com/pdahlbeck/hafthiWorld/blob/codex/boat-crew-sail/docs/screenshots/rigging-lowering.png)
