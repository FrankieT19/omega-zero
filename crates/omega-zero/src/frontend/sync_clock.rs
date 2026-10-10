//! The words printed on the shelf's slot and the switcher's bar, the time, its slot's name and
//! the battery's charge, and the Date & Time panel. The switcher keeps its white over the game;
//! the top bar prints its own in its ink (sync_top_bar.rs).

use omega_gfx::{Compositor, TexId};
use omega_ui::{bolt_face, hhmm, word_face, Field, HUD_INK};

use crate::app::App;
use crate::frontend::faces::{upload, Printing};
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
    let mut switcher = clocks.switcher;
    let word = |text: &String| word_face(text);
    if let Some(clock) = clocks.clock.print(compositor, hhmm(app.wall_secs()), word) {
        switcher.clock = clock;
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
    if let Some(battery) = app.battery() {
        let figure = format!("{}%", battery.percent);
        if let Some(percent) = clocks.percent.print(compositor, figure, word) {
            switcher.percent = percent;
        }
    }
    if switcher.bolt.is_none() {
        upload(compositor, &mut switcher.bolt, bolt_face(HUD_INK));
    }
    if switcher != clocks.switcher {
        app.set_switcher_readouts(switcher);
        clocks.switcher = switcher;
    }
}
