use slot_ui::{
    key_on, key_rect, out_h, out_w, row_len, stage_y, Key, Keyboard, Layer, BOTTOM, KEY_COLS,
    KEY_ROWS, MAX_TEXT,
};

const LAYERS: [Layer; 3] = [Layer::Lower, Layer::Upper, Layer::Symbols];

#[test]
fn every_printable_ascii_character_is_on_some_layer() {
    let mut typed: Vec<char> = Vec::new();
    for layer in LAYERS {
        for row in 0..KEY_ROWS {
            for col in 0..row_len(row) {
                match key_on(layer, row, col) {
                    Key::Char(c) => typed.push(c),
                    Key::Space => typed.push(' '),
                    _ => {}
                }
            }
        }
    }
    for c in ' '..='~' {
        assert!(typed.contains(&c), "{c:?} cannot be typed");
    }
    assert!(
        typed.iter().all(|c| (' '..='~').contains(c)),
        "a key types something wpa_supplicant would refuse"
    );
}

#[test]
fn the_rows_of_characters_are_full_and_the_last_row_changes_what_they_type() {
    for row in 0..KEY_ROWS - 1 {
        assert_eq!(row_len(row), KEY_COLS);
    }
    assert_eq!(row_len(KEY_ROWS - 1), BOTTOM.len());
    for layer in LAYERS {
        for (col, key) in BOTTOM.iter().enumerate() {
            assert_eq!(key_on(layer, KEY_ROWS - 1, col), *key);
        }
    }
}

#[test]
fn every_key_is_on_the_canvas_above_the_legend_and_none_overlap() {
    let mut rects = Vec::new();
    for row in 0..KEY_ROWS {
        for col in 0..row_len(row) {
            let [x, y, w, h] = key_rect(row, col);
            assert!(
                x >= 0.0 && x + w <= out_w() as f32,
                "key {row},{col} is off the side"
            );
            assert!(
                y >= 0.0 && y + h <= 427.0 + stage_y(),
                "key {row},{col} meets the legend"
            );
            assert!(y + h <= out_h() as f32);
            rects.push([x, y, w, h]);
        }
    }
    for (i, a) in rects.iter().enumerate() {
        for b in &rects[i + 1..] {
            let apart = a[0] + a[2] <= b[0]
                || b[0] + b[2] <= a[0]
                || a[1] + a[3] <= b[1]
                || b[1] + b[3] <= a[1];
            assert!(apart, "{a:?} overlaps {b:?}");
        }
    }
}

#[test]
fn the_bottom_row_spans_the_rows_above_it() {
    let first = key_rect(0, 0);
    let last = key_rect(0, KEY_COLS - 1);
    let left = key_rect(KEY_ROWS - 1, 0);
    let right = key_rect(KEY_ROWS - 1, BOTTOM.len() - 1);
    assert_eq!(left[0], first[0]);
    assert_eq!(right[0] + right[2], last[0] + last[2]);
}

#[test]
fn the_cursor_starts_on_q_and_wraps_round_each_row_and_column() {
    let mut k = Keyboard::default();
    assert_eq!(k.key(), Key::Char('q'));
    k.left();
    assert_eq!(k.key(), Key::Char('p'));
    k.right();
    assert_eq!(k.key(), Key::Char('q'));
    k.up();
    k.up();
    assert_eq!(
        k.row,
        KEY_ROWS - 1,
        "up from the top did not wrap to the bottom"
    );
    k.down();
    assert_eq!(k.row, 0);
}

#[test]
fn moving_between_rows_lands_on_the_key_nearest_below() {
    let mut k = Keyboard::default();
    k.down();
    k.down();
    for _ in 0..4 {
        k.right();
    }
    assert_eq!(k.key(), Key::Char('b'));
    k.down();
    assert_eq!(k.key(), Key::Space, "b sits over the space bar");
    k.right();
    assert_eq!(k.key(), Key::Delete);
    k.up();
    assert_eq!(
        k.key(),
        Key::Char('_'),
        "the cursor did not go back up near Delete"
    );
}

#[test]
fn shift_and_symbols_change_what_the_rows_type_and_go_back() {
    let mut k = Keyboard::default();
    k.shift();
    assert_eq!(k.key(), Key::Char('Q'));
    k.shift();
    assert_eq!(k.key(), Key::Char('q'));
    k.symbols();
    assert_eq!(k.layer, Layer::Symbols);
    k.shift();
    assert_eq!(
        k.layer,
        Layer::Lower,
        "shift on symbols did not go back to letters"
    );
    k.symbols();
    k.symbols();
    assert_eq!(k.layer, Layer::Lower);
}

#[test]
fn pressing_types_changes_layer_or_finishes() {
    let mut k = Keyboard::default();
    assert!(!k.press());
    assert_eq!(k.text, "q");
    k.row = KEY_ROWS - 1;
    k.col = BOTTOM.iter().position(|b| *b == Key::Shift).unwrap();
    assert!(!k.press());
    assert_eq!(k.layer, Layer::Upper);
    k.col = BOTTOM.iter().position(|b| *b == Key::Space).unwrap();
    k.press();
    k.col = BOTTOM.iter().position(|b| *b == Key::Delete).unwrap();
    k.press();
    k.press();
    assert_eq!(k.text, "");
    k.col = BOTTOM.iter().position(|b| *b == Key::Done).unwrap();
    assert!(k.press(), "Join did not finish");
}

#[test]
fn the_field_takes_no_more_than_wpa_supplicant_will() {
    let mut k = Keyboard::default();
    for _ in 0..MAX_TEXT + 5 {
        k.press();
    }
    assert_eq!(k.text.len(), MAX_TEXT);
}
