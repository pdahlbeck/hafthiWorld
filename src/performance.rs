//! Portable CPU wall-frame instrumentation. These are not GPU timestamp timings.
use crate::{
    camera::{CameraMode, CameraRig},
    world::Daylight,
};
use bevy::prelude::*;
use std::{
    collections::VecDeque,
    fs::File,
    io::{BufWriter, Write},
    time::Instant,
};

#[derive(Resource, Clone, Copy, Debug, PartialEq, Eq, Default)]
pub enum VisualQuality {
    Low,
    #[default]
    High,
}
impl VisualQuality {
    pub fn grass_patches(self) -> u32 {
        match self {
            Self::Low => 60,
            Self::High => 180,
        }
    }
    pub fn shadows(self) -> bool {
        self == Self::High
    }
    pub fn label(self) -> &'static str {
        match self {
            Self::Low => "low",
            Self::High => "high",
        }
    }
}
#[derive(Resource, Default)]
pub struct RunOptions {
    pub quality: VisualQuality,
    pub benchmark: Option<Benchmark>,
}
pub struct Benchmark {
    pub seconds: f64,
    pub camera: CameraMode,
    pub output: String,
}
impl RunOptions {
    pub fn parse(args: impl IntoIterator<Item = String>) -> Result<Self, String> {
        let mut args = args.into_iter();
        let mut options = Self::default();
        let mut seconds = None;
        let mut camera = CameraMode::Strategy;
        let mut output = String::from("benchmark.csv");
        while let Some(arg) = args.next() {
            let value = args
                .next()
                .ok_or_else(|| format!("Missing value after {arg}"))?;
            match arg.as_str() {
                "--quality" => {
                    options.quality = match value.as_str() {
                        "low" => VisualQuality::Low,
                        "high" => VisualQuality::High,
                        _ => return Err("--quality must be low or high".into()),
                    }
                }
                "--benchmark" => {
                    let n = value
                        .parse::<f64>()
                        .map_err(|_| "--benchmark requires seconds")?;
                    if !n.is_finite() || !(1.0..=3600.0).contains(&n) {
                        return Err("benchmark duration must be 1..3600 seconds".into());
                    }
                    seconds = Some(n);
                }
                "--benchmark-camera" => {
                    camera = match value.as_str() {
                        "strategy" => CameraMode::Strategy,
                        "first-person" => CameraMode::FirstPerson,
                        _ => return Err("benchmark camera must be strategy or first-person".into()),
                    }
                }
                "--benchmark-output" => output = value,
                _ => {
                    return Err(format!(
                        "Unknown option {arg}. Options: --quality low|high, --benchmark SECONDS, --benchmark-camera strategy|first-person, --benchmark-output PATH"
                    ));
                }
            }
        }
        if let Some(seconds) = seconds {
            options.benchmark = Some(Benchmark {
                seconds,
                camera,
                output,
            });
        } else if camera != CameraMode::Strategy || output != "benchmark.csv" {
            return Err("benchmark camera/output require --benchmark".into());
        }
        Ok(options)
    }
}
#[derive(Debug)]
struct Summary {
    frames: usize,
    mean_ms: f64,
    p50_ms: f64,
    p95_ms: f64,
    p99_ms: f64,
    fps: f64,
}
fn summarize(samples: impl IntoIterator<Item = f64>) -> Option<Summary> {
    let mut samples: Vec<_> = samples
        .into_iter()
        .filter(|x| x.is_finite() && *x > 0.0)
        .collect();
    if samples.is_empty() {
        return None;
    }
    samples.sort_by(f64::total_cmp);
    let frames = samples.len();
    let mean_ms = samples.iter().sum::<f64>() / frames as f64;
    let percentile = |p: f64| samples[(p * frames as f64).ceil() as usize - 1];
    Some(Summary {
        frames,
        mean_ms,
        p50_ms: percentile(0.5),
        p95_ms: percentile(0.95),
        p99_ms: percentile(0.99),
        fps: 1000.0 / mean_ms,
    })
}
#[derive(Resource)]
struct FrameMetrics {
    recent: VecDeque<f64>,
    visible: bool,
    started: Instant,
    last: Instant,
    last_ui: Instant,
    measured: Vec<f64>,
    writer: Option<BufWriter<File>>,
    failed: bool,
}
#[derive(Component)]
struct PerformanceText;
pub struct PerformancePlugin;
impl Plugin for PerformancePlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, setup).add_systems(Last, measure);
    }
}
fn setup(mut commands: Commands, options: Res<RunOptions>, mut day: ResMut<Daylight>) {
    let mut failed = false;
    let writer = options.benchmark.as_ref().and_then(|bench| {
        // Fixed lighting isolates quality/camera comparisons from the day cycle.
        day.hour = 16.0;
        day.running = false;
        match File::create(&bench.output) {
            Ok(file) => {
                let mut writer = BufWriter::new(file);
                if let Err(error) = writeln!(writer, "frame,frame_ms,camera,quality,os,arch,build")
                {
                    eprintln!("Benchmark output: {error}");
                    failed = true;
                }
                Some(writer)
            }
            Err(error) => {
                eprintln!("Benchmark output: {error}");
                failed = true;
                None
            }
        }
    });
    let now = Instant::now();
    commands.insert_resource(FrameMetrics {
        recent: VecDeque::new(),
        visible: true,
        started: now,
        last: now,
        last_ui: now,
        measured: Vec::new(),
        writer,
        failed,
    });
    commands.spawn((
        Text::new("Preparing frame measurements…"),
        TextFont {
            font_size: FontSize::Px(14.0),
            ..default()
        },
        TextColor(Color::WHITE),
        BackgroundColor(Color::srgba(0.04, 0.07, 0.05, 0.85)),
        Node {
            position_type: PositionType::Absolute,
            right: px(12),
            top: px(340),
            padding: UiRect::all(px(8)),
            ..default()
        },
        PerformanceText,
    ));
}
fn measure(
    mut metrics: ResMut<FrameMetrics>,
    options: Res<RunOptions>,
    rig: Res<CameraRig>,
    day: Res<Daylight>,
    keys: Res<ButtonInput<KeyCode>>,
    mut text: Query<(&mut Text, &mut Node), With<PerformanceText>>,
    mut exit: MessageWriter<AppExit>,
) {
    if metrics.failed {
        exit.write(AppExit::error());
        return;
    }
    let now = Instant::now();
    let ms = now.duration_since(metrics.last).as_secs_f64() * 1000.0;
    metrics.last = now;
    if ms > 0.0 {
        metrics.recent.push_back(ms);
        if metrics.recent.len() > 240 {
            metrics.recent.pop_front();
        }
    }
    if keys.just_pressed(KeyCode::F3) {
        metrics.visible = !metrics.visible;
    }
    if let Ok((mut text, mut node)) = text.single_mut() {
        node.display = if metrics.visible {
            Display::Flex
        } else {
            Display::None
        };
        if now.duration_since(metrics.last_ui).as_secs_f64() >= 0.25
            && let Some(s) = summarize(metrics.recent.iter().copied())
        {
            metrics.last_ui = now;
            text.0 = format!(
                "{:02}:{:02} | {} | T: sun | Y: +3h | F3: stats\nCPU wall frame / {} samples\n{:.1} fps | mean {:.1} ms | p95 {:.1} ms",
                day.hour as u32,
                ((day.hour.fract() * 60.0) as u32).min(59),
                options.quality.label(),
                s.frames,
                s.fps,
                s.mean_ms,
                s.p95_ms
            );
        }
    }
    let Some(bench) = &options.benchmark else {
        return;
    };
    let elapsed = now.duration_since(metrics.started).as_secs_f64();
    // Ten seconds exclude startup/pipeline compilation from the recorded interval.
    if elapsed - ms / 1000.0 < 10.0 {
        return;
    }
    metrics.measured.push(ms);
    let index = metrics.measured.len();
    let mode = match rig.mode {
        CameraMode::Strategy => "strategy",
        CameraMode::FirstPerson => "first-person",
    };
    let build = if cfg!(debug_assertions) {
        "debug"
    } else {
        "release"
    };
    if let Some(writer) = &mut metrics.writer
        && let Err(error) = writeln!(
            writer,
            "{index},{ms:.6},{mode},{},{},{},{build}",
            options.quality.label(),
            std::env::consts::OS,
            std::env::consts::ARCH
        )
    {
        eprintln!("Benchmark output: {error}");
        metrics.failed = true;
    }
    if elapsed >= 10.0 + bench.seconds {
        if let Some(writer) = &mut metrics.writer
            && let Err(error) = writer.flush()
        {
            eprintln!("Benchmark flush: {error}");
            metrics.failed = true;
        }
        if let Some(s) = summarize(metrics.measured.iter().copied()) {
            println!(
                "BENCHMARK {mode} {} {build}: {} frames; {:.2} fps; mean {:.3} ms; p50 {:.3} ms; p95 {:.3} ms; p99 {:.3} ms. CPU wall frames, not GPU timings. CSV: {}",
                options.quality.label(),
                s.frames,
                s.fps,
                s.mean_ms,
                s.p50_ms,
                s.p95_ms,
                s.p99_ms,
                bench.output
            );
        } else {
            metrics.failed = true;
        }
        exit.write(if metrics.failed {
            AppExit::error()
        } else {
            AppExit::Success
        });
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    fn parse(args: &[&str]) -> Result<RunOptions, String> {
        RunOptions::parse(args.iter().map(|s| s.to_string()))
    }
    #[test]
    fn percentiles_include_slow_frames_and_ignore_invalid_samples() {
        let s = summarize([1.0, 2.0, 3.0, 4.0, 90.0, f64::NAN, 0.0]).unwrap();
        assert_eq!(s.frames, 5);
        assert_eq!(s.mean_ms, 20.0);
        assert_eq!(s.p50_ms, 3.0);
        assert_eq!(s.p95_ms, 90.0);
        assert_eq!(s.p99_ms, 90.0);
        assert_eq!(s.fps, 50.0);
        assert!(summarize([]).is_none());
    }
    #[test]
    fn benchmark_arguments_reject_invalid_or_ambiguous_runs() {
        for args in [
            vec!["--benchmark", "NaN"],
            vec!["--benchmark", "0"],
            vec!["--quality", "ultra"],
            vec!["--quality"],
            vec!["--benchmark-camera", "first-person"],
        ] {
            assert!(parse(&args).is_err());
        }
        let o = parse(&[
            "--benchmark",
            "30",
            "--quality",
            "low",
            "--benchmark-camera",
            "first-person",
        ])
        .unwrap();
        assert_eq!(o.quality, VisualQuality::Low);
        assert!(!o.quality.shadows());
        assert_eq!(o.benchmark.unwrap().camera, CameraMode::FirstPerson);
    }
}
