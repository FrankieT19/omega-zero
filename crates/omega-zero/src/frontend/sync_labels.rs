//! The Labels screen's list, made again whenever what it shows changes.

use omega_gfx::Compositor;

use crate::app::App;
use crate::frontend::faces::{wifi_faces, WifiTexes};

pub(crate) fn sync_labels(app: &mut App, compositor: &mut Compositor, state: &mut WifiTexes) {
    let Some(view) = app.labels_view() else {
        state.view = None;
        return;
    };
    if state.view.as_ref() != Some(&view) {
        let faces = wifi_faces(&view, compositor, state);
        app.set_labels_faces(faces);
        state.view = Some(view);
    }
}
