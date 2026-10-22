## Inspect the live settlement without covering the landscape

The full debug-style overview previously covered much of the scene and exposed tasks as Rust enum names. The default HUD is now compact; F2 restores the detailed counters. A right-side inspector shows the actual resident job/cargo, owned household food/firewood, or the collection boat's four-slot wool manifest and visiting sailors.

Click a resident, house or visible boat in strategy, use native Home/R1/R2/Boat/Close buttons, or cycle subjects with I in either camera mode. Foreground trees retain their existing selection; hidden boats cannot be clicked. Inspector clicks are consumed before tree selection, absent helpers are skipped/dimmed, and losing focus discards pending actions. First-person navigation/click-to-capture and all production keys retain their controls.

Inspection reads the existing model and never grants resources or issues jobs. Pausing, camera changes and load keep the same live selection; loading a solo save while the helper is selected falls back to the household. Display selection/detail state stays outside saved economy state, so no schema/dependency changes are required. Picking uses conservative nearest footprints with tree occlusion, not arbitrary mesh-depth selection or camera following.

## Validation

- Formatting, locked check/test, Clippy with warnings denied, debug and release builds.
- **81 tests**. Three new checks cover nearest/occluded/hidden picks; native and keyboard input in both modes, focus suppression, closed-panel claims, helper disappearance and unchanged paused simulation; and owned-food/cargo/manifest descriptions from actual slots.
- Graphical Linux software-Vulkan smoke: world resident/house/boat clicks, native panel navigation, protected tree selection, F2, carried-wool inspection from an existing save, first-person inspection and helper-to-solo load fallback. Screenshots in `docs/screenshots/inspection-*.png`. This is not macOS/Metal performance measurement.

Stacked on #18 (`codex/boat-crew-sail`). Related to #1. No new historical claims, assets or economy changes.


![Compact overview and household](https://github.com/pdahlbeck/hafthiWorld/blob/codex/settlement-inspector/docs/screenshots/inspection-home.png)

![Actual carried wool](https://github.com/pdahlbeck/hafthiWorld/blob/codex/settlement-inspector/docs/screenshots/inspection-carried-wool.png)

![Boat manifest in first person](https://github.com/pdahlbeck/hafthiWorld/blob/codex/settlement-inspector/docs/screenshots/inspection-first-person.png)
