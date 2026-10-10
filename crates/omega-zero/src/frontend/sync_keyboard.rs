//! The on-screen keyboard's faces: its keys, made once, and what it asks for and what has been
//! typed, made again as they change.

use omega_gfx::Compositor;
use omega_ui::{
    bottom_key_face, bottom_label, field_face, hint_face, key_glyph_face, key_rect, prompt_face,
    KeyboardFaces, UndoFace, BOTTOM, KEY_ROWS, SHIFT_INK,
};

use crate::app::App;
use crate::frontend::faces::{upload, WifiTexes};

pub(crate) fn sync_keyboard(app: &mut App, compositor: &mut Compositor, state: &mut WifiTexes) {
    let Some(typing) = app.typing() else {
        state.typed = None;
        return;
    };
    let typed = (typing.prompt, typing.keys.text.clone(), typing.refused);
    if !app.keyboard_faces_ready() {
        let faces = keyboard_faces(compositor);
        app.set_keyboard_faces(faces);
    }
    if state.typed.as_ref() != Some(&typed) {
        let prompt = prompt_face(&typed.0);
        let field = field_face(&typed.1);
        let (p, f) = ((prompt.w, prompt.h), (field.w, field.h));
        let prompt = upload(compositor, &mut state.prompt, prompt);
        let field = upload(compositor, &mut state.field, field);
        app.set_typing_faces((prompt, p.0, p.1), (field, f.0, f.1));
        state.typed = Some(typed);
    }
}

fn keyboard_faces(compositor: &mut Compositor) -> KeyboardFaces {
    let mut up = |f: UndoFace| (compositor.create_texture(f.w, f.h, &f.rgba), f.w, f.h);
    let glyphs = (' '..='~').map(|c| up(key_glyph_face(c))).collect();
    let mut bottom = Vec::new();
    for (col, key) in BOTTOM.iter().enumerate() {
        let w = key_rect(KEY_ROWS - 1, col)[2] as u32;
        for layer in [omega_ui::Layer::Lower, omega_ui::Layer::Symbols] {
            let label = bottom_label(*key, layer);
            if !bottom.iter().any(|(l, _)| *l == label) {
                bottom.push((label, up(bottom_key_face(label, w, omega_ui::MENU_INK))));
            }
        }
    }
    let shift_w = key_rect(KEY_ROWS - 1, 0)[2] as u32;
    let shift_on = up(bottom_key_face("Shift", shift_w, SHIFT_INK));
    let legend = [
        hint_face("B", "Delete"),
        hint_face("X", "Shift"),
        hint_face("Y", "Space"),
        hint_face("START", "Done"),
    ]
    .into_iter()
    .map(|f| (compositor.create_texture(f.w, f.h, &f.rgba), f.w))
    .collect();
    KeyboardFaces {
        glyphs,
        bottom,
        shift_on,
        legend,
    }
}
