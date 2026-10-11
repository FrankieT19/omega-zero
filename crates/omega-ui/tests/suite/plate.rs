use omega_ui::{hint_face, hint_width, UndoFace, CAP, CAP_GAP, HINT_H};

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
fn the_cap_keeps_clear_of_its_word() {
    let inside = std::hint::black_box(CAP_GAP);
    assert!(inside >= 4, "the cap is crowding its word at {inside}px");
}
