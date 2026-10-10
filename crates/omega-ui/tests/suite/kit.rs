//! The kit's faces against the components they port: each is held to a screenshot of the HTML
//! kit in Chrome (kit/ref/, made by kit/make-refs.sh from kit/kit.html), pixel for pixel.
//!
//! The screenshots are drawn in palette.js's RAMP as written, which `RAMP` below is too, so a
//! face and its screenshot share their colours whatever the palette makes of a backdrop colour.
//! Set KIT_OUT to a folder to have each face written there as a PNG beside its reference.

use omega_ui::kit::action::{action_face, action_width, Side, BAR, RISE, TALL};
use omega_ui::kit::modal::{modal_face, modal_layout};
use omega_ui::kit::palette::Palette;
use omega_ui::kit::panel::{panel_face, tab_width, title_face, TAB_W};
use omega_ui::kit::pixel::{big_word, flip, keyed, letters, mirror, quad, ACCENT, ADVANCE, CAP};
use omega_ui::{Draw, TexId, UndoFace};

const RAMP: Palette = Palette {
    bg: [0x30, 0x22, 0x08],
    dark: [0x2a, 0x0f, 0x04],
    edge: [0x6a, 0x3d, 0x01],
    well: [0x4f, 0x30, 0x0d],
    plate: [0x76, 0x50, 0x20],
    tab: [0x84, 0x5a, 0x1a],
    band: [0xa5, 0x72, 0x21],
    stripe: [0xb0, 0x73, 0x1b],
    dim: [0xbe, 0x97, 0x32],
    frame: [0xe6, 0xab, 0x47],
    frame_hi: [0xf8, 0xb8, 0x3f],
    label: [0xf2, 0xc8, 0x7d],
    cream: [0xf5, 0xe5, 0xbb],
};

fn reference(name: &str) -> UndoFace {
    let path = format!(
        "{}/tests/suite/kit/ref/{name}.png",
        env!("CARGO_MANIFEST_DIR")
    );
    let file = std::fs::File::open(&path).unwrap_or_else(|e| panic!("{path}: {e}"));
    let mut decoder = png::Decoder::new(std::io::BufReader::new(file));
    decoder.set_transformations(png::Transformations::normalize_to_color8());
    let mut reader = decoder.read_info().expect("a PNG");
    let mut buf = vec![0; reader.output_buffer_size()];
    let info = reader.next_frame(&mut buf).expect("its pixels");
    assert_eq!(info.color_type, png::ColorType::Rgba, "{name} is not RGBA");
    buf.truncate(info.buffer_size());
    UndoFace {
        rgba: buf,
        w: info.width,
        h: info.height,
    }
}

fn write(path: &str, face: &UndoFace) {
    let f = std::fs::File::create(path).unwrap();
    let mut e = png::Encoder::new(std::io::BufWriter::new(f), face.w, face.h);
    e.set_color(png::ColorType::Rgba);
    e.set_depth(png::BitDepth::Eight);
    e.write_header()
        .unwrap()
        .write_image_data(&face.rgba)
        .unwrap();
}

/// `face` is `name`'s screenshot exactly: the same size, clear where it is clear, and every
/// other pixel the same opaque colour.
fn matches(face: &UndoFace, name: &str) {
    if let Ok(dir) = std::env::var("KIT_OUT") {
        write(&format!("{dir}/{name}.png"), face);
    }
    let shot = reference(name);
    assert_eq!((face.w, face.h), (shot.w, shot.h), "{name}'s size");
    let pixel = |f: &UndoFace, i: usize| match f.rgba[i * 4 + 3] {
        0 => [0; 4],
        _ => [
            f.rgba[i * 4],
            f.rgba[i * 4 + 1],
            f.rgba[i * 4 + 2],
            f.rgba[i * 4 + 3],
        ],
    };
    let off: Vec<_> = (0..(face.w * face.h) as usize)
        .filter(|&i| pixel(face, i) != pixel(&shot, i))
        .map(|i| {
            (
                i as u32 % face.w,
                i as u32 / face.w,
                pixel(face, i),
                pixel(&shot, i),
            )
        })
        .collect();
    assert!(
        off.is_empty(),
        "{name}: {} pixels differ from Chrome's, first (x, y, ours, Chrome's) {:?}",
        off.len(),
        &off[..off.len().min(8)]
    );
}

/// `top` laid over `under` with its top left at (`x`, `y`), where it is not clear.
fn over(under: &mut UndoFace, top: &UndoFace, x: u32, y: u32) {
    for ty in 0..top.h {
        for tx in 0..top.w {
            let from = ((ty * top.w + tx) * 4) as usize;
            if top.rgba[from + 3] == 0 {
                continue;
            }
            let to = (((y + ty) * under.w + x + tx) * 4) as usize;
            under.rgba[to..to + 4].copy_from_slice(&top.rgba[from..from + 4]);
        }
    }
}

#[test]
fn the_modal_is_chromes() {
    matches(&modal_face(200, 120, &RAMP), "modal");
}

/// A modal holding BUTTON SETTING, active, over ATTACK MODE, sharing its height by `weights`.
fn modal_with_panels(w: u32, h: u32, weights: &[u32]) -> UndoFace {
    let mut face = modal_face(w, h, &RAMP);
    let titles = [("BUTTON SETTING", true), ("ATTACK MODE", false)];
    for ([x, y, pw, ph], (title, active)) in modal_layout(w, h, weights).into_iter().zip(titles) {
        over(
            &mut face,
            &panel_face(pw, ph, title, active, &RAMP).face,
            x,
            y,
        );
    }
    face
}

#[test]
fn panels_in_a_modal_are_chromes_even_off_a_whole_pixel() {
    matches(&modal_with_panels(200, 121, &[1, 1]), "modal-panels");
    matches(&modal_with_panels(200, 120, &[4, 3]), "modal-panels-4-3");
    matches(&modal_with_panels(200, 120, &[1, 1]), "modal-panels-1-1");
}

#[test]
fn panels_share_a_modal_and_one_stripe() {
    assert_eq!(
        modal_layout(200, 121, &[1, 1]),
        vec![[4, 2, 192, 58], [4, 59, 192, 58]]
    );
    // 57.5 pixels each: the edge between them rounds down the page
    assert_eq!(
        modal_layout(200, 120, &[1, 1]),
        vec![[4, 2, 192, 58], [4, 59, 192, 57]]
    );
    assert_eq!(
        modal_layout(200, 120, &[4, 3]),
        vec![[4, 2, 192, 66], [4, 67, 192, 49]]
    );
    assert_eq!(modal_layout(200, 120, &[1]), vec![[4, 2, 192, 114]]);
}

#[test]
fn panels_are_chromes() {
    for (title, name) in [("BUTTON SETTING", "button-setting"), ("CONFIGS", "configs")] {
        matches(
            &panel_face(160, 60, title, true, &RAMP).face,
            &format!("panel-{name}-active"),
        );
        matches(
            &panel_face(160, 60, title, false, &RAMP).face,
            &format!("panel-{name}"),
        );
    }
}

#[test]
fn a_panels_well_and_title_are_where_its_css_puts_them() {
    let panel = panel_face(160, 60, "CONFIGS", true, &RAMP);
    assert_eq!(panel.well, [6, 18, 148, 37]);
    assert_eq!(panel.title, [6, 2, 42, 11]);
}

#[test]
fn the_active_title_over_a_dim_one_is_the_active_panel() {
    let mut dim = panel_face(160, 60, "BUTTON SETTING", false, &RAMP);
    let [x, y, w, h] = dim.title;
    let light = title_face("BUTTON SETTING", true, &RAMP);
    assert_eq!((light.w, light.h), (w, h));
    over(&mut dim.face, &light, x, y);
    matches(&dim.face, "panel-button-setting-active");
}

#[test]
fn a_tab_grows_to_a_long_title_and_no_narrower() {
    assert_eq!(tab_width("BUTTON SETTING"), TAB_W);
    let long = "A TITLE FAR TOO LONG FOR ITS TAB";
    assert_eq!(tab_width(long), 1 + 4 + long.len() as u32 * 6 + 4);
}

#[test]
fn actions_are_chromes() {
    let cases = [
        (Side::Left, "QUICKMENU", "SELECT", "action-left-quickmenu"),
        (Side::Right, "SETTINGS", "START", "action-right-settings"),
        (Side::Left, "RETURN", "B", "action-left-return"),
        (Side::Right, "CONFIRM", "A", "action-right-confirm"),
        (Side::Left, "", "", "action-left-empty"),
        (Side::Right, "", "", "action-right-empty"),
    ];
    for (side, word, pill, name) in cases {
        let action = action_face(side, word, pill, &RAMP);
        assert_eq!(action.rise, RISE);
        matches(&action.face, name);
    }
}

#[test]
fn actions_are_as_wide_as_their_text() {
    assert_eq!(action_width("SETTINGS", "START"), 124);
    assert_eq!(action_width("QUICKMENU", "SELECT"), 131);
    assert_eq!(action_width("RETURN", "B"), 110);
    assert_eq!(action_width("CONFIRM", "A"), 117);
    assert_eq!(action_width("", ""), 107, "room is kept for text to come");
    let face = action_face(Side::Left, "SETTINGS", "START", &RAMP).face;
    assert_eq!((face.w, face.h), (124, TALL));
    assert_eq!(TALL, BAR + RISE);
    assert_eq!(TALL, 27);
}

#[test]
fn a_big_word_is_twice_letters_and_its_shadow() {
    let word = big_word("SETTINGS");
    let ink = letters("SETTINGS", ADVANCE + 1);
    assert_eq!(ink.len(), 11);
    assert_eq!(word.len(), ink.len() * 2 + 1);
    assert_eq!(word.len(), 23);
    assert!(word.iter().all(|row| row.len() == 8 * 7));
    // the capitals stand on letters' rows 2 to 8, so twice that, shadow one row under
    let inked = |rows: &[String], key: char| -> Vec<usize> {
        (0..rows.len()).filter(|&y| rows[y].contains(key)).collect()
    };
    assert_eq!(
        inked(&word, 'L'),
        (2 * ACCENT..2 * (ACCENT + CAP)).collect::<Vec<_>>()
    );
    assert_eq!(
        inked(&word, 'w'),
        (2 * ACCENT + 1..=2 * (ACCENT + CAP)).collect::<Vec<_>>()
    );
}

#[test]
fn capitals_fill_letters_rows_two_to_eight() {
    let rows = letters("BUTTON SETTING 0123456789", ADVANCE);
    let inked: Vec<usize> = (0..rows.len()).filter(|&y| rows[y].contains('#')).collect();
    assert_eq!(inked, (ACCENT..ACCENT + CAP).collect::<Vec<_>>());
    assert_eq!(rows[0].len(), 25 * ADVANCE);
    assert_eq!(letters("", ADVANCE), vec![".".to_string(); 11]);
}

#[test]
fn mirror_makes_a_sprite_from_its_quarter_and_flip_turns_it() {
    assert_eq!(mirror(&[".s", "sA"]), [".s.", "sAs", ".s."]);
    assert_eq!(
        mirror(&["..k", ".kH", "kHs"]),
        ["..k..", ".kHk.", "kHsHk", ".kHk.", "..k.."]
    );
    assert_eq!(flip(&["AAAA....", "Abb"]), ["....AAAA", "bbA"]);
    assert_eq!(flip(&flip(&["k.e", "Hs"])), ["k.e", "Hs"]);
}

#[test]
fn keyed_paints_each_key_and_leaves_dots_clear() {
    let face = keyed(&["kA", ".c"], &RAMP);
    assert_eq!((face.w, face.h), (2, 2));
    assert_eq!(&face.rgba[0..4], &[0x2a, 0x0f, 0x04, 255]);
    assert_eq!(&face.rgba[4..8], &[0xe6, 0xab, 0x47, 255]);
    assert_eq!(face.rgba[11], 0, "a dot is clear");
    assert_eq!(&face.rgba[12..16], &[0xf5, 0xe5, 0xbb, 255]);
}

#[test]
fn a_face_is_drawn_two_screen_pixels_to_a_texel() {
    let tex = TexId::from_raw(1);
    assert_eq!(
        quad(tex, 124, 27, 10.0, 20.0),
        Draw::Tex {
            x: 10.0,
            y: 20.0,
            w: 248.0,
            h: 54.0,
            tex,
            alpha: 1.0
        }
    );
}
