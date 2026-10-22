## Replace the most visible block-shaped placeholder models

The boat hull was a stack of small cuboids, people had rectangular torsos/limbs, the house used flat roof slabs, and the pines were stacked cones. This pass replaces those silhouettes with original sculpted procedural meshes:

- Continuous curved inner/outer boat shell, shaped deck, rounded gunwales and swept stems, retaining the same sail, crew and actual cargo.
- Shaped tunics, rounded limbs/boots, belts, hands, hair and simple faces for both residents, the visitor and sailors.
- Rounded timber wall courses, framed doorway, bowed thatch with strand/color variation, irregular eave fringe and ridge.
- Irregular clustered matte foliage and softened edges on authored furniture, racks and handover planks.

Mesh normals/winding, material roughness and vertex-color variation provide depth at walking distance. The low tier reduces small eave/bough details while keeping the new main forms. No external models/textures, dependencies or generated-image assets are used.

This is a rendering/art pass. Model identity, world positions, routes, orders, collision footprints, material accounting, inspection and version-1 saves remain the existing simulation. The house and vessel are fictional interpretations, not a newly established archaeological reconstruction.

## Validation

- Formatting, locked check/test, Clippy with warnings denied, debug and release builds.
- **82 tests**, including finite geometry, unit normals, triangle orientation and upward roof/deck faces, plus the existing full economy/save/crew/sail/camera/inspection tests.
- Graphical Linux software-Vulkan inspection at low and high quality, strategy and first person, using the same existing loading save as prior screenshots. Functional rendering checks do not establish macOS/Metal frame rates.
- Before/after captures and close views in `docs/screenshots/sculpt-*.png`.

Stacked on #19 (`codex/settlement-inspector`), related to #1. User-requested visual upgrade; no new economy systems.


Same save, time and Low quality (software Vulkan; timing overlay is not Apple GPU performance):

| Before | After |
| --- | --- |
| ![Previous box models](https://github.com/pdahlbeck/hafthiWorld/blob/codex/sculpted-coastal-visuals/docs/screenshots/sculpt-before-low.png) | ![Sculpted models](https://github.com/pdahlbeck/hafthiWorld/blob/codex/sculpted-coastal-visuals/docs/screenshots/sculpt-after-low.png) |

![High-quality farmstead](https://github.com/pdahlbeck/hafthiWorld/blob/codex/sculpted-coastal-visuals/docs/screenshots/sculpt-farmstead-high.png)

![Continuous hull and shaped sailors](https://github.com/pdahlbeck/hafthiWorld/blob/codex/sculpted-coastal-visuals/docs/screenshots/sculpt-boat-high.png)
