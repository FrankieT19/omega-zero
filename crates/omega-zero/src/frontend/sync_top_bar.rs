//! The top bar's faces: what the handheld is connected to, on and off, the clock, the battery's
//! figure and its bolt, all baked in the top bar's ink (`ChromeColours::ink`). The ink is the
//! palette's, so a new colour in Settings > Theme bakes them all again, as it does the
//! kit's faces; a card's theme.txt keeps them white. The clock and the figure are also printed
//! again when what they say changes.

use omega_gfx::{Compositor, TexId};
use omega_ui::{bolt_face, hhmm, link_face, word_face_in, Link, LinkFaces};

use crate::app::{App, Readouts};
use crate::frontend::faces::{upload, Printing};

/// The top bar's textures, each reused as what it shows changes, and what they were baked from.
#[derive(Default)]
pub(crate) struct TopBarTexes {
    /// The ink, on and off, the icons and the bolt were baked in.
    ink: Option<([u8; 3], [u8; 3])>,
    /// Wi-Fi's, Bluetooth's and SSH's faces, each off then on.
    links: [[Option<TexId>; 2]; 3],
    bolt: Option<TexId>,
    clock: Printing<(String, [u8; 3])>,
    percent: Printing<(String, [u8; 3])>,
    /// What the app was last handed.
    readouts: Readouts,
}

pub(crate) fn sync_top_bar(app: &mut App, compositor: &mut Compositor, texes: &mut TopBarTexes) {
    let colours = app.chrome_colours();
    let ink = (colours.ink, colours.ink_off);
    let mut readouts = texes.readouts;
    if texes.ink != Some(ink) {
        let mut pair = |held: &mut [Option<TexId>; 2], link: Link| {
            [false, true].map(|on| {
                let face = link_face(link, on, &colours);
                let (w, h) = (face.w, face.h);
                (upload(compositor, &mut held[usize::from(on)], face), w, h)
            })
        };
        let [wifi, bluetooth, ssh] = &mut texes.links;
        app.set_link_faces(LinkFaces {
            wifi: pair(wifi, Link::Wifi),
            bluetooth: pair(bluetooth, Link::Bluetooth),
            ssh: pair(ssh, Link::Ssh),
        });
        readouts.bolt = Some(upload(compositor, &mut texes.bolt, bolt_face(colours.ink)));
        texes.ink = Some(ink);
    }
    let word = |(text, ink): &(String, [u8; 3])| word_face_in(text, *ink);
    let time = (hhmm(app.wall_secs()), colours.ink);
    if let Some(clock) = texes.clock.print(compositor, time, word) {
        readouts.clock = clock;
    }
    if let Some(battery) = app.battery() {
        let figure = (format!("{}%", battery.percent), colours.ink);
        if let Some(percent) = texes.percent.print(compositor, figure, word) {
            readouts.percent = percent;
        }
    }
    if readouts != texes.readouts {
        app.set_top_readouts(readouts);
        texes.readouts = readouts;
    }
}
