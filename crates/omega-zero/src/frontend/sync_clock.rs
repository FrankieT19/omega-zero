//! The words printed on the shelf's slot and the switcher's bar, the time, its slot's name and
//! the battery's charge, the alert a refused cart comes out with, and the Date & Time panel.
//! The switcher keeps its white over the game; the top bar prints its own in its ink
//! (sync_top_bar.rs).

use omega_gfx::{Compositor, TexId};
use omega_ui::{alert_face, bolt_face, hhmm, slot_name_face, word_face, Field, HUD_INK};

use crate::app::App;
use crate::frontend::faces::{upload_nearest, Printing};
use crate::frontend::lone::Baked;
use crate::screens::clock::clock_face;

#[derive(Default)]
pub(crate) struct Clocks {
    /// The Date & Time panel, baked again when the picker's line, its cursor or the palette
    /// changes.
    panel: Baked<(String, Field, u8)>,
    /// The switcher's clock and figure, and what each says.
    clock: Printing<String>,
    percent: Printing<String>,
    /// What the switcher was last handed, its bolt among them.
    switcher: crate::app::Readouts,
    /// The slot's name and the palette colour it was baked in, and its texture.
    platform: (String, u8),
    platform_tex: Option<TexId>,
    /// The refused cart's alert, baked again when the palette changes.
    alert: Baked<u8>,
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
    let colour = app.palette_colour();
    if let Some(alert) = clocks
        .alert
        .bake(compositor, colour, |_| alert_face(app.palette()))
    {
        app.set_alert_face(alert.tex);
    }
    let mut switcher = clocks.switcher;
    let word = |text: &String| word_face(text);
    if let Some(clock) = clocks.clock.print(compositor, hhmm(app.wall_secs()), word) {
        switcher.clock = clock;
    }
    let named = (app.slot_text().unwrap_or_default(), colour);
    if named != clocks.platform {
        if named.0.is_empty() {
            app.clear_shelf_platform();
        } else {
            let face = slot_name_face(&named.0, app.palette());
            let id = upload_nearest(compositor, &mut clocks.platform_tex, &face);
            app.set_shelf_platform_face(id, face.w);
        }
        clocks.platform = named;
    }
    if let Some(battery) = app.battery() {
        let figure = format!("{}%", battery.percent);
        if let Some(percent) = clocks.percent.print(compositor, figure, word) {
            switcher.percent = percent;
        }
    }
    if switcher.bolt.is_none() {
        upload_nearest(compositor, &mut switcher.bolt, &bolt_face(HUD_INK));
    }
    if switcher != clocks.switcher {
        app.set_switcher_readouts(switcher);
        clocks.switcher = switcher;
    }
}
