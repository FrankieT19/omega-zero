//! Headless hotkey/power companion. Neither ALSA nor EGL is opened in this process.
use std::io::{BufRead, BufReader, Write};
use std::os::unix::process::CommandExt;
use std::process::{Child, ChildStdin, Command, Stdio};
use std::sync::mpsc;
use std::time::{Duration, Instant};

use omega_input::{Action, Btn, Gestures, InputSource, Keys, RawEvent};
use omega_power::{Charge, DevicePlatform, LedState, LidPolicy, Power};
use omega_store::{
    read_app_state, write_app_state, AppState, Core, Platform, GB_PALETTES, SHADER_NAMES,
};

use super::{dim_gain, Launch};
use crate::input::{DeviceInput, Pad};

pub fn enter(launch: Launch) -> ! {
    let exe = std::env::current_exe().expect("frontend executable path");
    crate::spawn::keep_files_to_self();
    let error = Command::new(&exe)
        .arg("--retroarch-session")
        .arg(&launch.root)
        .arg(&launch.rom)
        .arg((launch.platform as usize).to_string())
        .arg(launch.core.as_str())
        .arg(if launch.clean { "1" } else { "0" })
        .arg(if launch.menu { "1" } else { "0" })
        .exec();
    eprintln!("omega-zero: RetroArch handoff: {error}");
    std::process::exit(1);
}

pub fn run(args: Vec<String>) {
    let launch = match args.as_slice() {
        [root, rom, platform, core, clean, menu] => platform
            .parse::<usize>()
            .ok()
            .and_then(|i| Platform::ALL.get(i))
            .copied()
            .zip(Core::parse(core))
            .map(|(platform, core)| Launch {
                root: root.into(),
                rom: rom.into(),
                platform,
                core,
                clean: clean == "1",
                menu: menu == "1",
            }),
        _ => None,
    };
    let Some(launch) = launch else {
        eprintln!("omega-zero: invalid RetroArch session arguments");
        return;
    };
    let mut state = read_app_state(&launch.root);
    let hover = (state.last_cart.clone(), state.last_platform);
    let result = play(&launch, &mut state);
    if launch.menu {
        state.last_cart = hover.0;
        state.last_platform = hover.1;
    }
    if let Err(error) = result {
        eprintln!("omega-zero: RetroArch: {error}");
    }
    // Normal exit or failed startup returns to the same hover/carousel, through a fresh
    // process: no game GL context, held key, audio thread or timing state can leak back.
    state.cart = None;
    state.cart_platform = None;
    if let Err(e) = write_app_state(&launch.root, &state) {
        eprintln!("omega-zero: save menu state: {e}");
    }
    crate::persist::clear_starting(&launch.root);
    crate::spawn::keep_files_to_self();
    let error = Command::new(std::env::current_exe().expect("frontend executable path")).exec();
    eprintln!("omega-zero: reopen carousel: {error}");
}

struct Player {
    child: Child,
    commands: ChildStdin,
}
impl Player {
    fn send(&mut self, command: impl AsRef<str>) -> std::io::Result<()> {
        writeln!(self.commands, "{}", command.as_ref())
    }
    fn param(&mut self, name: &str, value: f32) -> std::io::Result<()> {
        self.send(format!("OMEGA_PARAM {name} {value:.4}"))
    }
    fn message(&mut self, text: &str) -> std::io::Result<()> {
        self.send(format!("OMEGA_MSG {text}"))
    }
    fn finish(&mut self) -> std::io::Result<()> {
        self.send("OMEGA_INPUT 0 0\nSET_OPTION pause off\nQUIT")?;
        let limit = Instant::now() + Duration::from_secs(8);
        while Instant::now() < limit {
            if self.child.try_wait()?.is_some() {
                return Ok(());
            }
            std::thread::sleep(Duration::from_millis(20));
        }
        self.child.kill()?;
        self.child.wait()?;
        eprintln!("omega-zero: RetroArch did not quit within eight seconds; terminated it before returning/shutdown");
        Ok(())
    }
}
impl Drop for Player {
    fn drop(&mut self) {
        if self.child.try_wait().ok().flatten().is_none() {
            let _ = self.child.kill();
            let _ = self.child.wait();
        }
    }
}
fn keep(launch: &Launch, state: &AppState) -> std::io::Result<()> {
    write_app_state(&launch.root, state)
}
fn volume(player: &mut Player, power: &Power, state: &AppState) -> std::io::Result<()> {
    let (v, muted) = if power.headphones() {
        (state.volume_hp, state.muted_hp)
    } else {
        (state.volume, state.muted)
    };
    let db = if muted || v == 0 {
        -80.0
    } else {
        40.0 * (v as f32 / 100.0).log10()
    };
    player.send(format!("OMEGA_VOLUME {db:.3}"))
}

fn runtime_libraries() -> std::io::Result<std::ffi::OsString> {
    use std::os::unix::fs::symlink;
    let aliases = std::path::PathBuf::from("/run/omega-retroarch-libs");
    std::fs::create_dir_all(&aliases)?;
    // BaseOS images may supply the vendor libraries with unversioned names. Resolve
    // aliases inside our own config folder, leaving the OS library tree untouched.
    let mut search = std::env::var_os("LD_LIBRARY_PATH")
        .map(|v| std::env::split_paths(&v).collect::<Vec<_>>())
        .unwrap_or_default();
    search.extend(
        [
            "/lib",
            "/usr/lib",
            "/lib64",
            "/usr/lib64",
            "/lib/aarch64-linux-gnu",
            "/usr/lib/aarch64-linux-gnu",
        ]
        .map(std::path::PathBuf::from),
    );
    for (names, candidates) in [
        (
            ["libEGL.so.1", "libEGL.so"],
            ["libEGL.so.1", "libEGL.so", "libMali.so"],
        ),
        (
            ["libGLESv2.so.2", "libGLESv2.so"],
            ["libGLESv2.so.2", "libGLESv2.so", "libMali.so"],
        ),
    ] {
        if let Some(source) = candidates
            .into_iter()
            .flat_map(|name| search.iter().map(move |dir| dir.join(name)))
            .find(|p| p.is_file())
            .and_then(|p| p.canonicalize().ok())
        {
            for name in names {
                let target = aliases.join(name);
                if std::fs::symlink_metadata(&target).is_ok_and(|m| m.file_type().is_symlink()) {
                    std::fs::remove_file(&target)?;
                }
                if !target.exists() {
                    symlink(&source, &target)?;
                }
            }
        }
    }
    let mut paths = vec![aliases];
    paths.extend(search);
    std::env::join_paths(paths).map_err(std::io::Error::other)
}

fn play(launch: &Launch, state: &mut AppState) -> std::io::Result<()> {
    launch.prepare(state)?;
    state.cart = Some(launch.stem());
    state.cart_platform = Some(launch.platform);
    state.last_cart = state.cart.clone();
    state.last_platform = state.cart_platform;
    keep(launch, state)?;
    crate::persist::mark_starting(&launch.root, launch.platform, &launch.stem());
    let log = std::fs::File::create(launch.root.join("Config/retroarch/retroarch.log"))?;
    let mut command = Command::new(launch.root.join("System/retroarch"));
    command
        .env("LD_LIBRARY_PATH", runtime_libraries()?)
        .env("OMEGA_DISPLAY_DIM", dim_gain(state).to_string())
        .env(
            "OMEGA_DISPLAY_BLUE",
            (state.blue_light as f32 / 9.0).to_string(),
        )
        .arg("--config")
        .arg(launch.directory().join("current.cfg"))
        .arg("-L")
        .arg(
            launch
                .root
                .join("System")
                .join(format!("{}_libretro.so", launch.core.library())),
        )
        .arg(&launch.rom)
        .arg("--verbose")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(log);
    crate::spawn::keep_files_to_self();
    let mut child = command.spawn()?;
    let commands = child.stdin.take().expect("piped stdin");
    let output = child.stdout.take().expect("piped stdout");
    let (tx, rx) = mpsc::channel();
    std::thread::spawn(move || {
        for line in BufReader::new(output).lines().map_while(Result::ok) {
            if tx.send(line).is_err() {
                break;
            }
        }
    });
    let mut player = Player { child, commands };
    let mut power = Power::new(
        Box::new(DevicePlatform::new(launch.root.clone())),
        Duration::from_secs(180),
    );
    power.set_backlight(state.brightness.max(1));
    power.set_led(LedState::Running);
    let mut input = DeviceInput::open(&launch.root);
    let mut gestures = Gestures::new();
    let keys = Keys {
        game_l2_r2: launch.platform.game_owns_l2_r2(),
        rewind: launch.platform.rewinds(),
        colour: launch.platform.colour_correction(),
    };
    gestures.set_keys(keys);
    let mut pad = Pad::default();
    pad.set_keys(keys);
    let began = Instant::now();
    let mut last = began;
    let mut query = began - Duration::from_secs(1);
    let mut ready = false;
    let mut initialized = false;
    let mut shader_ready = false;
    let mut content_at = None;
    let mut held = 0u64;
    let mut idle = crate::inactivity::Idle::default();
    let mut fade = 1.0f32;
    let mut last_gain = -1.0f32;
    let mut sleeping = false;
    let mut woke_with_power = false;
    let mut closed_at = None;
    let mut lid = false;
    let mut want_sleep = false;
    let mut fast = false;
    let mut rewind = false;
    let mut menu_open = false;
    let mut sent_input = (u16::MAX, u8::MAX);
    let mut headphones = power.headphones();
    let mut next_headphones = 0u64;
    let mut next_battery = 0u64;
    let i = launch.platform as usize;
    loop {
        if let Some(status) = player.child.try_wait()? {
            if !status.success() {
                return Err(std::io::Error::other(format!(
                    "process exited {status}; see Config/retroarch/retroarch.log"
                )));
            }
            break;
        }
        let now = Instant::now();
        let ms = began.elapsed().as_millis() as u64;
        let dt = now.duration_since(last).as_secs_f32().min(0.05);
        last = now;
        while let Ok(line) = rx.try_recv() {
            if line.starts_with("GET_STATUS PLAYING") || line.starts_with("GET_STATUS PAUSED") {
                if !ready {
                    content_at = Some(now);
                }
                ready = true;
            }
            if line.starts_with("OMEGA_SHADER ")
                && line
                    .split_whitespace()
                    .nth(1)
                    .and_then(|s| s.parse::<u32>().ok())
                    .is_some_and(|n| n > 0)
                && !shader_ready
            {
                shader_ready = true;
                std::fs::write(
                    launch.root.join("Config/retroarch/shader-status.txt"),
                    &line,
                )?;
            }
            if line.starts_with("menu\t") {
                menu_open = line.trim_end().ends_with("on");
            }
            if let Some(value) = line
                .strip_prefix("OMEGA_RUMBLE ")
                .and_then(|v| v.parse::<u16>().ok())
            {
                power.set_rumble(if state.rumble { value } else { 0 });
            }
        }
        if ready && now.duration_since(query) > Duration::from_millis(250) {
            player.send(if shader_ready {
                "GET_OPTION menu\nOMEGA_SCREEN"
            } else {
                "GET_OPTION menu\nOMEGA_SCREEN\nOMEGA_SHADER"
            })?;
            query = now;
        }
        if ready
            && !shader_ready
            && content_at.is_some_and(|at| now.duration_since(at) > Duration::from_secs(15))
        {
            return Err(std::io::Error::other("RetroArch loaded the game but no shader preset; see Config/retroarch/retroarch.log"));
        }
        if !ready {
            if now.duration_since(query) > Duration::from_millis(250) {
                player.send("GET_STATUS")?;
                query = now;
            }
            if began.elapsed() > Duration::from_secs(60) {
                return Err(std::io::Error::new(std::io::ErrorKind::TimedOut,"RetroArch did not load the game within a minute; see Config/retroarch/retroarch.log"));
            }
        } else if !initialized {
            initialized = true;
            if launch.menu {
                player.send("SET_OPTION menu on")?;
                menu_open = true;
            }
            crate::persist::clear_starting(&launch.root);
            let key = format!("{}/{}", launch.platform.dir_name(), launch.stem());
            state.recent.retain(|v| v != &key);
            state.recent.insert(0, key);
            state.recent.truncate(10);
            keep(launch, state)?;
            player.param("OMEGA_BLUE", state.blue_light as f32 / 9.0)?;
            player.send("OMEGA_SCREEN")?;
            volume(&mut player, &power, state)?;
            idle.touch(ms);
        }
        let mut actions = Vec::new();
        for event in input.poll(ms) {
            let (button, down) = match event {
                RawEvent::Down(b) => (b, true),
                RawEvent::Up(b) => (b, false),
            };
            let bit = 1u64 << button as usize;
            if button != Btn::Lid && down && held & bit != 0 {
                continue;
            }
            if down {
                held |= bit;
            } else {
                held &= !bit;
            }
            if button != Btn::Lid {
                idle.touch(ms);
            }
            if sleeping && down && button == Btn::Power {
                sleeping = false;
                woke_with_power = true;
                want_sleep = false;
                closed_at = None;
                idle.touch(ms);
                power.on_open();
                player.send("SET_OPTION pause off")?;
                gestures = Gestures::new();
                gestures.set_keys(keys);
                pad.clear();
            }
            actions.extend(gestures.feed(event, ms));
        }
        actions.extend(gestures.tick(ms));
        if held & !(1u64 << Btn::Lid as usize) != 0 {
            idle.touch(ms);
        }
        for action in actions {
            if !ready
                && !matches!(
                    action,
                    Action::LidClose | Action::LidOpen | Action::PowerHold | Action::PowerOff
                )
            {
                continue;
            }
            if sleeping
                && !matches!(
                    action,
                    Action::LidOpen | Action::PowerTap | Action::PowerHold | Action::PowerOff
                )
            {
                continue;
            }

            pad.apply(action);
            match action {
                Action::Eject => {
                    player.finish()?;
                    launch.collect_battery()?;
                    power.set_rumble(0);
                    return Ok(());
                }
                Action::PowerOff | Action::PowerHold => {
                    keep(launch, state)?;
                    player.finish()?;
                    launch.collect_battery()?;
                    power.set_rumble(0);
                    power.poweroff();
                }
                Action::LidClose => {
                    lid = true;
                    want_sleep = true;
                    fast = false;
                    rewind = false;
                    pad.clear();
                }
                Action::LidOpen => {
                    lid = false;
                    want_sleep = false;
                    sleeping = false;
                    closed_at = None;
                    idle.touch(ms);
                    power.on_open();
                    player.send("SET_OPTION pause off")?;
                }
                Action::PowerTap if woke_with_power => {
                    woke_with_power = false;
                }
                Action::PowerTap if !lid => {
                    want_sleep = !want_sleep;
                    if !want_sleep {
                        sleeping = false;
                        closed_at = None;
                        idle.touch(ms);
                        power.on_open();
                        player.send("SET_OPTION pause off")?;
                    }
                }
                Action::FfStart => fast = true,
                Action::FfStop => fast = false,
                Action::RewindStart => rewind = true,
                Action::RewindStop => rewind = false,
                Action::GbaDown(Btn::L1 | Btn::R1)
                    if launch.platform.owns_shoulders() && !menu_open =>
                {
                    state.scales[i] = if action == Action::GbaDown(Btn::L1) {
                        if state.scales[i] == 1 {
                            0
                        } else {
                            1
                        }
                    } else {
                        if state.scales[i] == 2 {
                            0
                        } else {
                            2
                        }
                    };
                    let scaling = state.scales[i];
                    let aspect = if scaling == 2 { 24 } else { 21 };
                    player.send(format!("OMEGA_SCALE {scaling} {aspect}\nOMEGA_SCREEN"))?;
                    player.message(["Integer", "Fit", "Stretch"][scaling as usize])?;
                    keep(launch, state)?;
                }
                Action::SaveState => player.send("SAVE_STATE")?,
                Action::LoadState => player.send("LOAD_STATE")?,
                Action::Polaroids | Action::GameMenu | Action::QuickMenu | Action::PictureMenu => {
                    pad.clear();
                    player.send("MENU_TOGGLE")?;
                }
                Action::ShaderNext => {
                    state.shaders[i] = (state.shaders[i] + 1) % 8;
                    let mode = state.shaders[i];
                    let preset = launch.write_preset(state, mode)?;
                    player.send(format!("SET_SHADER {}", preset.display()))?;
                    player.message(SHADER_NAMES[mode as usize])?;
                    keep(launch, state)?;
                    last_gain = -1.0;
                }
                Action::PaletteNext
                    if launch.platform == Platform::Gb && launch.core == Core::Gambatte =>
                {
                    state.palettes[i] = (state.palettes[i] + 1) % GB_PALETTES.len() as u8;
                    for (key, value) in crate::core::gb_palette_options(state.palettes[i]) {
                        player.send(format!("SET_CORE_OPTION {key} {value}"))?;
                    }
                    player.message(GB_PALETTES[state.palettes[i] as usize])?;
                    keep(launch, state)?;
                }
                Action::ColourCorrectionToggle => {
                    state.colour_correction = !state.colour_correction;
                    if let Some((key, value)) =
                        crate::core::colour_option(launch.core, state.colour_correction)
                    {
                        player.send(format!("SET_CORE_OPTION {key} {value}"))?;
                    }
                    keep(launch, state)?;
                }
                Action::BrightnessUp | Action::BrightnessDown => {
                    if action == Action::BrightnessDown {
                        if state.brightness > 1 {
                            state.brightness -= 1;
                        } else {
                            state.dim = (state.dim + 1).min(4);
                        }
                    } else if state.dim > 0 {
                        state.dim -= 1;
                    } else {
                        state.brightness = (state.brightness + 1).min(9);
                    }
                    power.set_backlight(state.brightness.max(1));
                    player.message(&format!(
                        "Brightness {}/13",
                        state.brightness.max(1) + 4 - state.dim
                    ))?;
                    keep(launch, state)?;
                }
                Action::BlueLightDown | Action::BlueLightUp => {
                    state.blue_light = if action == Action::BlueLightDown {
                        (state.blue_light + 1) % 10
                    } else {
                        (state.blue_light + 9) % 10
                    };
                    player.param("OMEGA_BLUE", state.blue_light as f32 / 9.0)?;
                    player.message(&format!("Blue light {}/9", state.blue_light))?;
                    keep(launch, state)?;
                }
                Action::VolumeUp | Action::VolumeDown | Action::MuteToggle => {
                    let (v, muted) = if power.headphones() {
                        (&mut state.volume_hp, &mut state.muted_hp)
                    } else {
                        (&mut state.volume, &mut state.muted)
                    };
                    match action {
                        Action::VolumeUp => {
                            *v = v.saturating_add(5).min(100);
                            *muted = false;
                        }
                        Action::VolumeDown => {
                            *v = v.saturating_sub(5);
                        }
                        _ => *muted = !*muted,
                    }
                    volume(&mut player, &power, state)?;
                    keep(launch, state)?;
                }
                _ => {}
            }
        }
        if ready {
            if ms >= next_headphones {
                next_headphones = ms + 500;
                let plugged = power.headphones();
                if headphones != plugged {
                    headphones = plugged;
                    volume(&mut player, &power, state)?;
                }
            }
            if ms >= next_battery {
                next_battery = ms + 10_000;
                if let Some(battery) = power.battery() {
                    if battery.percent <= 5 && battery.charge == Charge::Discharging {
                        keep(launch, state)?;
                        player.finish()?;
                        launch.collect_battery()?;
                        power.set_rumble(0);
                        power.poweroff();
                    }
                    power.set_led(match battery.charge {
                        Charge::Charging => LedState::Charging,
                        Charge::Full => LedState::Charged,
                        _ if battery.percent <= 20 => LedState::Low,
                        _ => LedState::Running,
                    });
                }
            }
            if want_sleep {
                fade = (fade - dt).max(0.0);
                idle.gain = fade;
            } else if !sleeping {
                let timeout = idle.step(ms, dt, 1, state.idle_dim[1], state.idle_sleep[1]);
                fade = idle.gain;
                if timeout {
                    want_sleep = true;
                }
            }
            let gain = fade * dim_gain(state);
            if (gain - last_gain).abs() > 0.0005 {
                player.param("OMEGA_DIM", gain)?;
                last_gain = gain;
            }
            if want_sleep && !sleeping && fade <= 0.001 {
                player.send("OMEGA_INPUT 0 0\nSET_OPTION pause on\nSAVE_STATE_SLOT -1")?;
                power.set_rumble(0);
                power.on_close();
                sleeping = true;
                closed_at = Some(now);
            }
            if sleeping && closed_at.is_some_and(|at| now.duration_since(at) >= power.timeout()) {
                keep(launch, state)?;
                player.finish()?;
                launch.collect_battery()?;
                power.poweroff();
            }
            let mut mask = pad.mask().0;
            // X/Y turbo is a frontend feature on the Game Boys; the console buttons remain
            // ordinary buttons. RetroArch's menu receives the original four face buttons.
            if launch.platform.turbo_xy() && !menu_open {
                let pulse = (ms / 50).is_multiple_of(2);
                if mask & omega_retro::ButtonMask::X != 0 && pulse {
                    mask |= omega_retro::ButtonMask::A;
                }
                if mask & omega_retro::ButtonMask::Y != 0 && pulse {
                    mask |= omega_retro::ButtonMask::B;
                }
            }
            if sleeping {
                mask = 0;
            }
            let flags = if sleeping {
                0
            } else {
                u8::from(fast) | u8::from(rewind) << 1
            };
            if sent_input != (mask, flags) {
                player.send(format!("OMEGA_INPUT {mask:x} {flags}"))?;
                sent_input = (mask, flags);
            }
        }
        std::thread::sleep(Duration::from_millis(5));
    }
    launch.collect_battery()?;
    power.set_rumble(0);
    Ok(())
}
