//! What the HUD says just happened, each under the title of what it is about, drawn as the
//! kit's toast (kit::toast).

use crate::kit::palette::Palette;
use crate::kit::toast::toast_face;
use crate::plate::UndoFace;

#[derive(Copy, Clone, PartialEq, Eq, Debug)]
pub enum Toast {
    Shader(u8),
    Palette(u8),
    FavouriteAdded,
    FavouriteRemoved,
    StateSaved,
    StateLoaded,
    NeedsGpsp,
    NoLink,
    LinkEnded,
    PeerEnded,
    BiosMismatch,
    ColourOn,
    ColourOff,
    /// SELECT + MENU on a PlayStation game on one disc: there is no other to swap in.
    OneDisc,
    /// The DISC screen swapped disc `n`, counting from 1, into the drive.
    Disc(u8),
}

/// What the toast says once disc `n` is in, for every disc a PlayStation game can have.
const DISC_IN: [&str; 9] = [
    "Disc 1 inserted",
    "Disc 2 inserted",
    "Disc 3 inserted",
    "Disc 4 inserted",
    "Disc 5 inserted",
    "Disc 6 inserted",
    "Disc 7 inserted",
    "Disc 8 inserted",
    "Disc 9 inserted",
];

impl Toast {
    /// Every toast there is, a disc's once, as disc 1.
    pub const ALL: [Toast; 15] = [
        Toast::StateSaved,
        Toast::StateLoaded,
        Toast::NeedsGpsp,
        Toast::NoLink,
        Toast::LinkEnded,
        Toast::PeerEnded,
        Toast::BiosMismatch,
        Toast::ColourOn,
        Toast::ColourOff,
        Toast::OneDisc,
        Toast::Disc(1),
        Toast::Shader(0),
        Toast::FavouriteAdded,
        Toast::FavouriteRemoved,
        Toast::Palette(0),
    ];

    /// Its place in `ALL`: every disc's toast is the one place.
    pub fn index(self) -> usize {
        match self {
            Toast::Palette(_) => 14,
            Toast::Shader(_) => 11,
            Toast::FavouriteAdded => 12,
            Toast::FavouriteRemoved => 13,
            Toast::StateSaved => 0,
            Toast::StateLoaded => 1,
            Toast::NeedsGpsp => 2,
            Toast::NoLink => 3,
            Toast::LinkEnded => 4,
            Toast::PeerEnded => 5,
            Toast::BiosMismatch => 6,
            Toast::ColourOn => 7,
            Toast::ColourOff => 8,
            Toast::OneDisc => 9,
            Toast::Disc(_) => 10,
        }
    }

    /// What it is about, in the toast's title tab.
    pub fn label(self) -> &'static str {
        match self {
            Toast::Palette(_) => "PALETTE",
            Toast::Shader(_) => "SHADER",
            Toast::FavouriteAdded | Toast::FavouriteRemoved => "FAVOURITES",
            Toast::StateSaved | Toast::StateLoaded => "SAVE STATE",
            Toast::NeedsGpsp
            | Toast::NoLink
            | Toast::LinkEnded
            | Toast::PeerEnded
            | Toast::BiosMismatch => "LINK",
            Toast::ColourOn | Toast::ColourOff => "COLOUR",
            Toast::OneDisc | Toast::Disc(_) => "DISC",
        }
    }

    /// The line in the toast's well, in its own case.
    pub fn text(self) -> &'static str {
        match self {
            Toast::Palette(n) => ["Game colours", "Pocket", "DMG green", "Blue", "Grayscale"]
                .get(n as usize)
                .copied()
                .unwrap_or("Game colours"),
            Toast::Shader(n) => omega_store::SHADER_NAMES
                .get(n as usize)
                .copied()
                .unwrap_or("Clean"),
            Toast::FavouriteAdded => "Added to favourites",
            Toast::FavouriteRemoved => "Removed from favourites",
            Toast::StateSaved => "State saved",
            Toast::StateLoaded => "State loaded",
            Toast::NeedsGpsp => "Please switch to gpSP",
            Toast::NoLink => "No link support",
            Toast::LinkEnded => "Link ended",
            Toast::PeerEnded => "Link was ended",
            Toast::BiosMismatch => "BIOS does not match",
            Toast::ColourOn => "Correction on",
            Toast::ColourOff => "Correction off",
            Toast::OneDisc => "Only one disc",
            Toast::Disc(n) => DISC_IN
                .get(usize::from(n).wrapping_sub(1))
                .copied()
                .unwrap_or("Disc inserted"),
        }
    }

    /// The toast as the kit draws it, in `pal`.
    pub fn face(self, pal: &Palette) -> UndoFace {
        toast_face(self.label(), self.text(), pal)
    }
}
