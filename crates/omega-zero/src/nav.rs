//! Moving between screens. Every move slides: going deeper, the screen left behind leaves to
//! the right as the one it opens comes in from the left; going back, the screen leaves to the
//! left as the one it came from comes back from the right. The Controls panel stands beside the
//! shelf rather than under it, and slides the mirror way: the shelf leaves to the left as the
//! panel comes in from the right, and going back the panel leaves to the right as the shelf
//! comes back from the left. The backdrop under them never moves.
//!
//! The shelf slides in three layers rather than as one: its carts leave across, the way the
//! slide goes, its top bar up off the screen and its slot down, and they come back the same
//! ways.

use omega_input::Millis;
use omega_ui::{ease, CART_W, GB_CART_W, SPRITE_PX, TOP_BAR_H};

use crate::app::Phase;

/// How long a slide takes.
pub const SLIDE_MS: Millis = 280;

/// How far the slot's band and its name drop to be off the screen.
pub const SLOT_DROP: f32 = 122.0;

/// Which way a move goes.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Dir {
    /// Into a screen opened from the one shown.
    Deeper,
    /// Back out to the screen the one shown was opened from.
    Back,
    /// Out to the Controls panel, beside the shelf: the mirror of going deeper.
    Aside,
    /// Back from the Controls panel to the shelf beside it: the mirror of going back.
    FromAside,
}

/// Which side of a slide a screen is on.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum End {
    Leaving,
    Arriving,
}

/// A slide under way: the screen it leaves, which way it goes and when it started. The screen
/// it arrives at is the app's phase.
#[derive(Clone, Debug)]
pub struct Slide {
    pub from: Phase,
    pub dir: Dir,
    pub started: Millis,
}

impl Slide {
    /// How far it has gone at `now`.
    pub fn progress(&self, now: Millis) -> f32 {
        progress(self.started, now)
    }

    /// It has gone all the way by `now`.
    pub fn over(&self, now: Millis) -> bool {
        now.saturating_sub(self.started) >= SLIDE_MS
    }
}

/// How far a slide that started at `started` has gone at `now`, eased: 0 as it starts and 1
/// once it is over.
pub fn progress(started: Millis, now: Millis) -> f32 {
    ease((now.saturating_sub(started) as f32 / SLIDE_MS as f32).min(1.0))
}

/// `px` rounded to whole art pixels, SPRITE_PX screen pixels each, so a sliding screen keeps
/// its sprites on their grid.
pub fn even(px: f32) -> f32 {
    let art = SPRITE_PX as f32;
    (px / art).round() * art
}

/// The widest a cart is drawn: the neighbour peeking in at the left edge has to travel the
/// screen's width and its own to clear it.
pub fn widest_cart() -> f32 {
    CART_W.max(GB_CART_W) as f32
}

/// How far each layer of a screen is moved while it slides. A menu screen moves as one, `dx`
/// across; the shelf moves its carts `carts_dx` across, its top bar `bar_dy` down (up, being
/// negative) and its slot `slot_dy` down.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Offsets {
    pub dx: f32,
    pub carts_dx: f32,
    pub bar_dy: f32,
    pub slot_dy: f32,
}

impl Offsets {
    /// A screen at rest.
    pub const NONE: Offsets = Offsets {
        dx: 0.0,
        carts_dx: 0.0,
        bar_dy: 0.0,
        slot_dy: 0.0,
    };
}

/// How far across a screen is that has to travel `span` to clear a canvas, `p` of the way
/// through a slide going `dir`, as it leaves or arrives.
pub fn across(dir: Dir, end: End, p: f32, span: f32) -> f32 {
    // the way the screen left behind goes: right going deeper or back from Controls, left
    // going back or out to Controls
    let rightward = match dir {
        Dir::Deeper | Dir::FromAside => 1.0,
        Dir::Back | Dir::Aside => -1.0,
    };
    match end {
        End::Leaving => rightward * even(p * span),
        End::Arriving => -rightward * even((1.0 - p) * span),
    }
}

/// Where each layer of a screen on a canvas `w` wide is, `p` of the way through a slide going
/// `dir`, as it leaves or arrives. The shelf's layers are as far from home as the slide is
/// from the shelf: all the way gone once it has left, home once it has arrived.
pub fn offsets(dir: Dir, end: End, p: f32, w: f32) -> Offsets {
    let gone = match end {
        End::Leaving => p,
        End::Arriving => 1.0 - p,
    };
    Offsets {
        dx: across(dir, end, p, w),
        carts_dx: across(dir, end, p, w + widest_cart()),
        bar_dy: -even(gone * TOP_BAR_H),
        slot_dy: even(gone * SLOT_DROP),
    }
}
