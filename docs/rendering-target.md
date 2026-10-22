# Coastal rendering direction

The user-supplied village image is a visual target for scale, scene density,
material coherence, natural shore/vegetation and atmospheric lighting. It is not
an asset pack or proof of an archaeological reconstruction. We keep Rust/Bevy
0.20 and the shared economy/first-person world from issue #1.

The renderer work here is a substantial foundation change. It does not establish
visual parity with that image: the project still needs a broader authored asset
set, stronger vegetation/character art, distinct clothing/animations, detailed
roof/props and further composition/material work. No fabricated extra households,
workers, ships or production chains are inserted to make a screenshot look busy.

## Terrain and navigation

`landscape.rs` owns a 120 × 108 m continuous height field, replacing the small
island fan/discs. The same function provides terrain vertices, finite-difference
normals, shore placement and first-person eye height. The original working
clearing stays at y=0.2, including the boat landing, so saved jobs/cargo and their
existing routes remain valid. Workers still operate on that clearing; this is
not general terrain-aware worker pathfinding.

The surface uses 90 independently culled 12 m chunks. High uses a uniform 0.5 m
vertex spacing and Low 1 m, so there are no mixed-resolution T junctions. Hills
fade smoothly to a broad gravel beach. Navigation excludes the water, map edge,
existing buildings/fences/rocks, scenic tree trunks and the new dry stone walls.
The observer can walk into the hinterland and follows its height without a
separate scene. There is no terrain streaming, erosion simulation or editable
height field yet.

## Water

`water.rs` and the embedded `coastal_water.wesl` extend forward StandardMaterial
PBR with world-space wave normals, analytical shallow/deep absorption, a moving
foam fringe and an approximation of sky Fresnel reflection. The shore crest and
underwater slope match the CPU height function. Directional lighting still comes
from the shared daylight system. Water is a translucent surface above the actual
seabed, with the existing boat's water level retained at -0.38.

This is surface shading: there is no fluid simulation, physical refraction,
scene-object reflections, wakes or buoyancy change. Water/wind remain presentation
animations and continue independently of paused economy time, as before.

## Surfaces and vegetation

`terrain_material.rs` and `coastal_ground.wesl` blend a new generated meadow
albedo into soil at the beach and worn yard paths. Four deterministically
rotated/offset samples blend across patches to reduce visible tiling; explicit
texture gradients preserve mip filtering across those changes. Source/provenance is in
`tools/art/meadow-prompt.md`; this is an illustrative albedo, not a calibrated
photogrammetry material.

`surfaces.rs` builds mip chains once for material-bound 2D RGBA surface maps,
including non-power-of-two dimensions with full-area filtering.
Base color is averaged in linear light and normal mips are renormalized. The
sampler uses trilinear filtering and up to 8× anisotropy. UI/font atlas images are
not selected. Supported source maps stay embedded; no new Rust dependency.
Generated texture mip storage adds approximately a third to the base texel data.

Pebbles, wall courses and hinterland grass merge into static groups rather than
one entity per stone/blade. Crowns use shared variants of folded leaf geometry,
not closed foliage balloons; trunks and low shrubs establish depth behind the
original pine clearing. Tree positions are deterministic and use the same trunk
clearances for observation. Decorative woodland is distinct from the three
finite harvestable trees. Limestone stacks and wall placement are fictional
landscape interpretations, not surveyed Gotland locations.

## House, people and view

Boat timber and cloth use a more weathered, restrained palette while retaining
the same hull, crew and manifest. The owned home keeps its dimensions, door and collision footprint but now uses
weathered timber framing, earth-toned infill, a stone footing and restrained
thatch colors/edge fibers. The skinned model from the preceding study is shared by
both residents, the trader and two visiting sailors. Resident bone poses use each
actual resident ID, task, cargo, heading and saved clock; visitors keep standing
poses and do not become household workers. Distinct faces/costumes and polished
animation clips remain asset work.

High adds HDR, temporal anti-aliasing, restrained contrast-adaptive sharpening and screen-space ambient occlusion to the
existing directional shadows. Low retains the simpler camera pipeline and lower
geometry density. Both use the new terrain and water shader, without ray tracing
or a platform-specific rendering backend. Default strategy composition frames
the village from the land side with the sea to the right. **F4** toggles UI
visibility for scene review; it preserves simulation, selection and camera state.

## Desktop review

```sh
cargo run --release --locked -- --quality high
cargo run --release --locked -- --quality low
```

1. Compare the coast/house/woodland at the initial strategy angle; use F4 to view
   the scene clearly, then restore the controls with F4.
2. Tab and walk inland, along the dry beach and towards the stone boundaries.
   Eye height should track the hills; trunks/walls/sea should block movement.
3. Follow 1/F log work, invite the helper with N, then C/J or P/L. Each resident
   should retain their own pose and actual cargo through camera changes.
4. Load an older save with F9, including an active boat voyage. Inspect trader,
   crew, cargo and sail. F5/F9 must preserve jobs/materials and freeze resident
   poses, with no duplicate character bodies.
5. Check T/Y lighting and F3 native frame statistics. Linux software-Vulkan
   captures verify functionality and shader compilation; they are not macOS
   Metal performance evidence. No new performance benchmark matrix is claimed.
