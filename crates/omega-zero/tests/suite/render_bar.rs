#![cfg(target_os = "macos")]
//! The bar along the bottom as the panel shows it: every pixel of its plate and its actions,
//! notches and all, is the kit's face drawn at twice its size on whole pixels, on the shelf, in
//! Settings and under the core picker with ◀▶ SWAP on its plate, in Night and in Amber.

use crate::common;

use std::collections::VecDeque;

use common::{clocked, tmp_root_with_carts};
use omega_gfx::{out_h, out_w, Compositor, HeadlessSurface};
use omega_input::{Btn, InputSource, Millis, RawEvent};
use omega_power::SimPlatform;
use omega_store::{read_app_state, write_app_state};
use omega_ui::kit::action::{action_pair, Font, Side, RISE, TALL};
use omega_ui::kit::bar::{bar_layout, plate_face, Hint};
use omega_ui::kit::palette::Palette;
use omega_ui::{UndoFace, SPRITE_PX};
use omega_zero::frontend::Frontend;
use omega_zero::screens::bar::{Actions, SWAP};

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

/// The bar the screen's width with `actions` at its ends, made together as a pair, and `hints`
/// on its plate, at one texel to an art pixel: the plate, then the actions over it, on a face
/// `TALL` high whose row `RISE` is the bar's top.
fn kit_bar(actions: Actions, hints: &[Hint], pal: &Palette) -> UndoFace {
    let width = out_w() / SPRITE_PX;
    let [left, right] = action_pair(
        actions.action(Side::Left),
        actions.action(Side::Right),
        Font::Bold,
        pal,
    );
    let layout = bar_layout(width, &left, &right);
    let mut bar = UndoFace {
        rgba: vec![0; (width * TALL * 4) as usize],
        w: width,
        h: TALL,
    };
    let [px, py, pw, _] = layout.plate;
    let plate = plate_face(pw, hints, pal);
    over(&mut bar, &plate, px as i32, (py + RISE) as i32);
    for (face, [x, y]) in [(&left.face, layout.left), (&right.face, layout.right)] {
        over(&mut bar, face, x, y + RISE as i32);
    }
    bar
}

fn over(onto: &mut UndoFace, face: &UndoFace, x: i32, y: i32) {
    for fy in 0..face.h as i32 {
        for fx in 0..face.w as i32 {
            let i = ((fy * face.w as i32 + fx) * 4) as usize;
            let (tx, ty) = (x + fx, y + fy);
            if face.rgba[i + 3] == 0
                || tx < 0
                || ty < 0
                || tx >= onto.w as i32
                || ty >= onto.h as i32
            {
                continue;
            }
            let o = ((ty * onto.w as i32 + tx) * 4) as usize;
            onto.rgba[o..o + 4].copy_from_slice(&face.rgba[i..i + 4]);
        }
    }
}

/// Where the frame `px` is not `bar`'s opaque pixels drawn at twice their size along the
/// bottom of the screen, as (x, y, got, want).
fn misses(px: &[u8], bar: &UndoFace) -> Vec<(usize, usize, [u8; 3], [u8; 3])> {
    let (w, top) = (out_w() as usize, (out_h() - TALL * SPRITE_PX) as usize);
    let mut off = Vec::new();
    for ty in 0..bar.h as usize {
        for tx in 0..bar.w as usize {
            let t = (ty * bar.w as usize + tx) * 4;
            if bar.rgba[t + 3] != 0xff {
                continue;
            }
            let want = [bar.rgba[t], bar.rgba[t + 1], bar.rgba[t + 2]];
            for (dx, dy) in [(0, 0), (1, 0), (0, 1), (1, 1)] {
                let (x, y) = (tx * 2 + dx, top + ty * 2 + dy);
                let o = (y * w + x) * 4;
                let got = [px[o], px[o + 1], px[o + 2]];
                if got != want {
                    off.push((x, y, got, want));
                }
            }
        }
    }
    off
}

fn composed(f: &mut Frontend, c: &mut Compositor, name: &str) -> Vec<u8> {
    f.app_mut().finish_slide();
    f.compose(c);
    let px = c.read_frame();
    if let Ok(dir) = std::env::var("SCRATCH_PNG_DIR") {
        let path = format!("{dir}/bar-{name}.png");
        let file = std::fs::File::create(&path).expect("create png");
        let mut e = png::Encoder::new(std::io::BufWriter::new(file), out_w(), out_h());
        e.set_color(png::ColorType::Rgba);
        e.set_depth(png::BitDepth::Eight);
        e.write_header()
            .expect("png header")
            .write_image_data(&px)
            .expect("png data");
    }
    px
}

#[test]
fn the_bar_is_the_kits_pixel_for_pixel_on_the_shelf_and_in_settings() {
    let Ok(surface) = HeadlessSurface::new() else {
        return;
    };
    let Ok(mut c) = Compositor::new(&surface) else {
        return;
    };
    for colour in ["night", "amber"] {
        let d = tmp_root_with_carts(&["Emerald", "Fusion"]);
        clocked(d.path());
        let mut state = read_app_state(d.path());
        state.backdrop_colour = omega_store::colour_index(colour).expect("a backdrop colour");
        write_app_state(d.path(), &state).unwrap();
        let mut f = Frontend::boot(Box::new(SimPlatform::at(d.path().to_path_buf())));
        f.upload_faces(&mut c);
        let pal = f.app().palette();

        let shelf = composed(&mut f, &mut c, &format!("shelf-{colour}"));
        let off = misses(&shelf, &kit_bar(Actions::Shelf, &[], pal));
        assert!(
            off.is_empty(),
            "{colour}: the shelf's bar is off the kit's at {} pixels, first {:?}",
            off.len(),
            &off[..off.len().min(4)]
        );

        let mut input = Script(VecDeque::new());
        tap(&mut f, &mut input, Btn::Start);
        let settings = composed(&mut f, &mut c, &format!("settings-{colour}"));
        let off = misses(&settings, &kit_bar(Actions::Menu, &[], pal));
        assert!(
            off.is_empty(),
            "{colour}: Settings' bar is off the kit's at {} pixels, first {:?}",
            off.len(),
            &off[..off.len().min(4)]
        );
    }
}

#[test]
fn the_core_pickers_bar_has_the_swap_on_its_plate_pixel_for_pixel() {
    let Ok(surface) = HeadlessSurface::new() else {
        return;
    };
    let Ok(mut c) = Compositor::new(&surface) else {
        return;
    };
    for colour in ["night", "amber"] {
        let d = tmp_root_with_carts(&["Emerald", "Fusion"]);
        clocked(d.path());
        let mut state = read_app_state(d.path());
        state.backdrop_colour = omega_store::colour_index(colour).expect("a backdrop colour");
        write_app_state(d.path(), &state).unwrap();
        let mut f = Frontend::boot(Box::new(SimPlatform::at(d.path().to_path_buf())));
        f.upload_faces(&mut c);
        f.compose(&mut c);
        let pal = f.app().palette();

        let mut input = Script(VecDeque::new());
        tap(&mut f, &mut input, Btn::X);
        assert!(
            f.app().core_picker().is_some(),
            "{colour}: X opened no picker"
        );
        // past the wait for the board's faces, then all the way open
        f.app_mut().update(2.0);
        f.app_mut().update(1.0);
        assert_eq!(f.app().bar_hints_shown(), [SWAP], "{colour}");
        let picker = composed(&mut f, &mut c, &format!("core-picker-{colour}"));
        let off = misses(&picker, &kit_bar(Actions::Menu, &[SWAP], pal));
        assert!(
            off.is_empty(),
            "{colour}: the picker's bar is off the kit's at {} pixels, first {:?}",
            off.len(),
            &off[..off.len().min(4)]
        );
        // and the swap really is there: the plate without it is not what shows
        assert!(!misses(&picker, &kit_bar(Actions::Menu, &[], pal)).is_empty());
    }
}
