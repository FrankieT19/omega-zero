//! Process boundary between the carousel and RetroArch's unmodified Slang renderer.
//! Configuration and saves live separately so switching back to the native backend is safe.
use std::path::{Path, PathBuf};

use omega_store::{AppState, Core, Platform};

#[cfg(not(feature = "host"))]
mod device;
#[cfg(not(feature = "host"))]
pub use device::{enter, run};

#[derive(Clone, Debug)]
pub struct Launch {
    pub root: PathBuf,
    pub rom: PathBuf,
    pub platform: Platform,
    pub core: Core,
    pub clean: bool,
    pub menu: bool,
}

pub fn requested(root: &Path) -> bool {
    std::fs::read_to_string(root.join("Config/backend.txt")).map_or(true, |s| s.trim() != "native")
}

pub fn enabled(root: &Path) -> bool {
    cfg!(not(feature = "host")) && root.join("System/retroarch").is_file() && requested(root)
}

impl Launch {
    pub fn stem(&self) -> String {
        self.rom
            .file_stem()
            .unwrap_or_default()
            .to_string_lossy()
            .into_owned()
    }
    pub fn directory(&self) -> PathBuf {
        self.root
            .join("Config/retroarch")
            .join(self.platform.dir_name())
    }
    pub fn saves(&self) -> PathBuf {
        self.root
            .join("Saves/RetroArch")
            .join(self.platform.dir_name())
    }
    pub fn states(&self) -> PathBuf {
        self.root
            .join("States/RetroArch")
            .join(self.platform.dir_name())
            .join(self.core.library())
    }
    pub fn config(&self, state: &AppState) -> String {
        let dir = self.directory();
        let system = self.root.join("System");
        let i = self.platform as usize;
        let scaling = if self.platform == Platform::Gba {
            0
        } else {
            state.scales[i]
        };
        // Custom per-game scaling remains PS1-only; SNES always uses its system setting.
        let scaling = if matches!(self.platform, Platform::Gb | Platform::Gbc | Platform::Ps1) {
            omega_store::ini::value(
                &self.root,
                "Config/picture.ini",
                &format!("{}/{}", self.platform.dir_name(), self.stem()),
            )
            .as_deref()
            .and_then(crate::picture::Picture::parse)
            .and_then(|p| p.scaling)
            .unwrap_or(scaling)
        } else {
            scaling
        };
        let aspect = match scaling {
            2 => 24, // ASPECT_RATIO_FULL in the pinned RetroArch source.
            1 if matches!(self.platform, Platform::Ps1 | Platform::Snes) => 0, // 4:3
            _ => 21, // square source pixels (integer mode, or aspect-preserving GB Fit)
        };
        let mut s = String::new();
        for (key, value) in [
            ("video_driver", "glcore"),
            ("video_context_driver", "mali_fbdev"),
            ("audio_driver", "alsa"),
            ("audio_device", "default"),
            ("input_driver", "omega"),
            ("input_joypad_driver", "omega"),
            ("menu_driver", "rgui"),
            ("video_fullscreen", "true"),
            ("video_threaded", "false"),
            ("video_vsync", "true"),
            ("video_smooth", "false"),
            ("video_scale_integer_axis", "1"),
            ("video_shader_enable", "true"),
            ("input_overlay_enable", "false"),
            ("video_crop_overscan", "true"),
            ("stdin_cmd_enable", "true"),
            ("network_cmd_enable", "false"),
            ("config_save_on_exit", "false"),
            ("confirm_quit", "false"),
            ("savestate_auto_save", "true"),
            ("savestate_file_compression", "false"),
            ("block_sram_overwrite", "true"),
            ("sort_savefiles_enable", "false"),
            ("sort_savestates_enable", "false"),
            ("savefiles_in_content_dir", "false"),
            ("savestates_in_content_dir", "false"),
            ("savestate_thumbnail_enable", "true"),
            ("menu_pause_libretro", "true"),
            ("menu_mouse_enable", "false"),
            ("menu_pointer_enable", "false"),
            ("video_font_enable", "true"),
            ("video_font_size", "24"),
            ("notification_show_autoconfig", "false"),
            ("audio_sync", "true"),
            ("autosave_interval", "10"),
            ("rewind_buffer_size", "32"),
        ] {
            put(&mut s, key, value);
        }
        put(
            &mut s,
            "video_scale_integer",
            if scaling == 0 { "true" } else { "false" },
        );
        put(&mut s, "aspect_ratio_index", &aspect.to_string());
        put(
            &mut s,
            "rewind_enable",
            if self.platform.rewinds() {
                "true"
            } else {
                "false"
            },
        );
        put(&mut s, "fastforward_ratio", &state.ff_speed.to_string());
        put(
            &mut s,
            "audio_fastforward_mute",
            if state.ff_sound { "false" } else { "true" },
        );
        put(
            &mut s,
            "savestate_auto_load",
            if self.clean { "false" } else { "true" },
        );
        for (key, path) in [
            ("system_directory", self.root.join("BIOS")),
            ("cache_directory", self.root.join("Config/retroarch/cache")),
            ("savefile_directory", self.saves()),
            ("savestate_directory", self.states()),
            ("screenshot_directory", self.root.join("Screenshots")),
            (
                "core_options_path",
                dir.join(format!("{}.opt", self.core.library())),
            ),
            ("video_shader_dir", system.join("retroarch-shaders")),
            (
                "video_shader",
                if dir.join("custom.slangp").is_file() {
                    dir.join("custom.slangp")
                } else {
                    dir.join("current.slangp")
                },
            ),
            ("rgui_config_directory", dir.clone()),
        ] {
            put(&mut s, key, &path.to_string_lossy());
        }
        put(&mut s, "input_player1_joypad_index", "0");
        for (n, key) in [
            "b", "y", "select", "start", "up", "down", "left", "right", "a", "x", "l", "r", "l2",
            "r2", "l3", "r3",
        ]
        .into_iter()
        .enumerate()
        {
            put(&mut s, &format!("input_player1_{key}_btn"), &n.to_string());
        }
        s
    }
    pub fn prepare(&self, state: &AppState) -> std::io::Result<()> {
        let dir = self.directory();
        for path in [
            dir.clone(),
            self.saves(),
            self.states(),
            self.root.join("Config/retroarch/cache"),
            self.root.join("Config/retroarch/xdg"),
        ] {
            std::fs::create_dir_all(path)?;
        }
        omega_store::atomic_write(&dir.join("current.cfg"), self.config(state).as_bytes())?;
        let option_path = dir.join(format!("{}.opt", self.core.library()));
        let mut options = std::fs::read_to_string(&option_path).unwrap_or_default();
        for (k, v) in crate::core::core_options(
            self.core,
            "disabled",
            self.root.join("BIOS/gba_bios.bin").is_file(),
            state.colour_correction,
        ) {
            options = omega_store::ini::set(&options, k, Some(&format!("\"{v}\"")))?;
        }
        if self.platform == Platform::Gb && self.core == Core::Gambatte {
            for (k, v) in crate::core::gb_palette_options(state.palettes[self.platform as usize]) {
                options = omega_store::ini::set(&options, k, Some(&format!("\"{v}\"")))?;
            }
        }
        omega_store::atomic_write(
            &dir.join(format!("{}.opt", self.core.library())),
            options.as_bytes(),
        )?;
        // Preserve the original battery save before allowing a different frontend to write it.
        if let Some(bytes) = crate::persist::read_sav(&self.root, self.platform, &self.stem()) {
            let backup = self
                .root
                .join("Saves")
                .join(self.platform.dir_name())
                .join(format!("{}.pre-retroarch.sav", self.stem()));
            use std::io::Write;
            match std::fs::OpenOptions::new()
                .write(true)
                .create_new(true)
                .open(backup)
            {
                Ok(mut f) => {
                    f.write_all(&bytes)?;
                    f.sync_all()?;
                }
                Err(e) if e.kind() == std::io::ErrorKind::AlreadyExists => {}
                Err(e) => return Err(e),
            }
            let srm = self.saves().join(format!("{}.srm", self.stem()));
            let sav = self
                .root
                .join("Saves")
                .join(self.platform.dir_name())
                .join(format!("{}.sav", self.stem()));
            let newer = srm
                .metadata()
                .and_then(|m| m.modified())
                .ok()
                .zip(sav.metadata().and_then(|m| m.modified()).ok())
                .is_some_and(|(a, b)| a >= b);
            if !newer {
                omega_store::atomic_write(&srm, &bytes)?;
            }
        }
        for mode in 0..8 {
            self.write_preset(state, mode)?;
        }
        self.write_preset(state, state.shaders[self.platform as usize])?;
        Ok(())
    }
    pub fn write_preset(&self, state: &AppState, mode: u8) -> std::io::Result<PathBuf> {
        let path = self.directory().join(format!("mode-{mode}.slangp"));
        let text = self.preset(state, mode);
        omega_store::atomic_write(&path, text.as_bytes())?;
        omega_store::atomic_write(&self.directory().join("current.slangp"), text.as_bytes())?;
        Ok(path)
    }
    pub fn preset(&self, state: &AppState, mode: u8) -> String {
        let assets = self.root.join("System/retroarch-shaders");
        let mut passes = Vec::new();
        match mode {
            1 | 5 => passes.push("lcd1x.slang"),
            2 | 6 => passes.push("lcd3x.slang"),
            _ => {}
        }
        if matches!(mode, 4..=7) {
            passes.push("pixel_transparency/pixel_transparency.slang");
        }
        // The last pass does neutral colour controls, optional source cropping and the
        // screen raster. Unmodified LCD shaders run in RetroArch with original uniforms.
        passes.push("screen.slang");
        let mut s = String::new();
        put(&mut s, "shaders", &passes.len().to_string());
        for (n, name) in passes.into_iter().enumerate() {
            put(
                &mut s,
                &format!("shader{n}"),
                &assets.join(name).to_string_lossy(),
            );
            put(&mut s, &format!("filter_linear{n}"), "false");
            put(&mut s, &format!("scale_type{n}"), "viewport");
            put(&mut s, &format!("scale{n}"), "1.0");
            put(&mut s, &format!("srgb_framebuffer{n}"), "false");
            put(&mut s, &format!("float_framebuffer{n}"), "false");
        }
        let i = self.platform as usize;
        for (k, v) in [
            ("OMEGA_GRID", if matches!(mode, 3 | 7) { 1.0 } else { 0.0 }),
            ("OMEGA_SATURATION", state.saturation[i] as f32 / 100.0),
            ("PT_SHADOW_OFFSET_X", state.shadow_x[i] as f32 / 2.0),
            ("PT_SHADOW_OFFSET_Y", state.shadow_y[i] as f32 / 2.0),
            ("PT_SHADOW_MOTION", 0.0),
            ("PT_ACCEL_ENABLE", 0.0),
            // The upstream Pixel Transparency preset's parameter values.
            ("PT_PALETTE", 1.0),
            ("PT_POLARIZER", 0.0),
            ("PT_HIGHLIGHTS", 0.0),
            ("PT_DEFER_DITHER", 0.0),
            (
                "BRIGHTEN_SCANLINES",
                if state.lcd_darkness[i] == 0 {
                    32.0
                } else {
                    (1600.0 / state.lcd_darkness[i] as f32).clamp(1.0, 32.0)
                },
            ),
            (
                "BRIGHTEN_LCD",
                if state.lcd_darkness[i] == 0 {
                    12.0
                } else {
                    (400.0 / state.lcd_darkness[i] as f32).clamp(1.0, 12.0)
                },
            ),
        ] {
            put(&mut s, k, &format!("{v:.4}"));
        }
        // LCD3x uses lower-case parameter IDs; LCD1x uses upper-case IDs.
        let dark = state.lcd_darkness[i] as f32;
        put(
            &mut s,
            "brighten_scanlines",
            &format!(
                "{:.4}",
                if dark == 0.0 {
                    32.0
                } else {
                    (1600.0 / dark).clamp(1.0, 32.0)
                }
            ),
        );
        put(
            &mut s,
            "brighten_lcd",
            &format!(
                "{:.4}",
                if dark == 0.0 {
                    12.0
                } else {
                    (400.0 / dark).clamp(1.0, 12.0)
                }
            ),
        );
        if let Some(p) = omega_store::ini::value(
            &self.root,
            "Config/picture.ini",
            &format!("{}/{}", self.platform.dir_name(), self.stem()),
        )
        .as_deref()
        .and_then(crate::picture::Picture::parse)
        {
            for (key, v) in ["OMEGA_LEFT", "OMEGA_RIGHT", "OMEGA_TOP", "OMEGA_BOTTOM"]
                .into_iter()
                .zip(p.edges)
            {
                put(&mut s, key, &format!("{}", v as f32 / 100.0));
            }
            put(&mut s, "OMEGA_ZOOM", &format!("{}", p.zoom as f32 / 100.0));
        }
        s
    }
    pub fn collect_battery(&self) -> std::io::Result<()> {
        let path = self.saves().join(format!("{}.srm", self.stem()));
        if let Ok(bytes) = std::fs::read(path) {
            if !bytes.is_empty() {
                crate::persist::write_sav(&self.root, self.platform, &self.stem(), &bytes)?;
            }
        }
        Ok(())
    }
}
fn put(s: &mut String, key: &str, value: &str) {
    // Config values are data: quote paths without treating quotes or newlines as syntax.
    s.push_str(key);
    s.push_str(" = \"");
    for c in value.chars() {
        match c {
            '"' => s.push_str("\\\""),
            '\n' | '\r' => s.push(' '),
            _ => s.push(c),
        }
    }
    s.push_str("\"\n");
}
pub fn dim_gain(state: &AppState) -> f32 {
    [1.0, 0.75, 0.5, 0.25, 0.125][state.dim.min(4) as usize]
}

#[cfg(test)]
mod tests {
    use super::*;
    fn launch(root: &Path, platform: Platform) -> Launch {
        Launch {
            root: root.into(),
            rom: root.join("Games/A game.gba"),
            platform,
            core: Core::for_platform(platform)[0],
            clean: false,
            menu: false,
        }
    }
    #[test]
    fn real_slang_paths_and_physical_grid_are_independent_of_system_and_scaling() {
        let root = tempfile::tempdir().unwrap();
        for platform in Platform::ALL {
            let l = launch(root.path(), platform);
            for scaling in 0..3 {
                let mut state = AppState::default();
                state.scales[platform as usize] = scaling;
                for mode in 0..8 {
                    let text = l.preset(&state, mode);
                    assert!(text.contains("screen.slang"));
                    assert_eq!(
                        text.contains("OMEGA_GRID = \"1.0000\""),
                        matches!(mode, 3 | 7)
                    );
                    assert_eq!(text.contains("lcd1x.slang"), matches!(mode, 1 | 5));
                    assert_eq!(text.contains("lcd3x.slang"), matches!(mode, 2 | 6));
                    assert!(!text.contains("panel_raster"));
                }
            }
        }
    }
    #[test]
    fn stock_lcd_defaults_and_both_parameter_spellings_are_preserved() {
        let l = launch(Path::new("/card"), Platform::Gba);
        let mut state = AppState::default();
        for mode in [1, 2, 5, 6] {
            let text = l.preset(&state, mode);
            for key in ["BRIGHTEN_SCANLINES", "brighten_scanlines"] {
                assert!(text.contains(&format!("{key} = \"16.0000\"")));
            }
            for key in ["BRIGHTEN_LCD", "brighten_lcd"] {
                assert!(text.contains(&format!("{key} = \"4.0000\"")));
            }
        }
        state.lcd_darkness[0] = 200;
        let text = l.preset(&state, 6);
        assert!(text.contains("brighten_scanlines = \"8.0000\""));
        assert!(text.contains("PT_SHADOW_OFFSET_X = \"1.0000\""));
    }

    #[test]
    fn backend_configuration_uses_private_input_and_keeps_native_states_separate() {
        let l = launch(Path::new("/card"), Platform::Snes);
        let s = l.config(&AppState::default());
        for value in [
            "video_driver = \"glcore\"",
            "input_driver = \"omega\"",
            "network_cmd_enable = \"false\"",
            "aspect_ratio_index = \"0\"",
            "States/RetroArch/SNES/snes9x",
            "input_overlay_enable = \"false\"",
        ] {
            assert!(s.contains(value), "{value}");
        }
    }
    #[test]
    fn battery_backup_is_never_overwritten_and_snes_ignores_per_game_scale() {
        let root = tempfile::tempdir().unwrap();
        let mut l = launch(root.path(), Platform::Snes);
        l.core = Core::Snes9x;
        std::fs::create_dir_all(root.path().join("Saves/SNES")).unwrap();
        crate::persist::write_sav(root.path(), Platform::Snes, "A game", b"original").unwrap();
        let mut state = AppState::default();
        state.scales[3] = 2;
        l.prepare(&state).unwrap();
        std::fs::write(l.saves().join("A game.srm"), b"new battery").unwrap();
        l.collect_battery().unwrap();
        l.prepare(&state).unwrap();
        assert_eq!(
            std::fs::read(root.path().join("Saves/SNES/A game.pre-retroarch.sav")).unwrap(),
            b"original"
        );
        assert_eq!(
            std::fs::read(l.saves().join("A game.srm")).unwrap(),
            b"new battery"
        );
        omega_store::ini::write(
            root.path(),
            "Config/picture.ini",
            "SNES/A game",
            "100,0,0,0,0,0",
        )
        .unwrap();
        assert!(l.config(&state).contains("aspect_ratio_index = \"24\""));
    }
}
