use omega_ui::{
    out_w, plain_face, signal_glyph, wifi_rows_shown, wifi_window, Draw, TexId, WifiMenuFaces,
    WifiRowFaces, MENU_PAD, QUICK_PITCH,
};

#[test]
fn each_strength_has_its_own_glyph_with_and_without_a_lock() {
    let mut glyphs: Vec<char> = (1..=4)
        .flat_map(|b| [signal_glyph(b, false), signal_glyph(b, true)])
        .collect();
    glyphs.sort();
    glyphs.dedup();
    assert_eq!(glyphs.len(), 8);
    assert_eq!(
        signal_glyph(0, false),
        signal_glyph(1, false),
        "no bars still shows one"
    );
    assert_eq!(signal_glyph(9, true), signal_glyph(4, true));
}

#[test]
fn the_list_shows_at_least_six_rows_and_scrolls_only_as_far_as_it_must() {
    let shown = wifi_rows_shown();
    assert!(shown >= 6, "only {shown} rows fit");
    assert_eq!(wifi_window(3, 4, 0), 0, "a short list scrolled");
    assert_eq!(
        wifi_window(shown, shown + 4, 0),
        1,
        "the lit row went below the list"
    );
    assert_eq!(
        wifi_window(shown, shown + 4, 2),
        2,
        "the window moved though the row was in it"
    );
    assert_eq!(
        wifi_window(0, shown + 4, 3),
        0,
        "the lit row went above the list"
    );
    assert_eq!(wifi_window(shown + 3, shown + 4, 0), 4);
}

#[test]
fn a_long_network_name_ends_in_an_ellipsis_within_its_room() {
    let room = 300.0;
    let short = plain_face("Home", room, 27.0, 40, true);
    let long = plain_face(&"Very long network name ".repeat(4), room, 27.0, 40, true);
    assert!(short.w < long.w);
    assert!(long.w as f32 <= room, "{} is wider than {room}", long.w);
    let lit = plain_face("Home", room, 27.0, 40, true);
    let dim = plain_face("Home", room, 27.0, 40, false);
    assert_eq!((lit.w, lit.h), (dim.w, dim.h));
    assert_ne!(lit.rgba, dim.rgba, "a lit name looks like a dim one");
}

fn face(id: usize, w: u32) -> (TexId, u32, u32) {
    (TexId::from_raw(id), w, 40)
}

#[test]
fn the_rows_status_and_legend_are_drawn_in_order_down_the_screen() {
    let rows = (0..wifi_rows_shown())
        .map(|i| WifiRowFaces {
            label: face(10 + i, 200),
            value: Some(face(30 + i, 80)),
            icon: Some(face(50 + i, 28)),
            setting: i < 2,
        })
        .collect();
    let faces = WifiMenuFaces {
        rows,
        lit: Some(0),
        status: vec![face(70, 400), face(71, 500)],
        legend: vec![(TexId::from_raw(80), 90), (TexId::from_raw(81), 90)],
        carets: [face(90, 10), face(91, 10)],
    };
    let mut out = Vec::new();
    faces.draw_over(&mut out);
    let at = |id: usize| {
        out.iter()
            .find_map(|d| match *d {
                Draw::Tex {
                    x, y, w, h, tex, ..
                } if tex == TexId::from_raw(id) => Some([x, y, w, h]),
                _ => None,
            })
            .unwrap_or_else(|| panic!("{id} was not drawn"))
    };
    let last = at(10 + wifi_rows_shown() - 1);
    let (status, legend) = (at(70), at(80));
    assert!(
        last[1] + QUICK_PITCH <= status[1] + 8.0,
        "the list runs into the status"
    );
    assert!(at(71)[1] > status[1]);
    assert!(
        at(71)[1] + 26.0 <= legend[1],
        "the status runs into the legend"
    );
    for i in 0..wifi_rows_shown() {
        let icon = at(50 + i);
        assert!(
            icon[0] + icon[2] <= out_w() as f32,
            "row {i}'s signal is off the side"
        );
        assert!(
            at(30 + i)[0] + at(30 + i)[2] - MENU_PAD as f32 <= icon[0],
            "row {i}'s value runs into its signal"
        );
    }
    assert!(
        at(90)[0] < at(30)[0] && at(91)[0] > at(30)[0],
        "the lit setting has no carets"
    );
}
