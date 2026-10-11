use omega_ui::{sticker_lines, StickerFields};

fn fields() -> StickerFields<'static> {
    StickerFields {
        battery: Some(87),
        serial: "0473885",
        dirty_digit: '0',
    }
}

#[test]
fn the_headline_rows_read_as_a_device_plate() {
    let all = sticker_lines(&fields()).join("\n");
    assert!(all.contains("AGS-102"), "{all}");
    assert!(all.contains("5V"), "{all}");
    assert!(all.contains("1.5A"), "{all}");
    assert!(all.contains("87"), "the gauge reading is missing: {all}");
    assert!(all.contains("0473885"), "the serial went missing: {all}");
}

#[test]
fn the_input_row_carries_the_real_dc_symbol() {
    let all = sticker_lines(&fields()).join("\n");
    assert!(all.contains(omega_ui::DC), "the rating row lost its symbol");
    assert_eq!(omega_ui::DC, '\u{2393}');
}

#[test]
fn a_missing_gauge_is_not_drawn_as_empty() {
    let mut f = fields();
    f.battery = None;
    let all = sticker_lines(&f).join("\n");
    assert!(
        !all.contains("0%"),
        "no gauge was drawn as a flat battery: {all}"
    );
    assert!(
        all.contains("BATTERY"),
        "the row should still be there: {all}"
    );
}

#[test]
fn the_compliance_block_is_the_credits() {
    let all = sticker_lines(&fields()).join("\n").to_uppercase();
    for owed in [
        "GPL-3.0",
        "MGBA",
        "GPSP",
        "BASEOS BY PVAIBHAV",
        "PIXEL CODE",
        "NERD",
        "LCD3X",
        "CLAUDE",
    ] {
        assert!(all.contains(owed), "the credits do not mention {owed}");
    }
}

#[test]
fn the_serial_row_matches_the_encoded_hash() {
    let all = sticker_lines(&fields()).join("\n");
    assert!(all.contains("0473885"), "{all}");
}

#[test]
fn every_line_is_already_upper_case() {
    for line in sticker_lines(&fields()) {
        assert_eq!(line, line.to_uppercase(), "{line}");
    }
}

#[test]
fn the_credits_name_the_author_and_end_on_slot() {
    let lines = sticker_lines(&fields());
    let all = lines.join("\n");
    assert!(all.contains("OT\u{c1}VIO (@OAURAMOS)"), "{all}");
    assert_eq!(
        lines.last().map(String::as_str),
        Some(omega_ui::BASED_ON),
        "slot's credit is not the last line"
    );
    assert!(omega_ui::BASED_ON.contains("SLOT BY BRANDON T. KOWALSKI"));
}

/// The colours a pixel of the sticker may be: clear, or one of the label's flat inks.
const INKS: [[u8; 3]; 3] = [[0x23, 0x1f, 0x20], [0xff, 0xff, 0xff], [0x8a, 0x86, 0x87]];

#[test]
fn the_sticker_is_drawn_in_flat_colours_without_soft_edges() {
    let face = omega_ui::sticker_face(&fields());
    for (i, px) in face.rgba.chunks_exact(4).enumerate() {
        let (x, y) = (i as u32 % face.w, i as u32 / face.w);
        match px[3] {
            0 => {}
            255 => assert!(
                INKS.contains(&[px[0], px[1], px[2]]),
                "{x},{y} is {px:?}, between the label's colours"
            ),
            a => panic!("{x},{y} is part covered, at alpha {a}"),
        }
    }
}

#[test]
fn the_sticker_is_its_face_at_twice_on_whole_art_pixels_above_the_bar() {
    let (x, y, w, h) = omega_ui::sticker_rect();
    let (cw, ch) = (omega_ui::out_w() as f32, omega_ui::out_h() as f32);
    assert!(x >= 20.0 && x + w <= cw - 20.0, "{x}+{w} on a {cw} canvas");
    let bar_top = omega_ui::kit::bar::BarLayout::top(ch as u32);
    assert!(
        y >= 0.0 && y + h <= bar_top,
        "{y}+{h} reaches the bar at {bar_top}"
    );
    let face = omega_ui::sticker_face(&fields());
    assert_eq!(
        (w, h),
        (face.w as f32 * 2.0, face.h as f32 * 2.0),
        "the sticker is not its face at 2x"
    );
    assert!(
        [x, y].iter().all(|v| v % 2.0 == 0.0),
        "the sticker is off the art grid at {x},{y}"
    );
}
