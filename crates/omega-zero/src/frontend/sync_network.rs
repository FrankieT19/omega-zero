//! The Wi-Fi screen's list, made again whenever what it shows changes.

use omega_gfx::Compositor;

use crate::app::App;
use crate::frontend::faces::{wifi_faces, WifiTexes};

pub(crate) fn sync_wifi(app: &mut App, compositor: &mut Compositor, state: &mut WifiTexes) {
    if let Some(view) = app.wifi_view() {
        if state.view.as_ref() != Some(&view) {
            let faces = wifi_faces(&view, compositor, state);
            app.set_wifi_faces(faces);
            state.view = Some(view);
        }
    } else {
        state.view = None;
    }
}
