The previous coastal pass was rejected visually: uniform fill lighting, flat roof surfaces, triangular pine fans and speckled terrain still read as placeholder art. This revision uses Bevy 0.20's physical atmosphere and sky-derived environment lighting in High, with photographic exposure, ACES, temporal shadow filtering and contact shadows. The house gets original reed texture/micro-normals and geometric roof detail; pines use original alpha-cut branch art; terrain gets broader variation and water receives sky environment lighting.

This is a visual review, **not a claim of parity with the supplied finished-village reference**. Character art, architecture, stonework and vegetation still need substantial authored work. A Bevy fork is not included: the required sky/IBL/shadow features are already available upstream. No engine migration, new residents, invented production chains or changed inventory rules.

- Keeps the shared persistent simulation, first-person height/navigation and existing save format. New lighting/exposure derives from the saved hour, with the physical sun crossing below the horizon at night and a weak moonlight/readability floor.
- Low keeps the lighter lighting pipeline and shares the revised art. Alpha-weighted color mip filtering prevents transparent black from darkening needle edges.
- Generated material PNGs are unchanged original outputs; separate analytic reed normals and full provenance/prompts are checked in. Assets remain embedded; no runtime downloads or new Rust dependencies.

Validation: `cargo fmt --check`, `cargo check --locked`, all **97 tests**, Clippy with `-D warnings`, debug/release builds, original glTF validator. Tests include solar horizon/exposure restoration, transparent mip colors and mesh normals/winding, plus the existing shared economy/save suite. Both quality settings and camera modes are checked on software Vulkan. No macOS Metal performance claim or new benchmark matrix.

See [renderer notes](https://github.com/pdahlbeck/hafthiWorld/blob/codex/physical-coastal-lighting/docs/physical-renderer.md). Continues #1. Based on the open coastal renderer PR #23.

Raw game captures, using an existing saved voyage and frozen 16:00 light:

![High coastal view](https://github.com/pdahlbeck/hafthiWorld/blob/codex/physical-coastal-lighting/docs/screenshots/physical-coast-high.png)

![High first person](https://github.com/pdahlbeck/hafthiWorld/blob/codex/physical-coastal-lighting/docs/screenshots/physical-first-person-high.png)

![Low coastal view](https://github.com/pdahlbeck/hafthiWorld/blob/codex/physical-coastal-lighting/docs/screenshots/physical-coast-low.png)
