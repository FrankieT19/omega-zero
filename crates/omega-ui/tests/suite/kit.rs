//! The kit's faces against the components they port: each is held to a screenshot of the HTML
//! kit in Chrome (kit/ref/, made by kit/make-refs.sh from kit/kit.html), pixel for pixel.
//!
//! The screenshots are drawn in palette.js's RAMP as written, which `RAMP` below is too, so a
//! face and its screenshot share their colours whatever the palette makes of a backdrop colour.
//! Set KIT_OUT to a folder to have each face written there as a PNG beside its reference.
//!
//! Text: kit.html turns macOS's font smoothing off (`-webkit-font-smoothing: antialiased`),
//! which otherwise rings every DOM glyph with a faint halo a canvas word never gets, and every
//! component draws with `text-rendering: geometricPrecision`, which flow.html's notes and
//! columns inherit from their panel. So set, Pixel Code at 9 px lands on whole pixels, fully
//! inked or clear, and Chrome's ink is exactly the full face's as fontdue rasterizes it, a
//! pixel inked where its coverage is at least half: the web font is the same outlines cut to
//! Latin-1, so é and × match too. flow.html's ◀ ▶ ▲ ▼, which the cut face has not, are
//! flow/art.js's sprites, copied from the full face and the same pixels as fontdue's.

use omega_ui::kit::action::{action_face, action_width, Side, BAR, RISE, TALL};
use omega_ui::kit::bar::{
    action_swap, bar_layout, cap_width, hint_width, hints_width, plate_face, BarLayout, Hint, Swap,
    PLATE_H, PLATE_TOP, SINK,
};
use omega_ui::kit::list_row::{list_row_face, ListRow};
use omega_ui::kit::modal::{modal_face, modal_layout};
use omega_ui::kit::palette::Palette;
use omega_ui::kit::panel::{panel_face, tab_width, title_face, PanelFace, TAB_W};
use omega_ui::kit::pixel::{big_word, flip, keyed, letters, mirror, quad, ACCENT, ADVANCE, CAP};
use omega_ui::kit::row::{panel_height, row_face, row_rect, row_width, Row, ROW_H};
use omega_ui::kit::text::text_face;
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

/// A row as kit.html builds one from its query: `label`, `value`, and the flags that are set.
fn row<'a>(label: &'a str, value: Option<&'a str>, flags: &str) -> Row<'a> {
    let has = |flag| flags.split(' ').any(|f| f == flag);
    Row {
        label,
        value,
        lit: has("lit"),
        carets: has("carets"),
        opens: has("opens"),
        dim: has("dim"),
    }
}

#[test]
fn rows_are_chromes() {
    let cases = [
        ("row-plain", row("FAST FORWARD", Some("6X"), "carets")),
        (
            "row-lit-carets",
            row("FAST FORWARD", Some("6X"), "carets lit"),
        ),
        ("row-lit", row("RUMBLE", Some("ON"), "lit")),
        ("row-opens", row("LABELS", None, "opens")),
        ("row-opens-lit", row("NETWORK", Some("HOME"), "opens lit")),
        (
            "row-carets-opens-lit",
            row("SHADER", Some("SHARP"), "carets opens lit"),
        ),
        ("row-dim", row("WI-FI", Some("OFF"), "carets dim")),
        ("row-dim-lit", row("WI-FI", Some("OFF"), "carets dim lit")),
        ("row-dim-opens", row("SSH", None, "opens dim")),
        ("row-value", row("IP", Some("192.168.1.20"), "")),
        ("row-case", row("Host", Some("rg35xxsp.local"), "")),
        ("row-name", row("FETCH LABELS", None, "")),
    ];
    for (name, row) in cases {
        let face = row_face(150, &row, &RAMP);
        assert_eq!((face.w, face.h), (150, ROW_H));
        matches(&face, name);
    }
}

#[test]
fn a_row_too_narrow_cuts_its_label_as_chrome_does() {
    let cut = row("FAST FORWARD SOUND", Some("OFF"), "lit");
    matches(&row_face(60, &cut, &RAMP), "row-cut");
    let carets = row("FAST FORWARD", Some("6X"), "carets lit");
    matches(&row_face(50, &carets, &RAMP), "row-cut-carets");
    // no room at all: the label is gone and the value runs on past the band's inside
    let squeezed = row("RUMBLE", Some("OFF"), "");
    matches(&row_face(30, &squeezed, &RAMP), "row-squeezed");
}

/// A network's row as kit.html builds one: `label`, `value`, `signal`, and the flags set.
fn network<'a>(
    label: &'a str,
    value: Option<&'a str>,
    signal: Option<u8>,
    flags: &str,
) -> ListRow<'a> {
    let has = |flag| flags.split(' ').any(|f| f == flag);
    ListRow {
        label,
        value,
        signal,
        lock: has("lock"),
        lit: has("lit"),
        dim: has("dim"),
    }
}

#[test]
fn network_rows_are_chromes() {
    let cases = [
        (
            "list-home-lit",
            network("Home", Some("CONNECTED"), Some(3), "lock lit"),
        ),
        ("list-cafe", network("Café Wi-Fi", None, Some(2), "")),
        ("list-cafe-lit", network("Café Wi-Fi", None, Some(2), "lit")),
        (
            "list-upstairs",
            network("Upstairs 5G", None, Some(1), "lock"),
        ),
        ("list-attic", network("Attic", None, Some(0), "lock")),
        (
            "list-saved",
            network("Office", Some("SAVED"), Some(1), "lock"),
        ),
        ("list-dim", network("Home", None, Some(3), "lock dim")),
        (
            "list-dim-lit",
            network("Home", Some("CONNECTED"), Some(3), "lock dim lit"),
        ),
        ("list-bare", network("Hidden", None, None, "")),
    ];
    for (name, row) in cases {
        let face = list_row_face(150, &row, &RAMP);
        assert_eq!((face.w, face.h), (150, ROW_H));
        matches(&face, name);
    }
    let long = network("A very long network name", None, Some(2), "lock lit");
    matches(&list_row_face(100, &long, &RAMP), "list-cut");
}

#[test]
fn a_signal_past_three_is_three() {
    let full = network("Home", None, Some(3), "lit");
    let past = network("Home", None, Some(9), "lit");
    let face = |row| list_row_face(150, row, &RAMP).rgba;
    assert!(face(&past) == face(&full));
}

/// The bar `w` art pixels wide as omega-bar draws it with these actions and hints: the plate,
/// then the actions over it, on a face `TALL` high whose row `RISE` is the bar's top.
fn bar(w: u32, left: (&str, &str), right: (&str, &str), hints: &[Hint]) -> UndoFace {
    let left = action_face(Side::Left, left.0, left.1, &RAMP);
    let right = action_face(Side::Right, right.0, right.1, &RAMP);
    let layout = bar_layout(w, &left, &right);
    let mut face = omega_ui::kit::pixel::blank(w, TALL);
    let [x, y, pw, _] = layout.plate;
    over(&mut face, &plate_face(pw, hints, &RAMP), x, y + RISE);
    for (action, [ax, ay]) in [(&left, layout.left), (&right, layout.right)] {
        over(
            &mut face,
            &action.face,
            ax as u32,
            (ay + RISE as i32) as u32,
        );
    }
    face
}

#[test]
fn the_bar_is_chromes_at_640_and_720() {
    let shelf = (("QUICKMENU", "SELECT"), ("SETTINGS", "START"));
    let menu = (("RETURN", "B"), ("CONFIRM", "A"));
    let forget = [Hint {
        key: "Y",
        word: "FORGET",
    }];
    let two = [
        forget[0],
        Hint {
            key: "X",
            word: "RENAME",
        },
    ];
    for w in [320, 360] {
        matches(&bar(w, shelf.0, shelf.1, &[]), &format!("bar-shelf-{w}"));
        matches(&bar(w, menu.0, menu.1, &[]), &format!("bar-menu-{w}"));
        matches(&bar(w, menu.0, menu.1, &forget), &format!("bar-forget-{w}"));
        matches(&bar(w, menu.0, menu.1, &two), &format!("bar-two-{w}"));
    }
    let fetch = [Hint {
        key: "START",
        word: "FETCH",
    }];
    matches(&bar(320, menu.0, menu.1, &fetch), "bar-fetch-320");
}

#[test]
fn the_plate_runs_between_the_actions_and_under_their_ends() {
    let action = |side, word, pill| action_face(side, word, pill, &RAMP);
    let (quickmenu, settings) = (
        action(Side::Left, "QUICKMENU", "SELECT"),
        action(Side::Right, "SETTINGS", "START"),
    );
    let (back, confirm) = (
        action(Side::Left, "RETURN", "B"),
        action(Side::Right, "CONFIRM", "A"),
    );
    let shelf = bar_layout(320, &quickmenu, &settings);
    assert_eq!(shelf.plate, [124, PLATE_TOP, 79, PLATE_H]);
    assert_eq!((shelf.left, shelf.right), ([0, -6], [196, -6]));
    assert_eq!(
        bar_layout(360, &quickmenu, &settings).plate,
        [124, 3, 119, 18]
    );
    let menu = bar_layout(320, &back, &confirm);
    assert_eq!(menu.plate, [103, 3, 107, 18]);
    assert_eq!(menu.right, [203, -6]);
    assert_eq!(bar_layout(360, &back, &confirm).plate, [103, 3, 147, 18]);
    assert_eq!(PLATE_H + PLATE_TOP, BAR);
}

#[test]
fn hints_are_as_wide_as_their_caps_and_words() {
    let forget = Hint {
        key: "Y",
        word: "FORGET",
    };
    let rename = Hint {
        key: "X",
        word: "RENAME",
    };
    assert_eq!(cap_width("Y"), 9);
    assert_eq!(cap_width("START"), 33);
    assert_eq!(hint_width(&forget), 9 + 3 + 36);
    // ink to ink: the last word's blank pixel is not ink, and 8 pixels between two
    assert_eq!(hints_width(&[forget]), 47);
    assert_eq!(hints_width(&[forget, rename]), 47 + 8 + 47);
    assert_eq!(hints_width(&[Hint { key: "Y", word: "" }]), 9);
    assert_eq!(hints_width(&[]), 0);
    let plate = plate_face(107, &[forget], &RAMP);
    assert_eq!((plate.w, plate.h), (107, PLATE_H));
}

#[test]
fn the_bar_stands_on_the_screens_bottom_and_its_actions_sink_on_their_own() {
    let (left, right) = (
        action_face(Side::Left, "RETURN", "B", &RAMP),
        action_face(Side::Right, "CONFIRM", "A", &RAMP),
    );
    let layout = bar_layout(320, &left, &right);
    let tex = TexId::from_raw(3);
    assert_eq!(BarLayout::top(480), 438.0);
    assert_eq!(
        layout.plate_quad(tex, 480),
        quad(tex, 107, 18, 206.0, 444.0)
    );
    assert_eq!(
        layout.action_quad(Side::Left, tex, &left, 480, 0),
        quad(tex, 110, TALL, 0.0, 426.0)
    );
    assert_eq!(
        layout.action_quad(Side::Right, tex, &right, 720, SINK),
        quad(tex, 117, TALL, 406.0, 678.0 - 12.0 + 54.0)
    );
}

#[test]
fn a_change_of_actions_sinks_the_old_and_raises_the_new() {
    let swap = |new, sink| Swap { new, sink };
    assert_eq!(action_swap(0.0), swap(false, 0));
    // flow/player.js: even(p / 0.5 * 54) screen pixels, 13.5 art pixels rounding up
    assert_eq!(action_swap(0.25), swap(false, 14));
    assert_eq!(action_swap(0.49), swap(false, 26));
    assert_eq!(action_swap(0.5), swap(true, SINK));
    assert_eq!(action_swap(0.75), swap(true, 14));
    assert_eq!(action_swap(1.0), swap(true, 0));
    assert_eq!(action_swap(2.0), swap(true, 0));
    assert_eq!(SINK, TALL, "sunk that far an action is out of sight");
}

#[test]
fn lines_of_text_are_chromes() {
    let cases = [
        ("text-sftp", "SFTP uses the same login", RAMP.dim),
        ("text-offline", "Join a Wi-Fi network first.", RAMP.dim),
        (
            "text-labels",
            "Sign in with your free ScreenScraper account\nto fetch the real labels of your carts.",
            RAMP.dim,
        ),
        ("text-heading", "IN GAME", RAMP.dim),
        ("text-menu-twice", "MENU ×2", RAMP.frame_hi),
        ("text-does", "LOAD/SAVE STATE", RAMP.cream),
    ];
    for (name, text, colour) in cases {
        matches(&text_face(text, colour), name);
    }
}

/// flow.html draws the Quickmenu's ◀ ▶ ▲ ▼ as sprites copied from the full Pixel Code, which
/// the web font it is set in has no letter for; the face the crate embeds has them, pixel for
/// pixel the same.
#[test]
fn the_quickmenus_arrows_are_pixel_codes_own_as_flow_copies_them() {
    matches(
        &text_face("SELECT+▲▼", RAMP.frame_hi),
        "text-select-up-down",
    );
    matches(&text_face("◀▶", RAMP.frame_hi), "text-left-right");
}

/// What a well holds, laid over `panel`: rows from its top, then whatever is drawn after them.
fn hold(panel: &mut PanelFace, rows: &[Row]) {
    for (i, row) in rows.iter().enumerate() {
        let [x, y, w, _] = row_rect(panel.well, i as u32);
        over(&mut panel.face, &row_face(w, row, &RAMP), x, y);
    }
}

/// A note in a well, as flow.css's .note: 3 pixels in, `below` the rows above it.
fn note(panel: &mut PanelFace, text: &str, below: u32) {
    let [x, y, _, _] = panel.well;
    over(
        &mut panel.face,
        &text_face(text, RAMP.dim),
        x + 3,
        y + below,
    );
}

#[test]
fn ssh_is_flows() {
    let login = [
        ("IP", "192.168.1.20"),
        ("HOST", "rg35xxsp.local"),
        ("USER", "root"),
        ("PASSWORD", "root"),
        ("PORT", "22"),
    ];
    let rows: Vec<Row> = login
        .iter()
        .map(|&(label, value)| row(label, Some(value), ""))
        .collect();
    // the rows, then the note 4 below them on its line and 2 more under it
    let content = 5 * ROW_H + 4 + 10 + 2;
    let mut panel = panel_face(200, panel_height(content), "SSH", true, &RAMP);
    assert_eq!(panel.well[3], content);
    hold(&mut panel, &rows);
    note(&mut panel, "SFTP uses the same login", 5 * ROW_H + 4);
    matches(&panel.face, "flow-ssh");

    // offline, the note alone: 4 above its line and 5 under it
    let mut panel = panel_face(200, panel_height(4 + 10 + 5), "SSH", true, &RAMP);
    note(&mut panel, "Join a Wi-Fi network first.", 4);
    matches(&panel.face, "flow-ssh-offline");
}

#[test]
fn labels_is_flows() {
    let rows = [
        row("SCREENSCRAPER", Some("SIGN IN"), "lit"),
        row("FETCH LABELS", None, ""),
    ];
    let content = 2 * ROW_H + 4 + 2 * 10 + 2;
    let mut panel = panel_face(280, panel_height(content), "LABELS", true, &RAMP);
    hold(&mut panel, &rows);
    let text =
        "Sign in with your free ScreenScraper account\nto fetch the real labels of your carts.";
    note(&mut panel, text, 2 * ROW_H + 4);
    matches(&panel.face, "flow-labels");
}

/// The Quickmenu's two columns: a heading, then each button and what it does, a line and a
/// pixel apart; each column's buttons as many letters wide as `keys`, and 16 pixels between
/// the two columns.
#[test]
fn the_quickmenu_is_flows() {
    let shelf: [(&str, &str); 8] = [
        ("A", "PLAY"),
        ("HOLD A", "CLEAN START"),
        ("X", "CORE"),
        ("START", "SETTINGS"),
        ("SELECT", "QUICKMENU"),
        ("◀▶", "BROWSE"),
        ("▲▼", "LETTER"),
        ("L1 R1", "SHELF"),
    ];
    let in_game: [(&str, &str); 11] = [
        ("L2", "REWIND"),
        ("R2", "FAST FORWARD"),
        ("MENU ×2", "SAVE STATES"),
        ("HOLD MENU", "EJECT"),
        ("SELECT+MENU", "LINK"),
        ("SELECT+▲▼", "BRIGHTNESS"),
        ("SELECT+◀▶", "BLUE LIGHT"),
        ("SELECT+L1/R1", "LOAD/SAVE STATE"),
        ("SELECT+Y", "COLOUR"),
        ("VOL+ VOL-", "MUTE"),
        ("HOLD POWER", "POWER MENU"),
    ];
    // the heading's line and 2 below it, a pixel, then the lines; and a pixel under them all
    let content = 10 + 2 + 1 + 11 * 11 - 1 + 1;
    let mut panel = panel_face(304, panel_height(content), "QUICKMENU", true, &RAMP);
    let [wx, wy, ww, _] = panel.well;
    assert_eq!(ww, 108 + 16 + 168, "the widest line sets the panel's width");
    /// A button and what it does.
    type Line<'a> = (&'a str, &'a str);
    let columns: [(u32, &str, u32, &[Line]); 2] =
        [(0, "SHELF", 6, &shelf), (108 + 16, "IN GAME", 12, &in_game)];
    for (x, heading, keys, lines) in columns {
        over(&mut panel.face, &text_face(heading, RAMP.dim), wx + x, wy);
        for (i, (key, does)) in lines.iter().enumerate() {
            let y = wy + 13 + 11 * i as u32;
            over(&mut panel.face, &text_face(key, RAMP.frame_hi), wx + x, y);
            let does_x = wx + x + 6 * keys + 6;
            over(&mut panel.face, &text_face(does, RAMP.cream), does_x, y);
        }
    }
    matches(&panel.face, "flow-controls");
}

#[test]
fn rows_stand_in_a_well_a_pixel_into_its_sides() {
    let panel = panel_face(160, panel_height(3 * ROW_H), "CONFIGS", true, &RAMP);
    assert_eq!(panel.well, [6, 18, 148, 39]);
    assert_eq!(row_width(148), 150);
    assert_eq!(row_rect(panel.well, 0), [5, 18, 150, ROW_H]);
    assert_eq!(row_rect(panel.well, 2), [5, 18 + 2 * ROW_H, 150, ROW_H]);
    assert_eq!(panel_height(0), 23);
}
