#![cfg(target_os = "macos")]

use crate::common;

use std::collections::VecDeque;

use common::{clocked, tmp_root_with_carts};
use omega_gfx::{out_h, out_w, Compositor, HeadlessSurface};
use omega_input::{Btn, InputSource, Millis, RawEvent};
use omega_power::SimPlatform;
use omega_store::{read_app_state, write_app_state, BackdropStyle};
use omega_ui::{quick_rows_shown, quick_top, quick_window, stage_x, SettingsRow, QUICK_PITCH};
use omega_zero::frontend::Frontend;

struct Script(VecDeque<Vec<RawEvent>>);

impl InputSource for Script {
    fn poll(&mut self, _now: Millis) -> Vec<RawEvent> {
        self.0.pop_front().unwrap_or_default()
    }
}

fn tap(f: &mut Frontend, input: &mut Script, btn: Btn) {
    input.0.push_back(vec![RawEvent::Down(btn)]);
    f.advance(input);
    input.0.push_back(vec![RawEvent::Up(btn)]);
    f.advance(input);
}

fn at(px: &[u8], x: usize, y: usize) -> [u8; 3] {
    let o = (y * out_w() as usize + x) * 4;
    [px[o], px[o + 1], px[o + 2]]
}

fn inked(px: &[u8], xs: std::ops::Range<usize>, top: usize) -> Vec<usize> {
    xs.filter(|&x| (top + 8..top + 44).any(|y| at(px, x, y)[0] > 0x80))
        .collect()
}

/// The frame once the screen has slid into place.
fn composed(f: &mut Frontend, c: &mut Compositor, name: &str) -> Vec<u8> {
    f.app_mut().finish_slide();
    f.compose(c);
    let px = c.read_frame();
    if let Ok(dir) = std::env::var("SCRATCH_PNG_DIR") {
        let path = format!("{dir}/settings-{name}.png");
        let file = std::fs::File::create(&path).expect("create png");
        let mut e = png::Encoder::new(std::io::BufWriter::new(file), out_w(), out_h());
        e.set_color(png::ColorType::Rgba);
        e.set_depth(png::BitDepth::Eight);
        e.write_header()
            .expect("png header")
            .write_image_data(&px)
            .expect("png data");
        println!("wrote {path}");
    }
    px
}

#[test]
fn settings_renders_full_screen() {
    let Ok(surface) = HeadlessSurface::new() else {
        return;
    };
    let Ok(mut c) = Compositor::new(&surface) else {
        return;
    };
    let d = tmp_root_with_carts(&["Emerald", "Fusion"]);
    clocked(d.path());
    solid(d.path(), 0);
    let mut f = Frontend::boot(Box::new(SimPlatform::at(d.path().to_path_buf())));
    f.upload_faces(&mut c);
    let mut input = Script(VecDeque::new());
    tap(&mut f, &mut input, Btn::Start);

    let bar = [0x4d, 0x4d, 0x57];
    // a plain backdrop seen through the plate, wherever the list draws nothing: over the
    // bottom bar and its notches
    let over_the_bar = out_h() as usize - 2 - 2 * omega_ui::kit::action::TALL as usize;
    let ground = at(&composed(&mut f, &mut c, "ground"), 2, over_the_bar);
    assert_ne!(ground, bar, "the ground is the bar's colour");
    let mut bar_on = SettingsRow::ALL[0];
    let (n, mut first) = (SettingsRow::ALL.len(), 0);
    let shown = quick_rows_shown(n);
    for (name, selected) in [
        ("fast-forward", SettingsRow::FastForward),
        ("colour-correction", SettingsRow::ColourCorrection),
        ("date-time", SettingsRow::DateTime),
        ("about", SettingsRow::About),
    ] {
        for i in bar_on.index()..selected.index() {
            tap(&mut f, &mut input, Btn::Down);
            first = quick_window(i + 1, n, first);
        }
        bar_on = selected;
        let px = composed(&mut f, &mut c, name);
        let row_top = |row: SettingsRow| quick_top() + QUICK_PITCH * (row.index() - first) as f32;

        let top = row_top(selected) as usize;
        for x in [1, out_w() as usize - 2] {
            assert_eq!(at(&px, x, top + 26), bar, "{name}: no bar at x {x}");
        }
        assert!(
            (0..out_w() as usize).all(|x| at(&px, x, top + 26) != ground),
            "{name}: the bar breaks somewhere across the row"
        );
        assert_eq!(
            at(&px, out_w() as usize / 2, top + 1),
            ground,
            "{name}: the bar is not inset"
        );
        assert_eq!(
            at(&px, out_w() as usize - 2, 2),
            ground,
            "{name}: not on the ground"
        );

        for row in SettingsRow::ALL.into_iter().skip(first).take(shown) {
            let top = row_top(row) as usize;
            let label = inked(&px, 0..out_w() as usize / 2, top);
            let first = *label.first().expect("a row with no label");
            assert!(
                (32..=36).contains(&first),
                "{name}: {row:?}'s label starts at x {first}"
            );
            if matches!(
                row,
                SettingsRow::Labels | SettingsRow::About | SettingsRow::Background
            ) {
                continue;
            }
            let value = inked(&px, out_w() as usize / 2..out_w() as usize, top);
            let last = *value.last().expect("a row with no value");
            assert!(
                (out_w() as usize - 41..=out_w() as usize - 32).contains(&last),
                "{name}: {row:?}'s value ends at x {last}"
            );
        }
    }

    while f.app().settings_row() != Some(SettingsRow::DateTime) {
        tap(&mut f, &mut input, Btn::Up);
    }
    tap(&mut f, &mut input, Btn::A);
    let px = composed(&mut f, &mut c, "clock");
    assert!(
        (200..270).any(|x| at(
            &px,
            (x as f32 + stage_x()) as usize,
            298 + (out_h() as usize - 480) / 2
        ) == [0xf6, 0xf4, 0xef]),
        "the clock from the menu does not offer B BACK"
    );
}

/// Sets the card's background to the plain colour `colour`.
fn solid(root: &std::path::Path, colour: u8) {
    let mut s = read_app_state(root);
    s.backdrop = Some(BackdropStyle::Solid);
    s.backdrop_colour = colour;
    write_app_state(root, &s).expect("write omega-zero.state");
}

/// Settings over a card whose background is the plain colour `colour`.
fn settings_over(colour: u8, c: &mut Compositor) -> Vec<u8> {
    let d = tmp_root_with_carts(&["Emerald", "Fusion"]);
    clocked(d.path());
    solid(d.path(), colour);
    let mut f = Frontend::boot(Box::new(SimPlatform::at(d.path().to_path_buf())));
    f.upload_faces(c);
    let mut input = Script(VecDeque::new());
    tap(&mut f, &mut input, Btn::Start);
    assert!(f.app().settings_row().is_some(), "START opened no Settings");
    composed(&mut f, c, &format!("over-{colour}"))
}

#[test]
fn the_background_shows_through_under_settings() {
    let Ok(surface) = HeadlessSurface::new() else {
        return;
    };
    let Ok(mut c) = Compositor::new(&surface) else {
        return;
    };
    // slate and crimson, neither of them the night the quick menu's opaque ground was
    let (slate, crimson) = (settings_over(1, &mut c), settings_over(6, &mut c));
    let opaque = [0x05, 0x05, 0x08];
    for (x, y) in [
        (2, 2),
        (out_w() as usize - 2, out_h() as usize - 2),
        (2, out_h() as usize - 2),
    ] {
        assert_ne!(
            at(&slate, x, y),
            opaque,
            "the menu still hides the background at ({x}, {y})"
        );
        assert_ne!(
            at(&slate, x, y),
            at(&crimson, x, y),
            "the menu looks the same over two backgrounds at ({x}, {y})"
        );
    }
    let [r, g, b] = at(&crimson, 2, out_h() as usize - 2);
    assert!(
        r > g && r > b,
        "crimson does not show through the plate: {:?}",
        [r, g, b]
    );
}

#[test]
fn every_fast_forward_speed_sits_on_the_rows_right_edge_and_clears_the_label() {
    let Ok(surface) = HeadlessSurface::new() else {
        return;
    };
    let Ok(mut c) = Compositor::new(&surface) else {
        return;
    };
    let d = tmp_root_with_carts(&["Emerald", "Fusion"]);
    clocked(d.path());
    let mut f = Frontend::boot(Box::new(SimPlatform::at(d.path().to_path_buf())));
    f.upload_faces(&mut c);
    let mut input = Script(VecDeque::new());
    tap(&mut f, &mut input, Btn::Start);
    tap(&mut f, &mut input, Btn::Left);
    tap(&mut f, &mut input, Btn::Left);

    let top = quick_top() as usize;
    for name in ["2x", "3x", "4x", "6x", "8x"] {
        let px = composed(&mut f, &mut c, name);
        let value = inked(&px, out_w() as usize / 2..out_w() as usize, top);
        let last = *value
            .last()
            .unwrap_or_else(|| panic!("{name}: the Fast Forward row has no value"));
        assert!(
            (out_w() as usize - 41..=out_w() as usize - 32).contains(&last),
            "{name} ends at x {last}, off the edge every other value keeps"
        );
        assert!(
            inked(
                &px,
                out_w() as usize / 2 - 10..out_w() as usize / 2 + 10,
                top
            )
            .is_empty(),
            "{name} and its arrows reach the middle of the row"
        );
        let label = inked(&px, 0..350, top);
        let first = *label
            .first()
            .unwrap_or_else(|| panic!("{name}: the Fast Forward row has no label"));
        assert!(
            (32..=36).contains(&first),
            "the label moved to x {first} to make room for {name}"
        );
        tap(&mut f, &mut input, Btn::Right);
    }
}
