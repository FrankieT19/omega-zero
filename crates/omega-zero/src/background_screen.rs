//! The Background screen, opened from the quick menu: what the shelf is drawn over, shown
//! behind the list as it is changed.

use omega_input::{Action, Btn};
use omega_store::{BackdropStyle, BACKDROP_COLOURS};
use omega_ui::backdrop_colour_name;

use crate::list_view::{LegendHint, ListView, RowLabel, RowValue, ViewRow};

/// The shelf's background as it is drawn.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Backdrop {
    pub style: BackdropStyle,
    pub moving: bool,
    pub colour: u8,
}

impl Backdrop {
    /// What the settings come to on a card that does or does not have a wallpaper: one that
    /// never picked shows the wallpaper if there is one, as omega-zero always did, else the grid.
    pub fn resolve(
        picked: Option<BackdropStyle>,
        moving: bool,
        colour: u8,
        wallpaper: bool,
    ) -> Self {
        let style = match picked {
            Some(BackdropStyle::Wallpaper) | None if wallpaper => BackdropStyle::Wallpaper,
            Some(BackdropStyle::Wallpaper) | None => BackdropStyle::Grid,
            Some(style) => style,
        };
        Backdrop {
            style,
            moving,
            colour: colour.min(BACKDROP_COLOURS.len() as u8 - 1),
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Sel {
    Style,
    Motion,
    Colour,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct BackgroundScreen {
    pub sel: Sel,
}

impl Default for BackgroundScreen {
    fn default() -> Self {
        BackgroundScreen { sel: Sel::Style }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Outcome {
    Stay,
    Close,
    Changed(Backdrop),
}

/// The styles a card offers: the wallpaper only where there is one.
fn styles(wallpaper: bool) -> Vec<BackdropStyle> {
    BackdropStyle::ALL
        .into_iter()
        .filter(|s| *s != BackdropStyle::Wallpaper || wallpaper)
        .collect()
}

fn step<T: PartialEq + Copy>(items: &[T], at: T, forward: bool) -> T {
    let n = items.len();
    let i = items.iter().position(|x| *x == at).unwrap_or(0);
    items[if forward {
        (i + 1) % n
    } else {
        (i + n - 1) % n
    }]
}

impl BackgroundScreen {
    /// The rows a style has a use for: a picture has no colour, a plain colour does not move.
    fn rows(look: Backdrop) -> Vec<Sel> {
        let mut rows = vec![Sel::Style];
        if look.style.moves() {
            rows.push(Sel::Motion);
        }
        if look.style.coloured() {
            rows.push(Sel::Colour);
        }
        rows
    }

    pub fn input(&mut self, look: Backdrop, wallpaper: bool, action: Action) -> Outcome {
        let rows = Self::rows(look);
        if !rows.contains(&self.sel) {
            self.sel = Sel::Style;
        }
        let forward = match action {
            Action::GbaDown(Btn::B) | Action::QuickMenu => return Outcome::Close,
            Action::GbaDown(Btn::Up) => {
                self.sel = step(&rows, self.sel, false);
                return Outcome::Stay;
            }
            Action::GbaDown(Btn::Down) => {
                self.sel = step(&rows, self.sel, true);
                return Outcome::Stay;
            }
            Action::GbaDown(Btn::Left) => false,
            Action::GbaDown(Btn::Right | Btn::A) => true,
            _ => return Outcome::Stay,
        };
        let mut next = look;
        match self.sel {
            Sel::Style => next.style = step(&styles(wallpaper), look.style, forward),
            Sel::Motion => next.moving = !look.moving,
            Sel::Colour => {
                let colours: Vec<u8> = (0..BACKDROP_COLOURS.len() as u8).collect();
                next.colour = step(&colours, look.colour, forward);
            }
        }
        Outcome::Changed(next)
    }

    pub fn view(&self, look: Backdrop, wallpaper: bool) -> ListView {
        let rows = Self::rows(look);
        let at = rows.iter().position(|r| *r == self.sel).unwrap_or(0);
        let row = |sel: &Sel| match sel {
            Sel::Style => ViewRow {
                label: RowLabel::Setting("Style"),
                value: Some(RowValue::Choice(look.style.label())),
                signal: None,
            },
            Sel::Motion => ViewRow {
                label: RowLabel::Setting("Motion"),
                value: Some(RowValue::Choice(if look.moving {
                    "Moving"
                } else {
                    "Still"
                })),
                signal: None,
            },
            Sel::Colour => ViewRow {
                label: RowLabel::Setting("Colour"),
                value: Some(RowValue::Choice(backdrop_colour_name(look.colour))),
                signal: None,
            },
        };
        let status = if look.style == BackdropStyle::Wallpaper {
            "Your own picture, from Wallpapers/."
        } else if wallpaper {
            "Pick Wallpaper to see your picture again."
        } else {
            "Or put a .png of your own in Wallpapers/."
        };
        ListView {
            rows: rows.iter().map(row).collect(),
            lit: Some(at),
            status: vec![status.to_string()],
            legend: vec![LegendHint::Back, LegendHint::Change],
        }
    }
}
