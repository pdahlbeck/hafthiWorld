# Coastal art study — original project assets

The garment, rig, surface maps and environment are authored for hafthiWorld. The
head adapts MakeHuman’s explicitly CC0 anatomical base (source, pinned revision and
complete license in `tools/art/reference/`). No proprietary game meshes, scans,
photographs or third-party texture packs are included.

- `models/coastal-resident.glb`: one adult coastal resident, 15 deform joints,
  weighted tunic/sleeves/trousers, boots, neck/head, facial landmarks, hair,
  neckline, cuffs, seams, belt and small pouch. A compact exported mesh
  across material/mesh parts. Includes its wool texture maps.
- `materials/*-color.png`: sRGB base colors; `*-normal.png`: linear tangent-space
  normal maps; `*-orm.png`: linear occlusion/roughness/metallic channel packing.
  The runtime uses the roughness/metallic channels; no baked lighting in color.
- Editable source: `tools/art/coastal-resident.blend`. The Blender source uses
  explicit garment profiles, the documented CC0 anatomical head and weighted vertex groups. The accompanying
  scripts regenerate the model and deterministic 512px tileable surface maps.
  The separate generated meadow albedo stays checked in; see `tools/art/meadow-prompt.md`.

Regenerate from the repository root (authoring tools only; neither is needed to
build or play):

```sh
python3 tools/art/bake_surfaces.py  # numpy + Pillow
blender --background --python tools/art/build_resident.py  # Blender 4.3+
python3 tools/art/verify_export.py  # Python standard library
```

`src/art_assets.rs` bundles the runtime art into the binary. Both cargo run and a
standalone executable load the same embedded assets without an extra asset folder.
When adding a runtime asset, register it there as well.

This is a style/production-workflow study, not final character art. Garment cut,
facial appearance, hair and palette are fictional artistic choices, not a verified
reconstruction of one person or outfit on Gotland circa 900 CE. Elbow/knee motion
is procedural on a real weighted skeleton; there are no authored motion-capture
clips, finger animation, facial rig, terrain foot placement or full cargo IK yet.
The same skin is now shared by the founder, helper, trader and two visiting
sailors. Resident rigs follow their own work; visitors retain a standing pose.
Distinct costume/face variants and authored animation clips remain to be made.

`shaders/coastal_water.wesl` is the project-authored forward-PBR water extension.
See `docs/rendering-target.md` for the shader and terrain assumptions.

`materials/meadow-color.png` is an unchanged original ImageGen albedo study,
blended into the soil by `shaders/coastal_ground.wesl`. Its generation prompt,
size and limits are recorded in `tools/art/meadow-prompt.md`. It is not a scan.

### Physical-lighting material review

`materials/thatch-color.png` and `materials/pine-bough.png` are unchanged original
ImageGen outputs for this project (2026-10-23). Full prompts and limits are in
`tools/art/coastal-material-prompts.md`. The reed micro-normal is separately
procedural and reproducible with `python3 tools/art/bake_reed_normal.py`.
These are illustrative textures, not calibrated scans or borrowed game assets.
