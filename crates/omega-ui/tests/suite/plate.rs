use omega_ui::{
    hint_face, hint_width, title_face, UndoFace, CAP_GAP, HINT_GAP, HINT_H, TITLE_H, TITLE_W,
};

fn opaque(face: &UndoFace, x: u32, y: u32) -> bool {
    face.rgba[((y * face.w + x) * 4 + 3) as usize] > 0
}

fn ink(face: &UndoFace, x: u32, y: u32) -> bool {
    let i = ((y * face.w + x) * 4) as usize;
    face.rgba[i] > 128 && face.rgba[i + 3] > 128
}

#[test]
fn the_hint_is_a_key_cap_and_type_on_nothing_else() {
    let face = hint_face("B", "Back");
    assert_eq!((face.w, face.h), (hint_width("B", "Back"), HINT_H));
    assert!(opaque(&face, 1, HINT_H / 2), "the key cap is not filled");
    assert!(
        !opaque(&face, face.w - 1, 0),
        "the hint has a background behind its type"
    );
}

#[test]
fn the_label_is_drawn_and_stays_inside_the_hint() {
    for label in ["Back", "Undo save", "Undo an interminable action"] {
        let face = hint_face("X", label);
        assert!(
            (0..face.h).any(|y| (0..face.w).any(|x| ink(&face, x, y))),
            "{label} was never drawn"
        );
        for y in 0..face.h {
            assert!(!ink(&face, face.w - 1, y), "{label} overran its face");
        }
    }
}

#[test]
fn the_title_is_drawn_and_stays_inside_its_face() {
    for text in [
        "just now",
        "4 min ago",
        "2026-08-08 20:00",
        "2026-08-09_14-32-05",
    ] {
        let face = title_face(text);
        assert_eq!((face.w, face.h), (TITLE_W, TITLE_H));
        assert!(
            (0..face.h).any(|y| (0..face.w).any(|x| ink(&face, x, y))),
            "{text} was never drawn"
        );
        for y in 0..face.h {
            assert!(!ink(&face, face.w - 1, y), "{text} overran its face");
        }
    }
}

#[test]
fn grouping_reads_as_pairs() {
    let inside = std::hint::black_box(CAP_GAP) as f32;
    let between = std::hint::black_box(HINT_GAP);
    assert!(
        between >= inside * 2.0,
        "gap inside a pair is {CAP_GAP}px and between pairs {HINT_GAP}px: they will not group"
    );
    assert!(inside >= 4.0, "the cap is crowding its word at {inside}px");
}
