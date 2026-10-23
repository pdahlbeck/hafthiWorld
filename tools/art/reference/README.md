# Anatomical head reference (CC0)

`makehuman-head.obj` is a cropped head/neck excerpt from MakeHuman's `base.obj`
(body-group faces whose vertices are above Y=5.7), preserving upstream coordinates
and UVs. The runtime character adapts its scale, axis, material, eyes/hair and rig.
No MakeHuman application source code is included or required at build/runtime.

Source: https://github.com/makehumancommunity/makehuman/blob/a8bc2d54ff0ac92e78ff71431b1023eda42bf482/makehuman/data/3dobjs/base.obj

Upstream file header explicitly releases this asset under CC0 in September 2020.
Copyright holders named there: Data Collection AB, Joel Palmius, Jonas Hauquier.
The complete upstream asset license is included as `CC0-1.0.md`, retrieved from
`LICENSE.ASSETS.md` at the same repository. No scans or game-proprietary models.

## Full anatomical reference for the resident correction

`makehuman-base.obj` is the complete asset from the **same pinned source and CC0
license above**. It is kept as an editable anatomical reference. The builder
extracts only the clothed character's head/neck and hands, adapts their positions,
adds original skin colors/hair/costume and binds them to the project's existing
15-joint rig. The full unclothed reference body and helper meshes are not exported
to the game. The earlier head excerpt remains for provenance of the preceding
study; the new builder reads `makehuman-base.obj`.
