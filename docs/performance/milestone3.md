# Milestone 3: measured scope and remaining targets

Measured 2026-10-21 from implementation commit `3a8d0c08324427f60299da23a6a60cce6b521d42` with a clean working tree. Rust 1.99.1, Bevy 0.20.0, standard optimized release profile, Debian 13 x86_64 cloud machine, 5 available CPU cores (Intel Xeon Platinum 8573C), approximately 33 GiB RAM. Xvfb at 1280×800, Vulkan **llvmpipe** software rendering (Mesa 25.0.7, LLVM 19.1.7); no physical GPU. The adapter was identified in the graphical smoke log and software Vulkan was selected for the matrix via `VK_DRIVER_FILES` pointing to the installed llvmpipe ICD.

Command (after sourcing this cloud environment's tool/library activation):

```sh
DISPLAY=:99 XDG_CACHE_HOME=/workspace/.setup/cache WGPU_BACKEND=vulkan \
HAFTHI_BENCHMARK_REPEATS=1 HAFTHI_BENCHMARK_SECONDS=20 \
  bash scripts/benchmark.sh /workspace/.setup/m3-release-results
```

No compilation or other game process ran during sampling. Each case used a fresh process, ten seconds warmup, then twenty seconds of wall-time sampling. The view was untouched, the settlement idle, lighting fixed at 16:00, and `AutoNoVsync` requested. Driver caches were reused; this is not a cold-cache startup benchmark. The script exited successfully after all four cases. The table was independently recomputed from each current-run CSV, checking nonempty samples, contiguous frame numbers and matching quality/camera/release labels. Millisecond columns use nearest-rank percentiles; FPS is 1000 / mean frame milliseconds.

| Quality | Camera | Frames | FPS | Mean ms | p50 ms | p95 ms | p99 ms |
| --- | --- | ---: | ---: | ---: | ---: | ---: | ---: |
| low | strategy | 118 | 5.92 | 169.005 | 171.812 | 191.282 | 263.116 |
| low | first-person | 104 | 5.19 | 192.710 | 190.051 | 220.066 | 265.827 |
| high | strategy | 85 | 4.24 | 236.007 | 230.652 | 253.814 | 262.617 |
| high | first-person | 67 | 3.37 | 296.589 | 295.641 | 313.371 | 335.358 |

[Raw CSV, run logs and metadata](linux-llvmpipe/) are retained so slow frames remain visible. The CPU wall intervals include rendering/presentation waits and CSV/statistics overhead. They are **not GPU timestamp timings**. The short single-repeat cases validate the measurement pipeline and show this CPU renderer is slow; they establish neither a stable quality-tier speedup nor a before/after optimization claim. Do not extrapolate these FPS values to a hardware GPU.

## Validation

- `cargo fmt --check`, `cargo check --locked`, `cargo test --locked` (**15 passed**, zero ignored), `cargo clippy --locked -- -D warnings`, `cargo build --locked` and `cargo build --release --locked` passed.
- Both camera modes render the new scene. Keyboard harvest/build orders still start the same worker's tasks across a camera switch. A release graphical run also completed the rack (four units still in trees, two installed, none duplicated); T/Y paused and advanced the visible sky/sun through night while work continued. The existing simulation/presentation tests exercise the complete material chain and persistent entity identities; the graphical smoke is separate from those automated tests.
- New tests cover rendered-shore and pine-trunk clearance, readable day/night sun brightness, argument validation, and slow-frame percentile handling. An actual invalid-duration launch returned exit 2; an actual unwritable CSV destination returned exit 1. All four timed runs returned success and produced samples.
- Screenshots in `docs/screenshots/milestone3-*.png` document the scene and shared worker from both viewpoints. Development screenshot HUD timings are not the release benchmark data above. ALSA reports no sound device on this cloud host; audio hardware was not tested.

## Apple Silicon / Metal: not measured

No macOS machine is available here. Native macOS compilation, Metal behavior, GPU timings, thermal stability and Apple Silicon FPS remain **unrun**. The portable `Instant`-based instrumentation and `scripts/benchmark.sh` are implemented for a native Mac run; that is not a claim of Mac validation. On a target Mac, run the documented default three-repeat matrix, retain adapter/OS/chip/power settings and per-run percentiles, and compare the same workload on the same machine. Confirm the Metal adapter in logs (set `RUST_LOG=info` explicitly if the shell suppresses engine logs). Instruments/Metal profiling is still required to separate CPU and GPU bottlenecks. No 60-FPS budget or production performance claim is made by this PR.
