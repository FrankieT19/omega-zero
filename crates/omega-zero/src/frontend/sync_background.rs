//! The picture of the pattern the shelf is drawn over, and the Background screen's list.

use omega_gfx::{Compositor, TexId};

use crate::app::App;
use crate::frontend::faces::{upload, wifi_faces, WifiTexes};

/// The picture of the pattern the shelf is drawn over, made again when the style or colour
/// changes.
#[derive(Default)]
pub(crate) struct BackdropTex {
    tex: Option<TexId>,
    built: Option<(omega_store::BackdropStyle, u8)>,
}

pub(crate) fn sync_background(
    app: &mut App,
    compositor: &mut Compositor,
    pattern: &mut BackdropTex,
    list: &mut WifiTexes,
) {
    let want = app.backdrop_pattern();
    if pattern.built != want {
        if let Some((style, colour)) = want {
            if let Some(face) = omega_ui::backdrop_pattern_face(style, colour) {
                let w = face.w;
                let tex = upload(compositor, &mut pattern.tex, face);
                app.set_backdrop_face(tex, w, style, colour);
            }
        }
        pattern.built = want;
    }
    let Some(view) = app.background_view() else {
        list.view = None;
        return;
    };
    if list.view.as_ref() != Some(&view) {
        let faces = wifi_faces(&view, compositor, list);
        app.set_background_faces(faces);
        list.view = Some(view);
    }
}
