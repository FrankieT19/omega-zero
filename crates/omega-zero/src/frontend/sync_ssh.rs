//! The SSH screen's lines, made again whenever what they say changes, on textures of their own
//! so the Network screen it slides against keeps its own.

use omega_gfx::Compositor;

use crate::app::App;
use crate::frontend::faces::{wifi_faces, WifiTexes};

pub(crate) fn sync_ssh(app: &mut App, compositor: &mut Compositor, state: &mut WifiTexes) {
    let Some(view) = app.ssh_view().map(|v| v.list()) else {
        state.view = None;
        return;
    };
    if state.view.as_ref() != Some(&view) {
        let faces = wifi_faces(&view, compositor, state);
        app.set_ssh_faces(faces);
        state.view = Some(view);
    }
}
