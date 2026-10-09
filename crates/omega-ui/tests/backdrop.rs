use omega_store::{BackdropStyle, BACKDROP_COLOURS};
use omega_ui::{
    backdrop_colour_name, backdrop_pattern_face, backdrop_period, backdrop_rgb, backdrop_shift,
    backdrop_speed, out_h, out_w, BACKDROP_PALETTE,
};

const PATTERNS: [BackdropStyle; 2] = [BackdropStyle::Grid, BackdropStyle::Bits];

#[test]
fn a_pattern_repeats_every_period_so_drifting_it_never_shows_a_seam() {
    for style in PATTERNS {
        let p = backdrop_period(style);
        let face = backdrop_pattern_face(style, 3).expect("no pattern");
        assert_eq!((face.w, face.h), (out_w() + p, out_h()));
        let px = |x: u32, y: u32| &face.rgba[((y * face.w + x) * 4) as usize..][..4];
        for y in 0..face.h {
            for x in 0..out_w() {
                assert_eq!(px(x, y), px(x + p, y), "{style:?} breaks at {x},{y}");
            }
        }
        assert!(
            face.rgba.chunks_exact(4).all(|c| c[3] == 255),
            "{style:?} lets the black show through"
        );
    }
}

#[test]
fn every_period_divides_every_canvas_so_a_still_pattern_sits_square() {
    for style in PATTERNS {
        let p = backdrop_period(style);
        for size in [480, 640, 720] {
            assert_eq!(size % p, 0, "{style:?}'s {p} does not divide {size}");
        }
    }
}

#[test]
fn a_plain_colour_or_a_picture_has_no_pattern_to_make() {
    assert!(backdrop_pattern_face(BackdropStyle::Solid, 0).is_none());
    assert!(backdrop_pattern_face(BackdropStyle::Wallpaper, 0).is_none());
}

#[test]
fn a_drifting_pattern_moves_slowly_by_the_clock_not_the_frame_and_starts_over() {
    for style in PATTERNS {
        let p = backdrop_period(style);
        assert_eq!(
            backdrop_shift(style, false, 123_456),
            0,
            "a still one moved"
        );
        let lap = (p as f64 / backdrop_speed(style) * 1000.0).round() as u64;
        assert!(lap >= 2_000, "{style:?} crosses a period in {lap} ms");
        let mut was = 0;
        for ms in (0..lap).step_by(16) {
            let s = backdrop_shift(style, true, ms);
            assert!(s < p);
            assert!(s >= was, "{style:?} went back at {ms} ms");
            was = s;
        }
        for t in [0, 777, 5_000, 61_234] {
            assert_eq!(
                backdrop_shift(style, true, t),
                backdrop_shift(style, true, t + lap),
                "{style:?} did not start over after one period"
            );
        }
    }
    assert_eq!(backdrop_shift(BackdropStyle::Solid, true, 9_999), 0);
}

#[test]
fn every_colour_has_a_name_and_stays_dark_enough_for_a_label() {
    assert_eq!(BACKDROP_PALETTE.len(), BACKDROP_COLOURS.len());
    let mut names: Vec<&str> = (0..BACKDROP_COLOURS.len() as u8)
        .map(backdrop_colour_name)
        .collect();
    assert_eq!(names[0], "Night");
    names.sort();
    names.dedup();
    assert_eq!(
        names.len(),
        BACKDROP_COLOURS.len(),
        "two colours share a name"
    );
    for c in 0..BACKDROP_COLOURS.len() as u8 {
        let [r, g, b] = backdrop_rgb(c);
        let luma = 0.2126 * r as f32 + 0.7152 * g as f32 + 0.0722 * b as f32;
        assert!(
            luma < 48.0,
            "{} is too light to sit behind carts",
            backdrop_colour_name(c)
        );
    }
}
