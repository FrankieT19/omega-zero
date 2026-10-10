//! The save-state switcher's photos and title, made when it opens and as its selection moves.

use omega_gfx::{Compositor, TexId};
use omega_store::format_stamp;
use omega_ui::{hint_face, photo_face, title_face};

use crate::app::{App, Phase};
use crate::frontend::faces::upload;

#[derive(Default)]
pub(crate) struct Switcher {
    open: bool,
    titled: Option<String>,
}

pub(crate) struct Faces<'a> {
    pub(crate) pool: &'a mut Vec<TexId>,
    pub(crate) title: &'a mut Option<TexId>,
    pub(crate) undo: &'a mut Option<TexId>,
}

pub(crate) fn sync_switcher(
    app: &mut App,
    compositor: &mut Compositor,
    texes: Faces,
    state: &mut Switcher,
) {
    if !matches!(app.phase(), Phase::Polaroids { .. }) {
        state.open = false;
        return;
    }
    if !state.open {
        state.open = true;
        state.titled = None;
        let faces: Vec<_> = app.polaroid_entries().iter().map(photo_face).collect();
        let ids = faces
            .iter()
            .enumerate()
            .map(|(i, f)| match texes.pool.get(i) {
                Some(id) => {
                    compositor.update_texture(*id, f.w, f.h, &f.rgba);
                    *id
                }
                None => {
                    let id = compositor.create_texture_nearest(f.w, f.h, &f.rgba);
                    texes.pool.push(id);
                    id
                }
            })
            .collect();
        app.set_polaroid_faces(ids);

        let label = app
            .undo_label()
            .map(|l| upload(compositor, texes.undo, hint_face("X", l)));
        app.set_undo_face(label);
    }
    if state.titled.as_deref() != app.polaroid_stamp() {
        state.titled = app.polaroid_stamp().map(str::to_string);
        let face = title_face(&app.polaroid_title(&format_stamp(app.wall_secs())));
        let id = upload(compositor, texes.title, face);
        app.set_polaroid_title_face(id);
    }
}
