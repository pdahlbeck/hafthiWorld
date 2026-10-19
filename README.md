# hafthiWorld

**A living world inspired by Viking Age Gotland.**

hafthiWorld is an experimental 3D world-building game set on the Baltic island of Gotland around the 10th century. The goal is a beautiful, painterly landscape grounded in the island's distinct nature and archaeological history.

Explore limestone shores, meadows, winding paths, timber farmsteads, harbours and sailing vessels. The first milestone is **visual quality**, not a large gameplay feature set.

## Technology

- **Rust** with **Bevy 0.20**
- Native desktop development, initially macOS and Linux
- Original assets and shaders
- Procedural landscape experiments, with historically researched architecture

## Getting started

Install a recent Rust toolchain using [rustup](https://rustup.rs/), then:

```sh
git clone https://github.com/pdahlbeck/hafthiWorld.git
cd hafthiWorld
cargo run
```

Controls in the first prototype: **W/A/S/D** orbit camera, **Q/E** zoom, **Escape** quit.

> **Status:** Early prototype. The initial scene uses simple geometric placeholders; the finished visual style remains to be developed.

## Visual direction

Sunlit Baltic coasts; low coastal vegetation; characteristic Gotland limestone; warm timber and thatched roofs; atmospheric distance; painterly materials; richly populated meadows; carefully stylized, shallow seawater. Avoid generic fantasy imagery and anachronistic architecture.

## Roadmap

1. **First scene:** orbit camera, coast, sea, light and representative scale.
2. **Art prototype:** terrain topology, foliage scattering, wind, water and stylized materials.
3. **Gotland identity:** researched Viking Age farmsteads, stone monuments, boats and harbours.
4. **World interaction:** placement of buildings and paths, animals and settlements.
5. **Living world:** shipping, markets, seasons and everyday activity.

## Development with Codex

See [AGENTS.md](AGENTS.md) for project constraints, implementation conventions and the first suggested assignment.

## License

License has not yet been selected. All rights reserved until the repository owner chooses one.
