The resident close-up showed featureless skin, crude hands and rigid movement. Replace the runtime people with MIT Rocketbox head/neck/hair/hands and PBR skin maps, retaining the project's costume and actual simulation identities. Modern clothing/accessories are excluded. The continuous anatomical neck extends under the collar; its base and the upper garment follow the same upper spine to prevent a floating head during motion. Source commit, checksums, full MIT notice and editable Blender source are included; the notice is also embedded in the runtime GLB.

Use an 80-joint rig and four named clips through Bevy's native AnimationGraph/AnimationPlayer. Each worker crossfades from its own task and saved clock; headings interpolate, walk cadence follows actual movement speed, and post-animation two-bone IK corrects stance feet and reaches towards actual cargo. Trader and sailors breathe without acquiring jobs or inventory. Version-one saves gain optional blend/heading state; old payloads remain readable. No engine fork, Rust dependency or AI/economy change.

Fix conversion defects caught during visual review: incorrect sleeve weights pulling the skirt towards wrists, Blender zero-length bone initialization changing rest axes, and concealed trouser tops protruding during a forward stride. The independent validator now catches idle A-pose regression as well as weights, PBR maps, embedded license, four clips, sampled skin bounds unwanted root travel, and the entire lower neck rim staying inside the actual deformed tunic across rest, 36 clip poses and 18 crossfades.

Validation: fmt/check, 100 Rust tests, Clippy `-D warnings`, debug/release builds and GLB validation. Actual High/Low game captures accompany the pipeline notes. Linux software-Vulkan verifies rendering functionality; native Metal performance is unmeasured. Geometry drops from 61,117 to 18,115 vertices but texture/animation cost changes, so this is not a performance claim.

Limits: one shared face/costume family; no facial animation, full skin SSS, cloth simulation or new behavior/navigation AI. Generic work clips and the costume are artistic interpretations, not historical reconstructions. This does not establish finished-game/reference visual quality or improve the landscape.

Continues #1. Based on open PR #25; review this diff against `codex/resident-anatomy-and-skinning`.

See [pipeline, provenance and manual test](https://github.com/pdahlbeck/hafthiWorld/blob/codex/textured-residents-animation-graph/docs/textured-residents.md).

![Actual High game capture](https://github.com/pdahlbeck/hafthiWorld/blob/codex/textured-residents-animation-graph/docs/screenshots/textured-resident-front-high.png)

![Actual Low game capture](https://github.com/pdahlbeck/hafthiWorld/blob/codex/textured-residents-animation-graph/docs/screenshots/textured-resident-front-low.png)

[Actual restored log transport](https://github.com/pdahlbeck/hafthiWorld/blob/codex/textured-residents-animation-graph/docs/screenshots/textured-resident-cargo-restored-low.png)
