use omega_store::{parse_stamp, FF_SPEEDS};
use omega_ui::{
    date_time_text, quick_caret_face, quick_label_face, quick_rows_shown, quick_top_of,
    quick_value_face, quick_window, stage_y, ClockPicker, QuickRow, QuickValue, UndoFace, MENU_PAD,
    QUICK_PITCH,
};

fn inked(f: &UndoFace, x: u32, y: u32) -> bool {
    f.rgba[((y * f.w + x) * 4 + 3) as usize] > 128
}

fn ink_columns(f: &UndoFace) -> (u32, u32) {
    let cols: Vec<u32> = (0..f.w)
        .filter(|&x| (0..f.h).any(|y| inked(f, x, y)))
        .collect();
    (
        *cols.first().expect("an empty face"),
        *cols.last().expect("an empty face"),
    )
}

#[test]
fn the_type_sits_exactly_menu_pad_in_from_both_sides_of_its_face() {
    for f in [
        quick_label_face(QuickRow::FastForwardSound),
        quick_label_face(QuickRow::Rumble),
        quick_value_face("Off", false),
        quick_value_face(QuickValue::Speed6.text(), true),
        quick_value_face("SEP 15 16:35", true),
    ] {
        let (first, last) = ink_columns(&f);
        let right = f.w - 1 - MENU_PAD;
        assert!(
            (MENU_PAD..MENU_PAD + 4).contains(&first),
            "ink starts at {first} of {}",
            f.w
        );
        assert!(
            (right - 4..=right).contains(&last),
            "ink ends at {last} of {}",
            f.w
        );
    }
}

#[test]
fn a_long_label_is_set_as_large_as_a_short_one() {
    let tall = |f: &UndoFace| {
        (0..f.h)
            .filter(|&y| (0..f.w).any(|x| inked(f, x, y)))
            .count()
    };
    let long = tall(&quick_label_face(QuickRow::FastForwardSound));
    let short = tall(&quick_label_face(QuickRow::Rumble));
    assert!(long + 1 >= short, "{long} rows of ink against {short}");
}

#[test]
fn the_fast_forward_row_offers_the_four_ceilings_the_card_can_hold() {
    assert_eq!(FF_SPEEDS, [2, 3, 4, 6]);
    assert_eq!(
        FF_SPEEDS.map(QuickValue::speed),
        [
            Some(QuickValue::Speed2),
            Some(QuickValue::Speed3),
            Some(QuickValue::Speed4),
            Some(QuickValue::Speed6),
        ]
    );
    for other in [1, 5, 7, 9, 16, 28, 255] {
        assert_eq!(
            QuickValue::speed(other),
            None,
            "{other}x is not a value the row has"
        );
    }
}

#[test]
fn the_rows_run_in_the_order_the_user_chose() {
    let labels = QuickRow::ALL.map(QuickRow::label);
    assert_eq!(
        labels,
        [
            "Fast Forward",
            "Fast Forward Sound",
            "Colour Correction",
            "Rumble",
            "Background",
            "Wi-Fi",
            "Date & Time",
            "About"
        ]
    );
    let opens: Vec<QuickRow> = QuickRow::ALL.into_iter().filter(|r| r.opens()).collect();
    assert_eq!(
        opens,
        [
            QuickRow::Background,
            QuickRow::Wifi,
            QuickRow::DateTime,
            QuickRow::About
        ]
    );
}

#[test]
fn every_row_shows_where_they_fit_and_the_rest_scroll_into_view() {
    let all = QuickRow::ALL.len();
    let shown = quick_rows_shown(all);
    assert!(shown >= 7, "only {shown} rows fit");
    assert!(
        quick_top_of(shown) + QUICK_PITCH * shown as f32 <= 427.0 + stage_y(),
        "the last row in view runs into the legend"
    );
    assert_eq!(quick_rows_shown(3), 3, "a short menu scrolled");
    if shown < all {
        assert_eq!(quick_window(0, all, 0), 0);
        assert_eq!(
            quick_window(all - 1, all, 0),
            all - shown,
            "the last row is out of view"
        );
        assert_eq!(
            quick_window(all - 2, all, all - shown),
            all - shown,
            "the list moved though the lit row was in view"
        );
        assert_eq!(
            quick_window(0, all, all - shown),
            0,
            "the first row is out of view"
        );
    } else {
        assert_eq!(quick_window(all - 1, all, 0), 0);
    }
}

#[test]
fn a_hidden_row_is_stepped_over_both_ways() {
    let rows: Vec<QuickRow> = QuickRow::ALL
        .into_iter()
        .filter(|r| *r != QuickRow::Wifi)
        .collect();
    assert_eq!(QuickRow::Background.down_in(&rows), QuickRow::DateTime);
    assert_eq!(QuickRow::DateTime.up_in(&rows), QuickRow::Background);
    assert_eq!(QuickRow::About.down_in(&rows), QuickRow::FastForward);
    assert_eq!(QuickRow::Background.down(), QuickRow::Wifi);
}

#[test]
fn the_values_read_as_the_menu_prints_them() {
    assert_eq!(
        QuickValue::ALL.map(QuickValue::text),
        ["2×", "3×", "4×", "6×", "On", "Off"]
    );
    assert_eq!(QuickValue::flag(true), QuickValue::On);
    assert_eq!(QuickValue::flag(false), QuickValue::Off);
}

#[test]
fn the_date_and_time_read_as_a_month_a_day_and_the_carousels_24_hour_clock() {
    let at = |stamp: &str| date_time_text(parse_stamp(stamp).expect("a stamp"));
    assert_eq!(at("2026-09-15_16-35-00"), "SEP 15 16:35");
    assert_eq!(at("2027-01-05_04-07-59"), "JAN 5 04:07");
}

#[test]
fn a_picker_for_a_set_clock_starts_at_the_local_time_and_its_offset() {
    let utc = parse_stamp("2026-09-15_21-35-42").expect("a stamp");
    let p = ClockPicker::local(utc, -300);
    assert_eq!(p.offset_min(), -300);
    assert_eq!(
        p.secs(),
        utc - 42,
        "the picker does not show the minute it was opened in"
    );
    assert!(p.text().starts_with("2026-09-15 16:35"), "{}", p.text());
}

#[test]
fn a_value_is_grey_until_its_row_is_in_hand() {
    let inkiest = |lit| {
        let f = quick_value_face("4×", lit);
        f.rgba
            .chunks(4)
            .max_by_key(|p| p[3])
            .map(|p| [p[0], p[1], p[2]])
            .expect("an empty face")
    };
    assert_eq!(inkiest(true), [0xf6, 0xf4, 0xef]);
    assert_eq!(inkiest(false), [0x9a, 0x9a, 0xa4]);
}

#[test]
fn the_arrows_are_faces_the_height_of_a_value() {
    for right in [false, true] {
        let caret = quick_caret_face(right);
        assert!(
            caret.w > 0 && caret.rgba.chunks(4).any(|p| p[3] > 0),
            "an empty arrow"
        );
        assert_eq!(caret.h, quick_value_face("On", true).h);
    }
}
