use std::sync::OnceLock;

use omega_gfx::{out_h, out_w, Draw, MeshId, TexId};
use omega_store::Cart;
use omega_store::Theme;

use crate::cart::{cart_box, label_colour, label_text, CART_W, SEATED_W};
use crate::footer::Printed;
use crate::icon::icon_box;
use crate::kit::palette::Palette;
use crate::plate::HINT_H;
use crate::shelf::{foot_y, shelf_box, shelf_scale};
use crate::spin::{self, Pose};
use crate::sprite::SPRITE_PX;
use crate::status_bar::{LINK_OFF, LINK_ON};

/// The alert shown on a refused cart as it comes back out, sized to the cart, which now
/// keeps its shelf size in the slot.
pub const ALERT_PX: f32 = 66.0;

/// How much larger the slot is drawn than the one a seated-size cart went into: a cart keeps
/// the size it has on the shelf all the way in, so the slot is built around that size.
const K: f32 = CART_W as f32 / SEATED_W as f32;

/// One art pixel on the screen. Every edge of the slot lands on a whole one.
const ART: f32 = SPRITE_PX as f32;

/// `px` moved onto the art grid: to the nearest whole art pixel, a half rounded down. The slot
/// is the seated slot K times larger, which puts some of its edges between art pixels (K is
/// 1.5); each moves to the grid, so it is drawn within a pixel of where K puts it.
const fn on_grid(px: f32) -> f32 {
    let art = px / ART;
    let whole = art as i32;
    let up = (art - whole as f32 > 0.5) as i32;
    (whole + up) as f32 * ART
}

/// The band that holds the slot, standing on the bar across the bottom.
pub const MOUTH_H: f32 = 80.0;

/// The UI kit's bar across the bottom of the shelf: `kit::action::BAR` art pixels tall.
pub const BAR_H: f32 = (crate::kit::action::BAR * SPRITE_PX) as f32;

/// How far above the bottom of the screen the slot stands: it sits on top of the bar.
pub const SLOT_RAISE: f32 = BAR_H;

/// The top of the slot's bar, where a cart going in lands on the lip.
pub fn slot_top() -> f32 {
    out_h() as f32 - SLOT_RAISE - MOUTH_H
}

fn band_y() -> f32 {
    slot_top()
}

// How far below the top of the band each of the slot's edges is, on the grid: the lip, the
// slit's top and bottom (where the scoop starts), the bottom of the bay and of the scoop, and
// the scoop's rim under it. A cart sits in the slot from `SEATED_TOP` down.
pub const LIP_H: f32 = on_grid(2.0 * K);
const SLIT_TOP: f32 = on_grid(7.0 * K);
const SLIT_BOTTOM: f32 = on_grid(16.0 * K);
const RECESS_BOTTOM: f32 = on_grid(44.0 * K);
const RIM_BOTTOM: f32 = on_grid(46.0 * K);
const SEATED_TOP: f32 = on_grid(6.0 * K);

// Half of each width, on the grid, so the slot stands on whole art pixels either side of the
// middle: the slit, the bay round it and the scoop under it.
const MOUTH_HALF: f32 = on_grid((CART_W as f32 + 14.0 * K) / 2.0);
const BAY_HALF: f32 = on_grid((CART_W as f32 + 32.0 * K) / 2.0);
const SCOOP_HALF: f32 = on_grid((CART_W as f32 + 14.0 * K) * 0.88 / 2.0);

pub const MOUTH_W: f32 = 2.0 * MOUTH_HALF;
const SLIT_H: f32 = SLIT_BOTTOM - SLIT_TOP;
fn mouth_x() -> f32 {
    cx() - MOUTH_HALF
}
fn slit_y() -> f32 {
    band_y() + SLIT_TOP
}

const BAY_W: f32 = 2.0 * BAY_HALF;
fn bay_x() -> f32 {
    cx() - BAY_HALF
}
fn bay_y() -> f32 {
    band_y() + LIP_H
}
const SCOOP_W: f32 = 2.0 * SCOOP_HALF;
fn scoop_d() -> f32 {
    RECESS_BOTTOM - SLIT_BOTTOM
}
fn scoop_y() -> f32 {
    band_y() + SLIT_BOTTOM
}

const RECESS_H: f32 = RECESS_BOTTOM - LIP_H;
const RIM_W: f32 = RIM_BOTTOM - RECESS_BOTTOM;
/// The middle of the screen, on the grid.
fn cx() -> f32 {
    on_grid(out_w() as f32 / 2.0)
}
const SCOOP_FLAT: f32 = 4.0;

/// The card's Config/theme.txt, for the widgets that still draw in its greys rather than the
/// kit's palette. The slot and the top bar take theirs from [`ChromeColours`] instead, handed
/// to them where they are drawn.
static THEME: OnceLock<Theme> = OnceLock::new();

pub fn set_theme(theme: Theme) {
    let _ = THEME.set(theme);
}

pub fn theme() -> &'static Theme {
    THEME.get_or_init(Theme::default)
}

fn rgb(c: [u8; 3]) -> [f32; 4] {
    [
        c[0] as f32 / 255.0,
        c[1] as f32 / 255.0,
        c[2] as f32 / 255.0,
        1.0,
    ]
}

pub fn housing() -> [f32; 4] {
    rgb(theme().housing)
}

pub fn opening() -> [f32; 4] {
    rgb(theme().opening)
}

pub fn edge() -> [f32; 4] {
    rgb(theme().edge)
}

pub fn recess() -> [f32; 4] {
    rgb(theme().recess)
}

/// The colours the shelf's chrome is drawn in: the slot, and the bar across the top.
#[derive(Copy, Clone, Debug, PartialEq)]
pub struct ChromeColours {
    /// The slot's body, and the bar across the top.
    pub housing: [f32; 4],
    /// The bay the slot is set in, a step down from the housing.
    pub recess: [f32; 4],
    /// The slit and the scoop under it, the darkest of them: the hole a cart goes into.
    pub opening: [f32; 4],
    /// The lip over the slit, the scoop's rim and the top bar's rule.
    pub edge: [f32; 4],
    /// What the top bar prints in: the clock, the battery and its figure, and what the
    /// handheld is connected to. Its faces are baked in it, so it is kept as bytes.
    pub ink: [u8; 3],
    /// What the top bar prints something that is off in: Wi-Fi, Bluetooth or SSH not
    /// connected.
    pub ink_off: [u8; 3],
}

/// How much of the kit's darkest colour the slot's hole keeps: a quarter, so on Night it is as
/// near black as the hole always was, and on any other colour the same hue a step deeper than
/// the bay around it.
const HOLE: u32 = 4;

impl ChromeColours {
    /// The chrome in the kit's colours, as flow.html draws the shelf: the slot's band and the
    /// top bar in the well colour, ruled in the edge colour, the bay in the dark one and the
    /// hole in it darker still. The top bar prints in cream, as flow.html's does, and what is
    /// off in the plate colour, as a network row draws the Wi-Fi bars a signal does not reach.
    pub fn of(pal: &Palette) -> Self {
        ChromeColours {
            housing: rgb(pal.well),
            recess: rgb(pal.dark),
            opening: rgb(pal.dark.map(|c| ((c as u32 + HOLE / 2) / HOLE) as u8)),
            edge: rgb(pal.edge),
            ink: pal.cream,
            ink_off: pal.plate,
        }
    }

    /// The chrome a card's Config/theme.txt sets, whatever the palette. A theme names no ink,
    /// so the top bar keeps the white and grey it was printed in before it took the palette.
    pub fn themed(theme: &Theme) -> Self {
        ChromeColours {
            housing: rgb(theme.housing),
            recess: rgb(theme.recess),
            opening: rgb(theme.opening),
            edge: rgb(theme.edge),
            ink: LINK_ON,
            ink_off: LINK_OFF,
        }
    }

    /// What the top bar prints an item in, `on` or off.
    pub fn link_ink(&self, on: bool) -> [u8; 3] {
        if on {
            self.ink
        } else {
            self.ink_off
        }
    }
}

/// The greys the chrome was drawn in before it took the kit's palette, which a theme.txt
/// still starts from.
impl Default for ChromeColours {
    fn default() -> Self {
        ChromeColours::themed(&Theme::default())
    }
}

fn lip_y() -> f32 {
    band_y()
}

fn seated_y() -> f32 {
    band_y() + SEATED_TOP
}

fn catch_at(foot: f32, seated_foot: f32) -> f32 {
    (lip_y() - foot) / (seated_foot - foot)
}

const CATCH_IN: f32 = 0.42;
/// How far into its way in (`SlotChrome::seat`) a cart has crept along the lip and starts down
/// into the slot; coming out, where it has just left the slot.
pub const CATCH_OUT: f32 = 0.62;
/// How far a cart creeps on the lip before it goes in, as a share of the way it came down to
/// it: a tall pak that stands just over the slot still visibly stops there.
const CREEP: f32 = 0.1;

pub struct SlotChrome<'a> {
    pub cart: &'a Cart,
    /// What the slot is drawn in.
    pub colours: ChromeColours,
    pub face: Option<TexId>,
    /// The model the cart is drawn with, sides and all; without one its face is spun flat.
    pub mesh: Option<MeshId>,
    pub rest: f32,
    /// How big the cart was drawn on the shelf, as a share of its own size, with `rest` its
    /// left edge there: both as `Shelf::selected_at` gives them.
    pub scale: f32,
    /// How the cart was held on the shelf: it comes level and square by the time it reaches
    /// the lip, and goes back to this as it comes out.
    pub pose: Pose,
    pub seat: f32,
    pub alert: Option<(TexId, f32)>,
    pub dim: f32,
    pub screen: f32,
    pub game: bool,
}

impl SlotChrome<'_> {
    pub fn draw(&self, out: &mut Vec<Draw>) {
        let seat = self.seat.clamp(0.0, 1.0);
        let dim = self.dim.clamp(0.0, 1.0);
        if dim > 0.0 {
            out.push(Draw::Rect {
                x: 0.0,
                y: 0.0,
                w: out_w() as f32,
                h: out_h() as f32,
                colour: [0.0, 0.0, 0.0, dim],
            });
        }

        let chrome = 1.0 - self.screen.clamp(0.0, 1.0);

        draw_slot_back(&self.colours, chrome, out);

        let (cw, ch) = cart_box(self.cart.platform);
        let (cw, ch) = (cw as f32, ch as f32);

        // A cart comes up to its own size as it goes in: what the row had not finished growing
        // by the lip, and what the shelf holds a pak smaller than it is as it sinks from the
        // lip to its seat, since at its own size on the lip a pak would stand over the bar
        // at the top.
        let scale = self.scale.clamp(0.0, 1.0);
        let shelf = shelf_scale(self.cart.platform);
        let row = (scale / shelf).min(1.0);
        let w0 = cw * scale;
        let (foot0, seated_foot) = (foot_y(shelf_box(self.cart.platform).1), seated_y() + ch);
        let catch = catch_at(foot0, seated_foot);
        let travel = travel(seat, catch);
        let k = (travel / catch).min(1.0);
        let sunk = ((travel - catch) / (1.0 - catch)).clamp(0.0, 1.0);
        let grown = (row + (1.0 - row) * k) * (shelf + (1.0 - shelf) * sunk);
        let (cw, ch) = (cw * grown, ch * grown);
        let centre = self.rest + w0 / 2.0;
        let x = centre + (cx() - centre) * k - cw / 2.0;
        let pose = self.pose.toward_rest(k);
        let y = foot0 + (seated_foot - foot0) * travel - ch - pose.lift;
        let cart_alpha = if seat >= 1.0 { chrome } else { 1.0 };
        match (self.face, self.mesh) {
            (Some(tex), Some(mesh)) => out.push(spin::posed_cart(
                (x, y, cw, ch),
                mesh,
                tex,
                [1.0, 1.0, 1.0, cart_alpha],
                0.0,
                pose.yaw,
            )),
            (Some(tex), None) => {
                spin::push_cart_side(out, (x, y, cw, ch), tex, 0.0, pose.yaw, 1.0, cart_alpha);
                out.push(spin::posed_face(
                    (x, y, cw, ch),
                    tex,
                    cart_alpha,
                    0.0,
                    pose.yaw,
                ));
            }
            (None, _) => {
                let c = label_colour(&label_text(self.cart));
                out.push(Draw::Rect {
                    x,
                    y,
                    w: cw,
                    h: ch,
                    colour: [
                        c[0] as f32 / 255.0,
                        c[1] as f32 / 255.0,
                        c[2] as f32 / 255.0,
                        cart_alpha,
                    ],
                });
            }
        }

        if let Some((tex, alpha)) = self.alert {
            let (w, h) = icon_box(ALERT_PX);
            let (w, h) = (w as f32, h as f32);
            out.push(Draw::Tex {
                x: x + (cw - w) / 2.0,
                y: y + (ch - h) / 2.0,
                w,
                h,
                tex,
                alpha,
            });
        }

        if chrome < 1.0 {
            front_bands(&self.colours, |x, y, w, h, _| {
                out.push(Draw::Rect {
                    x,
                    y,
                    w,
                    h,
                    colour: [0.0, 0.0, 0.0, 1.0],
                });
            });
        }

        if self.game && self.screen > 0.0 {
            out.push(Draw::Game);
        }

        draw_slot_front(&self.colours, chrome, out);
    }
}

fn band(x: f32, y: f32, w: f32, h: f32, c: [f32; 4], alpha: f32) -> Draw {
    Draw::Rect {
        x,
        y,
        w,
        h,
        colour: [c[0], c[1], c[2], c[3] * alpha],
    }
}

fn draw_slot_back(c: &ChromeColours, alpha: f32, out: &mut Vec<Draw>) {
    out.push(band(bay_x(), band_y(), BAY_W, LIP_H, c.housing, alpha));
    out.push(band(mouth_x(), band_y(), MOUTH_W, LIP_H, c.edge, alpha));
    out.push(band(bay_x(), bay_y(), BAY_W, RECESS_H, c.recess, alpha));
    out.push(band(mouth_x(), slit_y(), MOUTH_W, SLIT_H, c.opening, alpha));
    for_each_scoop_span(|x, w, depth| {
        out.push(band(x, scoop_y(), w, depth, c.opening, alpha));
    });
}

fn draw_slot_front(c: &ChromeColours, alpha: f32, out: &mut Vec<Draw>) {
    front_bands(c, |x, y, w, h, c| out.push(band(x, y, w, h, c, alpha)));
}

fn front_bands(c: &ChromeColours, mut band: impl FnMut(f32, f32, f32, f32, [f32; 4])) {
    let w = out_w() as f32;
    let right = bay_x() + BAY_W;
    let floor = scoop_y() + scoop_d() + RIM_W;
    let housing = c.housing;
    band(0.0, band_y(), bay_x(), MOUTH_H, housing);
    band(right, band_y(), w - right, MOUTH_H, housing);

    let near = cx() - SCOOP_W / 2.0;
    band(
        bay_x(),
        scoop_y(),
        near - bay_x(),
        floor - scoop_y(),
        housing,
    );
    let far = cx() + SCOOP_W / 2.0;
    band(far, scoop_y(), right - far, floor - scoop_y(), housing);

    for_each_scoop_span(|x, w, depth| {
        let top = scoop_y() + depth + RIM_W;
        band(x, top, w, floor - top, housing);
    });
    band(0.0, floor, w, out_h() as f32 - floor, housing);
    for_each_scoop_span(|x, w, depth| {
        band(x, scoop_y() + depth, w, RIM_W, c.edge);
    });
}

/// The scoop's curve as spans of columns one art pixel wide, each as deep as the curve is in
/// its middle, to the nearest art pixel, so the curve steps an art pixel at a time.
fn for_each_scoop_span(mut span: impl FnMut(f32, f32, f32)) {
    let hw = SCOOP_W / 2.0;
    let depth = |x: f32| {
        let curve = scoop_d() * (1.0 - ((x + ART / 2.0).abs() / hw).powf(SCOOP_FLAT)).max(0.0);
        on_grid(curve)
    };
    let mut x = -hw;
    while x < hw {
        let d = depth(x);
        let start = x;
        while x < hw && depth(x) == d {
            x += ART;
        }
        span(cx() + start, x - start, d);
    }
}

pub fn draw_slot_name(name: Printed, alpha: f32, out: &mut Vec<Draw>) {
    let (Some(tex), true) = (name.face, alpha > 0.0) else {
        return;
    };
    let (w, h) = (name.w as f32, HINT_H as f32);
    let hole = scoop_y() + scoop_d() - slit_y();
    out.push(Draw::Tex {
        x: cx() - w / 2.0,
        y: slit_y() + (hole - h) / 2.0,
        w,
        h,
        tex,
        alpha,
    });
}

/// The slot with no cart in it, in `colours`.
pub fn draw_empty_slot(colours: &ChromeColours, out: &mut Vec<Draw>) {
    draw_slot_back(colours, 1.0, out);
    draw_slot_front(colours, 1.0, out);
}

fn travel(seat: f32, catch: f32) -> f32 {
    let creep = CREEP * catch;
    if seat < CATCH_IN {
        catch * ease(seat / CATCH_IN)
    } else if seat < CATCH_OUT {
        catch + creep * (seat - CATCH_IN) / (CATCH_OUT - CATCH_IN)
    } else {
        let caught = catch + creep;
        caught + (1.0 - caught) * ease((seat - CATCH_OUT) / (1.0 - CATCH_OUT))
    }
}

pub fn ease(u: f32) -> f32 {
    u * u * u * (u * (u * 6.0 - 15.0) + 10.0)
}
