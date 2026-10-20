# Instructions for Codex — hafthiWorld

You are working on **hafthiWorld**, a Rust/Bevy 0.20 3D world-building game inspired by Gotland during the Viking Age (c. 900 CE).

## Non-negotiable goals

1. Visual excellence first: an original, colorful, painterly 3D rendering style. Not photorealism and not generic fantasy.
2. Historically plausible Gotland: limestone coast, meadows, coastal farmsteads, timber architecture, harbours and seafaring. Validate claims and distinguish evidence from artistic interpretation.
3. Rust-native code using idiomatic Bevy APIs. Keep compilation and macOS/Linux compatibility in mind.
4. Keep the project original: do not reproduce proprietary assets, source code, textures or designs from another game.
5. Keep the initial scope small: a beautiful, interactive coastal diorama before gameplay systems.

## Engineering

- Target Bevy **0.20**. Check the exact 0.20 API before using examples from earlier releases.
- Prefer clean, readable ECS systems and small, named functions.
- Add dependencies sparingly.
- Prefer deterministic procedural generation with a seed where possible.
- Avoid claiming features are complete until implemented and tested.
- Test with `cargo fmt --check`, `cargo check`, and `cargo test` where possible; report any environment blockers.
- Document controls and any changed architecture in README.md.

## First suggested Codex task

Create the **first visually convincing coastal diorama**: an irregular low limestone shore, shallow Baltic water with a clear nearshore color transition, animated grass patches, a small timber building with an era-appropriate silhouette and warm late-afternoon lighting. Preserve orbit camera controls. Favor atmospheric coherence over quantity of objects. Split rendering, world generation, and camera into logical Rust modules. Confirm compatibility with Bevy 0.20.

The checked-in initial scene is a placeholder, not a visual target.

## Updated product direction — 2026-10-20 (user-approved)

**Read and implement the scoped plan in [GitHub issue #1](https://github.com/pdahlbeck/hafthiWorld/issues/1).** The game is now explicitly a **settlement/economy simulation with both an overhead strategy view and first-person walkable view of the same persistent simulation**. Remain on **Rust + Bevy**; do not migrate to Unreal or another engine. Begin with **PR 1: camera-mode switching (Tab) and a small persistent ECS proof of shared simulation**. Then iterate resource gathering, logistics, farms, households, barter, seasons, ships and later conflict. Do not attempt the whole roadmap in one PR.

The earlier visual-only coastal-diorama milestone remains important, but it is **no longer the sole first task**. Keep existing rendering working while incrementally adding the hybrid camera/real-world simulation foundation. The art direction remains distinctive and painterly with plausible natural materials and excellent lighting; aim for visually rich realistic behavior without requiring photorealism. All camera modes must observe the same inhabitants, resources, inventory and ongoing tasks. Build and test changes incrementally.
