//! The words printed on the shelf, the time, its slot's name and the battery's charge, and
//! the Date & Time panel.

use omega_gfx::{Compositor, TexId};
use omega_ui::{hhmm, word_face, Field};

use crate::app::App;
use crate::frontend::faces::upload;
use crate::frontend::lone::Baked;
use crate::screens::clock::clock_face;

#[derive(Default)]
pub(crate) struct Clocks {
    /// The Date & Time panel, baked again when the picker's line, its cursor or the palette
    /// changes.
    panel: Baked<(String, Field, u8)>,
    shelf: Option<TexId>,
    shown: String,
    battery: String,
    battery_tex: Option<TexId>,
    platform: String,
    platform_tex: Option<TexId>,
}

pub(crate) fn sync_clock(app: &mut App, compositor: &mut Compositor, clocks: &mut Clocks) {
    if let Some(picker) = app.picker() {
        let pal = app.palette();
        let from = (picker.line(), picker.cursor(), app.palette_colour());
        let baked = clocks
            .panel
            .bake(compositor, from, |_| clock_face(picker, pal));
        if let Some(panel) = baked {
            app.set_clock_panel(panel);
        }
    }
    let shown = hhmm(app.wall_secs());
    if shown != clocks.shown {
        let face = word_face(&shown);
        clocks.shown = shown;
        let w = face.w;
        let id = upload(compositor, &mut clocks.shelf, face);
        app.set_shelf_clock_face(id, w);
    }
    let platform_shown = app.slot_text().unwrap_or_default();
    if platform_shown != clocks.platform {
        clocks.platform = platform_shown.clone();
        if platform_shown.is_empty() {
            app.clear_shelf_platform();
        } else {
            let face = word_face(&platform_shown);
            let w = face.w;
            let id = upload(compositor, &mut clocks.platform_tex, face);
            app.set_shelf_platform_face(id, w);
        }
    }
    let battery_shown = app
        .battery()
        .map(|b| format!("{}%", b.percent))
        .unwrap_or_default();
    if battery_shown != clocks.battery {
        clocks.battery = battery_shown.clone();
        if !battery_shown.is_empty() {
            let face = word_face(&battery_shown);
            let w = face.w;
            let id = upload(compositor, &mut clocks.battery_tex, face);
            app.set_battery_percent_face(id, w);
        }
    }
}
