use crate::common;

use omega_input::{Action, Btn, Millis, RawEvent};
use omega_zero::app::Phase;
use omega_zero::screens::controls::ControlsView;
use omega_zero::session::Session;

fn on_shelf() -> (tempfile::TempDir, Session) {
    let d = common::tmp_root_with_carts(&["Emerald", "Fusion"]);
    let (s, _motor) = common::session_with_platform(d.path());
    assert!(
        matches!(s.app().phase(), Phase::Shelf),
        "not on the shelf: {:?}",
        s.app().phase()
    );
    (d, s)
}

fn feed(s: &mut Session, at: Millis, events: &[RawEvent]) {
    s.feed(events.iter().copied(), at);
}

fn tap_select(s: &mut Session, at: Millis) {
    feed(s, at, &[RawEvent::Down(Btn::Select)]);
    feed(s, at + 80, &[RawEvent::Up(Btn::Select)]);
    feed(s, at + 200, &[]);
}

#[test]
fn a_select_tap_on_the_shelf_opens_controls_and_another_closes_it() {
    let (_d, mut s) = on_shelf();
    tap_select(&mut s, 1_000);
    assert!(
        matches!(s.app().phase(), Phase::Controls),
        "a SELECT tap left {:?}",
        s.app().phase()
    );
    tap_select(&mut s, 2_000);
    assert!(
        matches!(s.app().phase(), Phase::Shelf),
        "a second SELECT tap left {:?}",
        s.app().phase()
    );
}

#[test]
fn b_closes_controls() {
    let (_d, mut s) = on_shelf();
    tap_select(&mut s, 1_000);
    assert!(matches!(s.app().phase(), Phase::Controls));
    s.app_mut().apply(Action::GbaDown(Btn::B));
    assert!(matches!(s.app().phase(), Phase::Shelf));
}

#[test]
fn a_select_held_for_the_game_opens_nothing() {
    let (_d, mut s) = on_shelf();
    feed(&mut s, 1_000, &[RawEvent::Down(Btn::Select)]);
    feed(&mut s, 1_700, &[]);
    feed(&mut s, 1_800, &[RawEvent::Up(Btn::Select)]);
    feed(&mut s, 2_000, &[]);
    assert!(
        matches!(s.app().phase(), Phase::Shelf),
        "a held SELECT left {:?}",
        s.app().phase()
    );
}

#[test]
fn a_select_chord_opens_nothing() {
    let (_d, mut s) = on_shelf();
    let before = s.app().brightness();
    feed(&mut s, 1_000, &[RawEvent::Down(Btn::Select)]);
    feed(&mut s, 1_050, &[RawEvent::Down(Btn::Up)]);
    feed(&mut s, 1_100, &[RawEvent::Up(Btn::Up)]);
    feed(&mut s, 1_150, &[RawEvent::Up(Btn::Select)]);
    feed(&mut s, 1_400, &[]);
    assert_ne!(s.app().brightness(), before, "the chord did not fire");
    assert!(
        matches!(s.app().phase(), Phase::Shelf),
        "SELECT+Up left {:?}",
        s.app().phase()
    );
}

#[test]
fn a_select_tap_in_a_game_opens_nothing() {
    let d = common::tmp_root_with_carts(&["Emerald", "Fusion"]);
    let mut a = common::app_playing_in(d.path(), "Emerald");
    a.apply(Action::GbaDown(Btn::Select));
    a.apply(Action::SelectTap);
    assert!(
        matches!(a.phase(), Phase::Playing { .. }),
        "a SELECT tap left the game for {:?}",
        a.phase()
    );
}

#[test]
fn controls_says_what_the_shelf_and_a_game_do_with_each_button() {
    let view = ControlsView::new();
    assert_eq!(view.title, "Quickmenu");
    let names: Vec<_> = view.groups.iter().map(|g| g.name).collect();
    assert_eq!(names, ["Shelf", "In Game"]);
    assert_eq!(
        view.groups[0].lines,
        [
            ("A", "Play"),
            ("Hold A", "Clean start"),
            ("X", "Core picker (GBA)"),
            ("START", "Settings"),
            ("SELECT", "Quickmenu"),
            ("◀▶", "Browse"),
            ("▲▼", "Letter jump"),
            ("L1/R1", "Switch shelf"),
        ]
    );
    assert_eq!(
        view.groups[1].lines,
        [
            ("L2", "Rewind"),
            ("R2", "Fast forward"),
            ("R2 ×2", "Latch fast forward"),
            ("MENU ×2", "Save states"),
            ("Hold MENU", "Eject"),
            ("SELECT+MENU", "Link"),
            ("SELECT+▲▼", "Brightness"),
            ("SELECT+◀▶", "Blue light"),
            ("SELECT+L1", "Load state"),
            ("SELECT+R1", "Save state"),
            ("SELECT+Y", "Colour correction"),
            ("VOL+ VOL-", "Mute, together"),
            ("Hold POWER", "Power menu"),
            ("L1/R1", "Picture size (GB)"),
        ]
    );
    for (keys, does) in view.groups.iter().flat_map(|g| g.lines.iter()) {
        assert!(
            omega_ui::text::label_font().is_some_and(|font| format!("{keys}  {does}")
                .chars()
                .all(|c| c == ' ' || font.lookup_glyph_index(c) != 0)),
            "{keys} {does} has a character the label font has no glyph for"
        );
    }
}
