//! The About screen's label, made again when the battery's charge on it changes: pixel art
//! laid out for the canvas, uploaded with nearest filtering and drawn at 2x.

use omega_gfx::{Compositor, TexId};
use omega_ui::{sticker_face, StickerFields};

use crate::app::{App, Phase};
use crate::build_info::Build;
use crate::frontend::faces::upload_nearest;

#[derive(Default)]
pub(crate) struct AboutFace {
    tex: Option<TexId>,
    battery: Option<u8>,
}

pub(crate) fn sync_about(app: &mut App, compositor: &mut Compositor, state: &mut AboutFace) {
    if !matches!(app.phase(), Phase::About) {
        return;
    }
    let battery = app.battery().map(|b| b.percent);
    if state.tex.is_some() && state.battery == battery {
        return;
    }
    state.battery = battery;
    let build = Build::current();
    let face = sticker_face(&StickerFields {
        battery,
        serial: &build.serial(),
        dirty_digit: build.dirty_digit(),
    });
    let id = upload_nearest(compositor, &mut state.tex, &face);
    app.set_sticker_face(id);
}
