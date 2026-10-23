The user’s close-up exposed specific character defects: mitten hands, pale featureless skin, a cap-like hair shell, disconnected sleeve/body forms, white trousers protruding through the tunic at the hips, and rigid boot cuffs. This revision rebuilds the resident mesh and its materials rather than adjusting scene lighting again.

The character now has anatomical hands/head from the same pinned CC0 MakeHuman reference, a continuous lower neck fitted under the shirt, a fused single-component tunic with transferred/blended weights, trouser tops hidden inside the garment, boot shafts blended to the shin, scalp-fitted hair/strand geometry and authored skin colors. Garment tinting identifies the glTF material by name so future skin/hair textures cannot inherit shirt tint. The first-person entry frames the founder in a fresh session.

Keeps the existing 15-joint rig, real resident/visitor identities, saved-clock/task/cargo poses and save format. Material batching reduces 29 primitives to 9; exported vertices increase from 17,212 to 61,117, so no equal-cost or native Metal performance claim is made. Source, provenance and reproduction commands are checked in.

**This corrects model defects; it does not establish finished-game/reference quality or claim a landscape improvement.** Skin detail is vertex-authored, not a scanned skin texture. Faces/fingers are not animated; distinct people/costumes and polished motion remain unfinished. No engine fork or new gameplay roles.

Validation: fmt/check, all **97 Rust tests**, Clippy `-D warnings`, debug/release builds. GLB validation checks joints, normalized weights/normals, embedded textures and skin color preservation. Blender source validation checks that the tunic has one connected component and that 66 hidden upper-trouser vertices are inside the actual cloth surface in rest pose. This catches the reported white hip triangles; it does not promise collision-free cloth in all animation. Actual High/Low game screenshots and work/save restoration checks complement the automated checks.

See [model review](https://github.com/pdahlbeck/hafthiWorld/blob/codex/resident-anatomy-and-skinning/docs/resident-correction.md). Continues #1, based on open PR #24.

![Actual High close-up](https://github.com/pdahlbeck/hafthiWorld/blob/codex/resident-anatomy-and-skinning/docs/screenshots/resident-correction-front-high.png)

[Uncropped functional capture of a restored wool-carry pose](https://github.com/pdahlbeck/hafthiWorld/blob/codex/resident-anatomy-and-skinning/docs/screenshots/resident-correction-loaded-high.png)

![Actual Low close-up](https://github.com/pdahlbeck/hafthiWorld/blob/codex/resident-anatomy-and-skinning/docs/screenshots/resident-correction-front-low.png)
