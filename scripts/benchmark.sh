#!/usr/bin/env bash
# Run from a desktop session (Metal on macOS; a working display/Vulkan on Linux).
set -euo pipefail
cd "$(dirname "$0")/.."
output="${1:-/tmp/hafthi-benchmark-$(date +%Y%m%d-%H%M%S)}"
mkdir -p "$output"
output="$(cd "$output" && pwd)"
seconds="${HAFTHI_BENCHMARK_SECONDS:-30}"
repeats="${HAFTHI_BENCHMARK_REPEATS:-3}"
if ! [[ "$repeats" =~ ^[1-9][0-9]*$ ]]; then
  echo 'HAFTHI_BENCHMARK_REPEATS must be a positive integer' >&2
  exit 2
fi
# Refuse to mix a fresh run with old samples.
if [[ -e "$output/metadata.txt" ]]; then
  echo "Choose a fresh output directory: $output already contains a run" >&2
  exit 2
fi
cargo build --release --locked
{
  git rev-parse HEAD
  git status --short
  rustc --version
  uname -sm
  echo '1280x800; idle settlement; fixed 16:00; 10s warmup per process; AutoNoVsync requested'
  echo "duration_seconds=$seconds repeats=$repeats"
  if [[ "$(uname -s)" == Darwin ]]; then
    sw_vers -productVersion
    sysctl -n machdep.cpu.brand_string
  else
    lscpu | sed -n '/Model name:/p'
  fi
} > "$output/metadata.txt"
export RUST_LOG="${RUST_LOG:-bevy_render=info,wgpu_core=warn}"
for quality in low high; do
  for camera in strategy first-person; do
    for ((run=1; run<=repeats; run++)); do
      name="$quality-$camera-$run"
      target/release/hafthi_world --quality "$quality" --benchmark "$seconds" \
        --benchmark-camera "$camera" --benchmark-output "$output/$name.csv" \
        > "$output/$name.log" 2>&1
      # A log line confirms that sampling ended and the CSV was flushed.
      grep '^BENCHMARK ' "$output/$name.log"
    done
  done
done
