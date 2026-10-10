//! Settings' faces that change while it is open: the date and time, and what it says beside
//! Network.

use omega_gfx::{out_w, Compositor, TexId};
use omega_ui::{date_time_text, plain_face, quick_value_face};

use crate::app::App;
use crate::frontend::faces::{upload, WifiTexes};
use crate::wifi_screen::Summary;

#[derive(Default)]
pub(crate) struct SettingsClock {
    dim: Option<TexId>,
    lit: Option<TexId>,
    shown: String,
}

pub(crate) fn sync_settings_clock(
    app: &mut App,
    compositor: &mut Compositor,
    state: &mut SettingsClock,
) {
    if app.settings_row().is_none() {
        return;
    }
    let text = date_time_text(app.wall_secs());
    if text == state.shown {
        return;
    }
    let (dim, lit) = (
        quick_value_face(&text, false),
        quick_value_face(&text, true),
    );
    let (dim_size, lit_size) = ((dim.w, dim.h), (lit.w, lit.h));
    let dim = upload(compositor, &mut state.dim, dim);
    let lit = upload(compositor, &mut state.lit, lit);
    app.set_settings_clock_faces((dim, dim_size.0, dim_size.1), (lit, lit_size.0, lit_size.1));
    state.shown = text;
}

pub(crate) fn sync_settings_network(
    app: &mut App,
    compositor: &mut Compositor,
    state: &mut WifiTexes,
) {
    if app.settings_row().is_some() {
        let summary = app.wifi_summary();
        if state.summary.as_ref() != Some(&summary) {
            let face = |lit: bool| match &summary {
                Summary::Off => quick_value_face("Off", lit),
                Summary::On => quick_value_face("On", lit),
                Summary::Joined(name) => {
                    plain_face(name, out_w() as f32 * 0.45, 27.0, omega_ui::MENU_H, lit)
                }
            };
            let [dim, lit] = [false, true].map(face);
            let (dim_size, lit_size) = ((dim.w, dim.h), (lit.w, lit.h));
            let dim = upload(compositor, &mut state.summary_tex[0], dim);
            let lit = upload(compositor, &mut state.summary_tex[1], lit);
            app.set_settings_network_faces(
                (dim, dim_size.0, dim_size.1),
                (lit, lit_size.0, lit_size.1),
            );
            state.summary = Some(summary);
        }
    }
}
