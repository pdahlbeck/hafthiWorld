# Resident mesh correction

The close-up supplied by the user rejected the resident's face, hair, clothing
and disconnected limbs. This revision addresses specific model defects; it is
not a finished-character-quality claim or a landscape upgrade.

## Geometry and materials

- Head and hands now derive from the same pinned CC0 MakeHuman anatomical
  reference, with project-authored adaptation, clothes, hair and rig. Hands have
  palms, thumbs and individual fingers instead of overlapping ellipsoids.
- The neck continues into the head. Its lower geometry fits under the neckline;
  the reference's cut shoulder border is not exposed above the shirt.
- Torso and sleeves are fused into one connected remeshed garment. Smooth
  transferred bone weights replace the sharp chest/pelvis boundary. The lower
  hem receives restrained leg influence to reduce rigid skirt/leg overlap.
- Trousers stop inside the tunic instead of sticking through its fitted waist.
  Their lower cuffs sit inside the boots; boot shafts blend to the shin instead
  of rotating rigidly with the whole foot.
- Skin has linear vertex colors, cheek/stubble variation and darker material
  response. Hair follows the anatomical scalp, with an irregular hairline and
  fine strand geometry, rather than an oversized spherical cap. These are
  vertex/procedural surface details, not photogrammetry or a high-resolution
  scanned skin texture.
- Runtime garment tinting uses the glTF material's explicit name, so adding a
  skin or hair texture later cannot accidentally recolor it as a shirt.

The same 15-joint rig, resident IDs, work/heading poses, visiting roles and save
format are retained. Fingers are not articulated; facial animation, distinct
faces/costumes and polished locomotion remain unfinished. The source geometry
has 61,117 exported vertices compared to 17,212 before. Material batching reduces
29 mesh primitives to 9, but this does not prove equal GPU cost; native Metal
performance has not been measured. The first-person entry frames the founder in
a fresh world so close-up review is immediately accessible with Tab.

## Rebuild and verify

```sh
blender --background --python-exit-code 1 --python tools/art/build_resident.py
blender --background tools/art/coastal-resident.blend --python-exit-code 1 --python tools/art/verify_resident_source.py
python3 tools/art/verify_export.py
cargo run --release --locked -- --quality high
```

The source validator checks that the tunic is one connected component and that
66 vertices above the visible trouser hem are inside the actual cloth surface
in rest pose. This prevents the white hip triangles seen in the rejected image;
it does not prove collision-free cloth in every animation. The export validator
also checks skin color data/range, normalized skin weights/normals, joint names
and embedded textures. Blender's `--python-exit-code 1` makes authoring errors
fail the command instead of leaving an old GLB looking like a successful export.

Use F4 to hide the HUD, Space to freeze work, T to freeze sunlight. Review front,
side, walking and carrying poses in the actual game. No illustration or separate
beauty render is used as evidence of game appearance.
