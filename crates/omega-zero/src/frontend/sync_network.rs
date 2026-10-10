//! The Network screen's list, made again whenever what it shows changes.

use omega_gfx::Compositor;

use crate::app::App;
use crate::frontend::faces::{wifi_faces, WifiTexes};

pub(crate) fn sync_network(app: &mut App, compositor: &mut Compositor, state: &mut WifiTexes) {
    if let Some(view) = app.network_view() {
        if state.view.as_ref() != Some(&view) {
            let faces = wifi_faces(&view, compositor, state);
            app.set_network_faces(faces);
            state.view = Some(view);
        }
    } else {
        state.view = None;
    }
}
