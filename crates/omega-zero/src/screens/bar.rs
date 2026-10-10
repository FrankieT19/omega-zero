//! The bar along the bottom of the shelf and of every menu, as flow.html draws it: an action at
//! each end and the plate between them, with the keys a screen has beyond its two actions shown
//! on the plate as hints. The shelf's actions are SELECT CONTROLS and START SETTINGS, a menu's
//! B RETURN and A CONFIRM. Setting the clock on first boot has nowhere to go back to, so its
//! left action is the kit's empty one: the bar keeps its shape and offers only CONFIRM.
//!
//! The plate never moves with the screens. When a slide's two ends have different actions, the
//! old ones sink out of the screen over its first half and the new ones rise into place over
//! the second, as flow/player.js does; between two menus they stay put, and the hints change at
//! halfway. The core picker swaps them the same way as it opens and closes, its ◀▶ SWAP coming
//! onto the plate with RETURN and CONFIRM and leaving with them.
//!
//! A cart going in takes the whole bar down out of the screen with it, by the time it starts
//! down into the slot; the bar stays away while the game is on and rises back as the cart comes
//! out.

use omega_gfx::{out_h, Draw, TexId};
use omega_ui::kit::action::Side;
use omega_ui::kit::bar::{action_swap, BarLayout, Hint, SINK};
use omega_ui::kit::pixel::quad;
use omega_ui::{ease, Millis, CATCH_OUT, SPRITE_PX};

use crate::app::{App, Phase};
use crate::core_picker::CorePicker;
use crate::screens::labels::labels_hints;

/// The core picker's one key beyond the bar's actions: Left and Right move the chip.
pub const SWAP: Hint<'static> = Hint {
    key: "◀▶",
    word: "SWAP",
};

/// Which pair of actions the bar shows at its ends.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Actions {
    /// The shelf's: SELECT CONTROLS on the left, START SETTINGS on the right.
    Shelf,
    /// A menu's: B RETURN and A CONFIRM.
    Menu,
    /// A menu with no way back, the clock set on first boot: A CONFIRM alone, the left action
    /// empty.
    Confirm,
}

impl Actions {
    pub const ALL: [Actions; 3] = [Actions::Shelf, Actions::Menu, Actions::Confirm];

    pub fn index(self) -> usize {
        self as usize
    }

    /// The big word and the button in its pill of the action at `side`.
    pub fn action(self, side: Side) -> (&'static str, &'static str) {
        match (self, side) {
            (Actions::Shelf, Side::Left) => ("CONTROLS", "SELECT"),
            (Actions::Shelf, Side::Right) => ("SETTINGS", "START"),
            (Actions::Menu, Side::Left) => ("RETURN", "B"),
            (Actions::Menu | Actions::Confirm, Side::Right) => ("CONFIRM", "A"),
            (Actions::Confirm, Side::Left) => ("", ""),
        }
    }
}

/// Where the bar is in a frame: whose actions it shows, how far they are sunk below their place
/// in a swap, and how far the whole bar is sunk below the screen's bottom edge as a cart goes
/// in or comes out, both in art pixels.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct BarPlace {
    pub actions: Actions,
    pub sink: u32,
    pub drop: u32,
}

/// One pair of actions uploaded, each face with its size in art pixels, and where they and the
/// plate between them go.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct BarSet {
    pub layout: BarLayout,
    pub left: (TexId, u32, u32),
    pub right: (TexId, u32, u32),
}

/// The bar's faces: each pair of actions, and the plate laid out between each, in
/// `Actions::ALL`'s order. A plate is made with the hints of the screen it is shown on.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct BarFaces {
    pub sets: [BarSet; 3],
    pub plates: [Option<TexId>; 3],
}

/// The actions the bar shows on `phase` at rest, or None where there is no bar: in a game, its
/// menus and its photos, and asleep. A cart going in or coming out still has the shelf's.
pub fn actions_on(phase: &Phase) -> Option<Actions> {
    match phase {
        Phase::Shelf | Phase::Inserting { .. } | Phase::Ejecting { .. } => Some(Actions::Shelf),
        Phase::SetClock {
            from_menu: false, ..
        } => Some(Actions::Confirm),
        Phase::Settings { .. }
        | Phase::Controls
        | Phase::Network(_)
        | Phase::Ssh(_)
        | Phase::Labels(_)
        | Phase::Background(_)
        | Phase::SetClock { .. }
        | Phase::About => Some(Actions::Menu),
        Phase::Playing { .. } | Phase::Polaroids { .. } | Phase::Doze { .. } => None,
    }
}

/// How far the bar has sunk `seat` of the way through a cart's way in (0 on the shelf, 1
/// seated): gone, eased, by the time the cart has crept along the lip and starts down into
/// the slot, and back as it comes up out of it.
fn sunk_for(seat: f32) -> u32 {
    (ease((seat / CATCH_OUT).clamp(0.0, 1.0)) * SINK as f32).round() as u32
}

/// How far through the swap to its actions the core picker is, eased: the bar follows the
/// picker opening and closing.
fn picker_swap(picker: &CorePicker, now: Millis) -> f32 {
    ease(picker.openness(now))
}

/// The actions shown, and how far sunk, `p` of the way (eased) from `from`'s to `to`'s.
fn swapped(from: Option<Actions>, to: Option<Actions>, p: f32) -> Option<(Actions, u32)> {
    if from == to {
        return to.map(|a| (a, 0));
    }
    let swap = action_swap(p);
    match (swap.new, from, to) {
        (false, Some(a), _) | (true, _, Some(a)) => Some((a, swap.sink)),
        // one end has no bar: the plate stays and the other end's actions keep out of sight
        (_, Some(a), None) | (_, None, Some(a)) => Some((a, SINK)),
        (_, None, None) => None,
    }
}

impl App {
    /// Where the bar is now, or None when it is out of sight.
    pub fn bar_place(&self) -> Option<BarPlace> {
        let now = self.now();
        let (actions, sink) = match (&self.slide, self.core_picker_shown()) {
            (Some(slide), _) => swapped(
                actions_on(&slide.from),
                actions_on(&self.phase),
                slide.progress(now),
            )?,
            // the picker's B and A are the bar's: it swaps to them as the picker opens
            (None, Some(picker)) if matches!(self.phase, Phase::Shelf) => swapped(
                Some(Actions::Shelf),
                Some(Actions::Menu),
                picker_swap(&picker, now),
            )?,
            (None, _) => (actions_on(&self.phase)?, 0),
        };
        let drop = match self.phase {
            Phase::Inserting { .. } | Phase::Ejecting { .. } => sunk_for(self.seat()),
            _ => 0,
        };
        (drop < SINK).then_some(BarPlace {
            actions,
            sink: (sink + drop).min(SINK),
            drop,
        })
    }

    /// The keys `phase` has beyond its two actions, for its plate: Y FORGET on a saved network
    /// (`network_forgets`' rule), on Labels START FETCH or START STOP and Y SIGN OUT
    /// (`labels_bar_hints`'), and on the shelf, while the core picker over it shows RETURN and
    /// CONFIRM, ◀▶ SWAP. They are asked of `phase` rather than the screen on show so a slide
    /// can show the hints of the screen it leaves. A screen with more keys adds them here;
    /// Settings and SSH have none, and the keyboard keeps its own line in its modal.
    pub fn bar_hints(&self, phase: &Phase) -> Vec<Hint<'static>> {
        match phase {
            Phase::Shelf if self.picker_has_bar() => vec![SWAP],
            Phase::Network(screen) if screen.typing.is_none() && screen.forgets(&self.wifi) => {
                vec![Hint {
                    key: "Y",
                    word: "FORGET",
                }]
            }
            Phase::Labels(screen) if screen.typing.is_none() => {
                labels_hints(&screen.view(&self.labels, self.online()).legend)
            }
            _ => Vec::new(),
        }
    }

    /// Whether the core picker is open far enough to have the bar: past halfway through the
    /// swap to its actions, where they and their plate come in.
    fn picker_has_bar(&self) -> bool {
        self.core_picker_shown()
            .is_some_and(|picker| action_swap(picker_swap(&picker, self.now())).new)
    }

    /// The hints the plate shows now: the screen's on show, and through a slide the leaving
    /// screen's until halfway, the arriving one's after.
    pub fn bar_hints_shown(&self) -> Vec<Hint<'static>> {
        match &self.slide {
            Some(slide) if slide.progress(self.now()) < 0.5 => self.bar_hints(&slide.from),
            _ => self.bar_hints(&self.phase),
        }
    }

    /// Each pair of actions, uploaded: what the bar's plates are laid out from.
    pub fn set_bar_sets(&mut self, sets: [BarSet; 3]) {
        self.bar_faces = Some(BarFaces {
            sets,
            plates: [None; 3],
        });
    }

    /// The bar's faces once its actions are uploaded.
    pub fn bar_faces(&self) -> Option<&BarFaces> {
        self.bar_faces.as_ref()
    }

    /// The plate laid out between `actions`, with the hints of the screen it is shown on.
    pub fn set_bar_plate(&mut self, actions: Actions, tex: TexId) {
        if let Some(faces) = &mut self.bar_faces {
            faces.plates[actions.index()] = Some(tex);
        }
    }

    /// The bar, over whatever screen is shown or sliding and still as they slide.
    pub(crate) fn draw_bar(&self, out: &mut Vec<Draw>) {
        let (Some(place), Some(faces)) = (self.bar_place(), &self.bar_faces) else {
            return;
        };
        let set = &faces.sets[place.actions.index()];
        let (h, px) = (out_h(), |art: i32| (art * SPRITE_PX as i32) as f32);
        if let Some(tex) = faces.plates[place.actions.index()] {
            out.push(
                set.layout
                    .plate_quad(tex, h)
                    .shifted(0.0, px(place.drop as i32)),
            );
        }
        if place.sink >= SINK {
            return;
        }
        for ((tex, w, th), [x, y]) in [(set.left, set.layout.left), (set.right, set.layout.right)] {
            let y = BarLayout::top(h) + px(y + place.sink as i32);
            out.push(quad(tex, w, th, px(x), y));
        }
    }
}
