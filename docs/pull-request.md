## Continuous coastal terrain and a layered PBR renderer

The farm previously sat on a small level island with a flat sea plane and simple repeated scenery. This pass builds the rendering/navigation foundation for a more coherent coastal landscape while retaining the existing economy and saved work:

- **120 × 108 m shared height field**, 90 independently culled chunks, hills fading continuously into the beach, and terrain-following first-person eye height. Original work/landing coordinates retain y=0.2; observer navigation respects water, trees, stone walls and existing obstacles.
- **Two-layer terrain material** blending a new meadow albedo into soil at the shore and worn paths. Deterministic rotated/offset sampling reduces repeating patterns. Original ImageGen albedo and prompt/provenance are checked in; this is illustrative art, not a calibrated scan.
- **Forward-PBR coastal water extension** with world-space wave normals, shallow/deep color, foam fringe and approximate sky Fresnel shading over the seabed. Existing boat water level is retained. Both shaders are embedded and use Bevy's standard lighting/fog pipeline.
- **Surface mip chains** with linear-light color filtering, renormalized normals, full-area filtering for odd/non-power-of-two dimensions, and anisotropic sampling. Font/UI images are excluded.
- Deterministic woodland with folded leaf geometry and shared crown variants, shrubs, merged gravel/grass/stone-wall groups, and interpreted limestone stacks. Decorative trees remain separate from finite harvestable resources.
- The owned house gains weathered timber framing, muted infill, stone footing and restrained thatch; boat timber/cloth use the same weathered material direction. The authored skin now replaces helper, trader and sailor bodies too. Each resident's rig follows its own ID/task/cargo/saved clock; visitors retain standing poses. Garment tints distinguish residents and visitors without changing shared source materials.
- High adds HDR, TAA, restrained sharpening and SSAO to directional shadows. Low keeps simpler camera effects and lower geometry density. F4 toggles HUD visibility for scene review. Strategy composition frames the sea on the right.

No new workers, households, ships or material stocks are fabricated for scene density. The existing settlement, resource chains, boat manifest, inspection and session controls remain the source of visible work/cargo. No Rust dependency was added.

### Validation

- Formatting, locked check/test, Clippy with warnings denied, debug and release builds.
- **95 Rust tests**, covering shared height/landing/boat contracts, chunk edges, coastline continuity, scenic obstacle placement, sRGB/normal/odd-size mip filtering, per-resident rig ownership/clock restoration and the existing economy/save/camera/crew/inspection suite.
- Independent glTF export validator: 17,212 vertices, 15 joints, finite positions, unit normals/weights, valid joint indices and embedded textures.
- Linux software-Vulkan graphical checks: Low and High shaders/pipelines, strategy and first person, F4, work/cargo, and older session/boat loading. Captures are raw in-game frames. macOS Metal performance has not been measured; no new benchmark matrix is claimed.

### Art and scope limits

This is a rendering foundation change towards the supplied reference, **not a claim of visual parity**. Stronger authored vegetation/character/roof/prop art, distinct faces/costumes, polished animation and further composition/material work remain. Water uses approximate sky reflection and analytical shallow color, with no scene reflections, physical refraction, wakes or buoyancy. Workers remain on the original level clearing; observer height/navigation extends into the hills. No terrain streaming/editing or general terrain-aware worker pathfinding yet. All historical layouts, costume, walls and stack locations remain artistic interpretations.

Architecture and desktop review sequence: `docs/rendering-target.md`. Asset sources: `assets/README.md`, including the preceding CC0 anatomical head and the new generated meadow albedo.

Stacked on #22 (`codex/coastal-art-study`), continuing #1.

![Coastal renderer, High](https://github.com/pdahlbeck/hafthiWorld/blob/codex/coastal-landscape-renderer/docs/screenshots/renderer-coast-high.png)

![Boat and real cargo from an existing session](https://github.com/pdahlbeck/hafthiWorld/blob/codex/coastal-landscape-renderer/docs/screenshots/renderer-boat-high.png)

![First-person High](https://github.com/pdahlbeck/hafthiWorld/blob/codex/coastal-landscape-renderer/docs/screenshots/renderer-first-person-high.png)

![Low, same saved loading state](https://github.com/pdahlbeck/hafthiWorld/blob/codex/coastal-landscape-renderer/docs/screenshots/renderer-coast-low.png)
