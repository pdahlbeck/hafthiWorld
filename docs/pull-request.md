## Animate residents from their actual work and cargo

Residents previously slid along their routes with static limbs. The founder and coastal helper now walk with alternating legs/arms, perform procedural work gestures, and reach towards the real cargo they carry. The axe follows the right arm and hides while carrying goods.

Poses derive from the saved simulation clock, task and cargo. Body heading is recorded from fixed-step movement and saved with the resident, so pausing, changing camera and loading restore a consistent pose. Older version-1 saves default the optional heading field until the next movement. Visiting sailors and trader retain their existing poses.

This uses the existing original meshes, with shoulder/hip pivots and bounded arm reaches. It is a small procedural animation pass without articulated elbows/knees or terrain foot placement. Routes, job timing, cargo positions and material accounting retain their existing behavior. No dependencies or external assets.

## Validation

- Locked check/test, formatting, Clippy with warnings denied, debug and release builds.
- **85 tests passed**, including new cargo-directed arm/alternating-leg tests, legacy heading defaults, and exact resident pose/heading restoration through pause and camera switches.
- Linux software-Vulkan Low graphical check: actual wool-carrying resident and helper, pause/resume, F5/F9 and both camera modes. This is functional validation, not a macOS performance measurement.
- README documents the implementation, limits and desktop test sequence.

Stacked on #20 (`codex/sculpted-coastal-visuals`), continuing #1.
