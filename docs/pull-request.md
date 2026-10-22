## Coastal art study: original PBR surfaces and one skinned resident

The flat-color ground, repeated rounded stones and primitive people were useful alpha placeholders but did not establish a convincing finished art direction. This is a focused **quality/asset-workflow study** for review, covering the coastal environment and founder.

- Continuous tessellated meadow, gravel shoulder and submerged shelf; clustered angular limestone beds with broken ledges.
- Original tileable base-color, tangent-normal and roughness maps for soil, stone and timber, with broader vertex-color meadow/path variation and less orange late-afternoon light.
- Curved, varied grass blades and open pine needle sprays, tapered trunks and bough supports. Low/High use up to 160/450 grass patches before placement exclusions.
- One Blender-authored resident with project-specific clothing/rig and a CC0 MakeHuman anatomical head: adult proportions, facial landmarks, tunic/sleeves/trousers, cuffs, seams, boots, belt and pouch; **15-joint weighted glTF skin**, with procedural shoulder/elbow/hip/knee/ankle poses from the real worker/task/cargo/saved clock. The existing axe attaches to the hand.
- Editable `.blend`, surface/model generation scripts and independent export validation checked in. Runtime art is embedded into the executable, so both cargo run and standalone binaries find the same assets. No added Rust dependencies or proprietary game art. The anatomical head reference has its pinned source and complete CC0 license under `tools/art/reference/`.

The founder's render root and simulation remain the existing resident. Swapping the model waits for complete asset loading. Jobs, routes, finite cargo, inspection and session saves retain their simulation behavior. Helper/trader/sailors retain the previous models so the first authored resident can be reviewed before extending the cast.

### Limits to review

This is **not finished production art**. The playable centre stays level; navigation is not terrain-aware. Roof/water/props and the remaining people still need work. The skin has procedural bone poses rather than polished authored clips, finger/facial animation, full cargo IK or foot planting. There is one character mesh LOD/texture resolution across quality tiers. Appearance/costume/terrain details remain fictional artistic interpretations, not a new archaeological reconstruction.

### Validation

- Formatting, locked check/test, Clippy with warnings denied, debug and release builds passed.
- **87 Rust tests passed**, including new finite coastal geometry/UV and deterministic rig-pose/cargo-heading tests, plus the existing economy, save, camera, crew, inspection and pose-restoration suite.
- Independent glTF validator: **17,212 exported vertices**, 15 joints, finite positions, unit normals, normalized weights, valid joint indices and embedded textures.
- Linux software-Vulkan graphical checks in High and Low, strategy/first person, with actual work and session save/load. These establish functionality, not macOS/Metal frame rates.
- Raw in-game captures below; source/provenance/regeneration and manual desktop sequence in README and `assets/README.md`.

Stacked on #21 (`codex/resident-motion`), continuing #1 and the requested art-direction review.

![Coastal environment, High](https://github.com/pdahlbeck/hafthiWorld/blob/codex/coastal-art-study/docs/screenshots/art-study-coast-high.png)

![First resident study, High](https://github.com/pdahlbeck/hafthiWorld/blob/codex/coastal-art-study/docs/screenshots/art-study-resident-high.png)
