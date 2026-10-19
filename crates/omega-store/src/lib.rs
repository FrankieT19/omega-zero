mod app_state;
mod atomic;
mod backdrop;
pub mod cart_shell;

pub const SHADER_NAMES: [&str; 11] = [
    "Clean",
    "LCD1x",
    "LCD3x",
    "Grid",
    "Pixel transparency",
    "LCD1x + PT",
    "LCD3x + PT",
    "Grid + PT",
    "CRT scanlines",
    "zfast CRT",
    "CRT Pi (flat)",
];

/// Stable preset IDs: handheld configurations keep their original numeric values.
pub fn shader_choices(platform: Platform) -> &'static [u8] {
    match platform {
        Platform::Ps1 | Platform::Snes => &[0, 8, 9, 10],
        _ => &[0, 1, 2, 3, 4, 5, 6, 7],
    }
}
pub fn shader_for(platform: Platform, value: u8) -> u8 {
    if shader_choices(platform).contains(&value) {
        value
    } else {
        0
    }
}
pub fn next_shader(platform: Platform, current: u8, by: i32) -> u8 {
    let choices = shader_choices(platform);
    let index = choices.iter().position(|&n| n == current).unwrap_or(0) as i32;
    choices[(index + by).rem_euclid(choices.len() as i32) as usize]
}
mod config;
mod core;
mod font;
pub mod gb;
mod gba;
pub mod ini;
mod platform;
pub mod ps1;
mod region;
mod ring;
mod scan;
pub mod snes;
mod stamp;
mod theme;

pub const GB_PALETTES: [&str; 16] = [
    "GBC (game)",
    "GB - Pocket",
    "GB - DMG",
    "GBC - Blue",
    "GBC - Grayscale",
    "GB - Light",
    "GBC - Brown",
    "GBC - Dark Blue",
    "GBC - Dark Brown",
    "GBC - Dark Green",
    "GBC - Green",
    "GBC - Inverted",
    "GBC - Orange",
    "GBC - Pastel Mix",
    "GBC - Red",
    "GBC - Yellow",
];

pub use app_state::{
    read_app_state, write_app_state, AppState, BLUE_LIGHT_MAX, BRIGHTNESS_MAX, FF_SPEEDS,
    FF_SPEED_DEFAULT, UTC_OFFSET_MAX, UTC_OFFSET_MIN, VOLUME_MAX,
};
pub use atomic::{atomic_write, atomic_write_dated};
pub use backdrop::{colour_index, BackdropStyle, BACKDROP_COLOURS};
pub use cart_shell::{Outline, ShellChoice, ShellFinish, CART_SHELL_FILE, LABELS_SHELL_FILE};
pub use config::{move_config, CONFIG_DIR};
pub use core::{
    core_for, core_for_platform, read_selected_cores, write_selected_core, Core, SELECTED_CORE_FILE,
};
pub use font::Font;
pub use gba::{header_clean, header_code, header_title};
pub use platform::Platform;
pub use region::Region;
pub use ring::{StateEntry, StateRing, RING_MAX};
pub use scan::{initial, is_hidden, scan, sort_key, Cart, StoreError};
pub use stamp::{
    civil_from_days, days_from_civil, days_in_month, format_stamp, parse_stamp, stamp_now,
};
pub use theme::{Theme, THEME_FILE};

#[cfg(test)]
mod shader_tests {
    use super::*;
    #[test]
    fn shader_ids_keep_handheld_settings_and_migrate_console_lcd_choices() {
        for platform in Platform::ALL {
            for mode in 0..=10 {
                let value = shader_for(platform, mode);
                assert!(shader_choices(platform).contains(&value));
                let next = next_shader(platform, value, 1);
                assert_eq!(next_shader(platform, next, -1), value);
            }
        }
        assert_eq!(shader_for(Platform::Gba, 7), 7);
        assert_eq!(shader_for(Platform::Snes, 2), 0);
        assert_eq!(shader_for(Platform::Ps1, 10), 10);
    }
}
