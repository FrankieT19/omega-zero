//! Drives the whole frontend headless through a fixed script and writes a PNG of every `shot`,
//! so a change to how slot draws can be checked pixel for pixel against the build before it.
//!
//! `screens OUT [BASELINE]` writes OUT/<name>.png. With BASELINE, every shot is also compared
//! with BASELINE/<name>.png and the run fails on the first byte that differs.

use std::path::{Path, PathBuf};
use std::time::Duration;

use slot::frontend::Frontend;
use slot::video_mode::{source_rect, VideoMode};
use slot_gfx::{out_h, out_w, Compositor, HeadlessSurface};
use slot_input::{Btn, InputSource, Millis, RawEvent};
use slot_power::SimPlatform;
use slot_store::{write_slot_state, Platform, SlotState};

const FPS: f64 = 60.0;

const CLOCK: i64 = 1_790_784_000;

const SETTLE: Duration = Duration::from_millis(400);

const LIT_FRAMES: u32 = 600;

const SCRIPT: &str = "
wait 120
settle
shot shelf-gba
tap right
wait 40
shot shelf-scrolled
tap left
wait 40
tap start
settle
wait 90
shot core-picker
tap b
wait 60
tap menu
wait 60
shot quick-menu
tap down
wait 10
tap down
wait 10
tap down
wait 10
tap down
wait 10
tap a
settle
wait 60
shot clock
tap b
wait 60
tap down
wait 10
tap a
settle
wait 90
look about
tap b
wait 60
tap b
wait 60
tap r1
wait 60
shot shelf-gb
tap l1
wait 60
tap a
wait 120
lit
wait 240
lit
look game-gba
tap volup
wait 12
look hud-volume
wait 180
down select
tap r1
up select
wait 60
look toast-saved
wait 180
down select
wait 2
tap menu
wait 2
up select
settle
wait 90
shot link
tap b
wait 120
tap menu 4
wait 4
tap menu 4
settle
wait 90
look switcher
tap b
wait 90
hold menu 80
wait 180
shot ejected
tap r1
wait 60
tap a
wait 120
lit
wait 240
lit
look game-gb
tap l1
wait 30
look game-gb-stretch
tap r1
wait 30
hold power 80
wait 60
shot power-menu
";

enum Step {
    Settle,
    Lit,
    Wait(u32),
    Down(Btn),
    Up(Btn),
    Shot(String),
    Look(String),
}

fn btn(name: &str) -> Btn {
    match name {
        "up" => Btn::Up,
        "down" => Btn::Down,
        "left" => Btn::Left,
        "right" => Btn::Right,
        "a" => Btn::A,
        "b" => Btn::B,
        "x" => Btn::X,
        "y" => Btn::Y,
        "l1" => Btn::L1,
        "r1" => Btn::R1,
        "l2" => Btn::L2,
        "r2" => Btn::R2,
        "start" => Btn::Start,
        "select" => Btn::Select,
        "menu" => Btn::Menu,
        "volup" => Btn::VolUp,
        "voldown" => Btn::VolDown,
        "power" => Btn::Power,
        other => panic!("unknown button {other}"),
    }
}

fn parse(script: &str) -> Vec<Step> {
    let mut steps = Vec::new();
    for line in script.lines() {
        let words: Vec<&str> = line.split('#').next().unwrap().split_whitespace().collect();
        let n = |i: usize, default: u32| words.get(i).map_or(default, |w| w.parse().unwrap());
        match words.as_slice() {
            [] => {}
            ["wait", ..] => steps.push(Step::Wait(n(1, 1))),
            ["settle"] => steps.push(Step::Settle),
            ["lit"] => steps.push(Step::Lit),
            ["tap", b, ..] | ["hold", b, ..] => {
                steps.push(Step::Down(btn(b)));
                steps.push(Step::Wait(n(2, 6)));
                steps.push(Step::Up(btn(b)));
            }
            ["down", b] => steps.push(Step::Down(btn(b))),
            ["up", b] => steps.push(Step::Up(btn(b))),
            ["shot", name] => steps.push(Step::Shot(name.to_string())),
            ["look", name] => steps.push(Step::Look(name.to_string())),
            _ => panic!("cannot read {line:?}"),
        }
    }
    steps
}

#[derive(Default)]
struct Scripted(Vec<RawEvent>);

impl InputSource for Scripted {
    fn poll(&mut self, _now: Millis) -> Vec<RawEvent> {
        std::mem::take(&mut self.0)
    }
}

fn tick(
    frontend: &mut Frontend,
    compositor: &mut Compositor,
    input: &mut Scripted,
    frame: &mut u64,
) {
    let loading = std::time::Instant::now();
    while frontend.core_settling() && loading.elapsed() < Duration::from_secs(5) {
        std::thread::sleep(Duration::from_millis(5));
    }
    *frame += 1;
    let now = (*frame as f64 * 1000.0 / FPS) as Millis;
    frontend.advance_at(input, now, (1.0 / FPS) as f32);
    frontend.step_emulator(Duration::from_secs_f64(1.0 / FPS), Duration::from_secs(1));
    frontend.compose(compositor);
}

fn card(dir: &Path) {
    for sub in slot::root::DIRS {
        std::fs::create_dir_all(dir.join(sub)).expect("create the card's folders");
    }
    for title in ["Emerald", "Fire Red", "Minish Cap", "Zelda"] {
        let mut rom = vec![0u8; 0x100];
        let header = title.to_uppercase();
        rom[0xa0..0xa0 + header.len().min(12)]
            .copy_from_slice(&header.as_bytes()[..header.len().min(12)]);
        std::fs::write(dir.join(format!("Games/GBA/{title}.gba")), rom).expect("write a GBA rom");
    }
    for title in ["Tetris", "Links Awakening"] {
        let mut rom = vec![0u8; 0x150];
        let header = title.to_uppercase();
        rom[0x134..0x134 + header.len().min(11)]
            .copy_from_slice(&header.as_bytes()[..header.len().min(11)]);
        std::fs::write(dir.join(format!("Games/GB/{title}.gb")), rom).expect("write a GB rom");
    }
    write_slot_state(
        dir,
        &SlotState {
            clock_set: true,
            ..Default::default()
        },
    )
    .expect("write slot.state");
}

/// The mock core's pattern moves with however many frames its thread ran, so every shot shows
/// this fixed picture in the game texture instead.
fn still_frame() -> Vec<u8> {
    let (w, h) = (slot_retro::GBA_W, slot_retro::GBA_H);
    let mut xrgb = Vec::with_capacity((w * h * 4) as usize);
    for y in 0..h {
        for x in 0..w {
            let check = if (x / 8 + y / 8) % 2 == 0 { 0xff } else { 0x30 };
            xrgb.extend_from_slice(&[
                check,
                (y * 255 / (h - 1)) as u8,
                (x * 255 / (w - 1)) as u8,
                0xff,
            ]);
        }
    }
    xrgb
}

fn write_png(path: &Path, rgba: &[u8]) {
    let file = std::fs::File::create(path).expect("create the png");
    let mut e = png::Encoder::new(std::io::BufWriter::new(file), out_w(), out_h());
    e.set_color(png::ColorType::Rgba);
    e.set_depth(png::BitDepth::Eight);
    e.write_header()
        .expect("png header")
        .write_image_data(rgba)
        .expect("png data");
}

fn read_png(path: &Path) -> Option<(u32, u32, Vec<u8>)> {
    let decoder = png::Decoder::new(std::io::BufReader::new(std::fs::File::open(path).ok()?));
    let mut reader = decoder.read_info().ok()?;
    let mut buf = vec![0; reader.output_buffer_size()];
    let info = reader.next_frame(&mut buf).ok()?;
    buf.truncate(info.buffer_size());
    Some((info.width, info.height, buf))
}

fn compare(name: &str, rgba: &[u8], baseline: &Path) -> Result<(), String> {
    let path = baseline.join(format!("{name}.png"));
    let Some((w, h, want)) = read_png(&path) else {
        return Err(format!("{name}: no baseline at {}", path.display()));
    };
    if (w, h) != (out_w(), out_h()) {
        return Err(format!(
            "{name}: baseline is {w}x{h}, the shot is {}x{}",
            out_w(),
            out_h()
        ));
    }
    let differ = rgba
        .chunks_exact(4)
        .zip(want.chunks_exact(4))
        .filter(|(a, b)| a != b)
        .count();
    match differ {
        0 => Ok(()),
        n => Err(format!("{name}: {n} pixels differ from the baseline")),
    }
}

fn main() {
    let args: Vec<PathBuf> = std::env::args().skip(1).map(PathBuf::from).collect();
    let (out, baseline) = match args.as_slice() {
        [out] => (out.clone(), None),
        [out, baseline] => (out.clone(), Some(baseline.clone())),
        _ => {
            eprintln!("usage: screens OUT [BASELINE]");
            std::process::exit(2);
        }
    };
    std::fs::create_dir_all(&out).expect("create the output folder");
    let root = tempfile::tempdir().expect("a scratch card");
    card(root.path());
    std::env::set_var("SLOT_SILENT", "1");
    std::env::remove_var("SLOT_CORE");

    let surface = HeadlessSurface::new().expect("headless GL");
    let mut compositor = Compositor::new(&surface).expect("compositor");
    let mut frontend = Frontend::boot(Box::new(
        SimPlatform::at(root.path().into()).stopped_at(CLOCK),
    ));
    frontend.upload_faces(&mut compositor);
    frontend.drive_emulator();

    let steps = parse(SCRIPT);
    let shots: Vec<String> = steps
        .iter()
        .filter_map(|s| match s {
            Step::Shot(name) => Some(name.clone()),
            _ => None,
        })
        .collect();
    let mut input = Scripted::default();
    let mut frame = 0u64;
    let mut failures = Vec::new();
    for step in steps {
        match step {
            Step::Wait(n) => {
                for _ in 0..n {
                    tick(&mut frontend, &mut compositor, &mut input, &mut frame);
                }
            }
            Step::Lit => {
                let mut left = LIT_FRAMES;
                while frontend.app().screen_power() < 1.0 && left > 0 {
                    tick(&mut frontend, &mut compositor, &mut input, &mut frame);
                    left -= 1;
                }
            }
            Step::Down(b) => input.0.push(RawEvent::Down(b)),
            Step::Up(b) => input.0.push(RawEvent::Up(b)),
            Step::Settle => {
                std::thread::sleep(SETTLE);
                frontend.compose(&mut compositor);
            }
            Step::Shot(name) | Step::Look(name) => {
                std::thread::sleep(SETTLE);
                frontend.compose(&mut compositor);
                compositor.upload_game(&still_frame());
                frontend.compose(&mut compositor);
                let rgba = compositor.read_frame();
                write_png(&out.join(format!("{name}.png")), &rgba);
                if let (Some(baseline), true) = (&baseline, shots.contains(&name)) {
                    if let Err(e) = compare(&name, &rgba, baseline) {
                        failures.push(e);
                    }
                }
            }
        }
    }
    for platform in [Platform::Gba, Platform::Gb] {
        for mode in [VideoMode::Actual, VideoMode::Stretch] {
            for power in [0.05, 0.5, 1.0] {
                let name = format!("view-{}-{}-{power}", platform.dir_name(), mode.as_str());
                compositor.upload_game(&still_frame());
                compositor.set_game_source_rect(source_rect(platform, mode));
                compositor.set_screen_power(power);
                compositor.begin_frame();
                compositor.draw_game();
                let rgba = compositor.read_frame();
                write_png(&out.join(format!("{name}.png")), &rgba);
                if let Some(baseline) = &baseline {
                    if let Err(e) = compare(&name, &rgba, baseline) {
                        failures.push(e);
                    }
                }
            }
        }
    }
    if failures.is_empty() {
        eprintln!("screens: wrote {}", out.display());
        return;
    }
    for f in &failures {
        eprintln!("screens: {f}");
    }
    std::process::exit(1);
}
