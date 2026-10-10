//! Controls, opened with a SELECT tap on the shelf and titled QUICKMENU: what each button does
//! on the shelf and in a game, as `App::apply` and the gestures in omega-input do it.

use omega_gfx::{out_w, Draw, TexId};
use omega_input::{Action, Btn};
use omega_ui::{centred_hints, stage_y, HINT_H, LEGEND_GAP, MENU_PAD};

use crate::app::{App, Phase};

/// A group of controls: its name, and each control's keys and what they do.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ControlsGroup {
    pub name: &'static str,
    pub lines: Vec<(&'static str, &'static str)>,
}

/// What the Controls screen says.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ControlsView {
    pub title: &'static str,
    pub groups: Vec<ControlsGroup>,
}

impl ControlsView {
    pub fn new() -> Self {
        ControlsView {
            title: "Quickmenu",
            groups: vec![
                ControlsGroup {
                    name: "Shelf",
                    lines: vec![
                        ("A", "Play"),
                        ("Hold A", "Clean start"),
                        ("X", "Core picker (GBA)"),
                        ("START", "Settings"),
                        ("SELECT", "Quickmenu"),
                        ("◀▶", "Browse"),
                        ("▲▼", "Letter jump"),
                        ("L1/R1", "Switch shelf"),
                    ],
                },
                ControlsGroup {
                    name: "In Game",
                    lines: vec![
                        ("L2", "Rewind"),
                        ("R2", "Fast forward"),
                        ("R2 ×2", "Latch fast forward"),
                        ("MENU ×2", "Save states"),
                        ("Hold MENU", "Eject"),
                        ("SELECT+MENU", "Link"),
                        ("SELECT+▲▼", "Brightness"),
                        ("SELECT+◀▶", "Blue light"),
                        ("SELECT+L1", "Load state"),
                        ("SELECT+R1", "Save state"),
                        ("SELECT+Y", "Colour correction"),
                        ("VOL+ VOL-", "Mute, together"),
                        ("Hold POWER", "Power menu"),
                        ("L1/R1", "Picture size (GB)"),
                    ],
                },
            ],
        }
    }
}

impl Default for ControlsView {
    fn default() -> Self {
        ControlsView::new()
    }
}

pub type Face = (TexId, u32, u32);

/// A group's column: where its ink starts across the screen, its name, and its lines.
pub struct ControlsColumn {
    pub x: f32,
    pub heading: Face,
    pub lines: Vec<Face>,
}

/// The Controls screen's faces: its title, a column a group, and B BACK.
pub struct ControlsFaces {
    pub title: Face,
    pub columns: Vec<ControlsColumn>,
    pub back: (TexId, u32),
}

const TITLE_Y: f32 = 24.0;
const HEADING_Y: f32 = 68.0;
const LINES_Y: f32 = 112.0;
pub const LINE_H: u32 = 22;
pub const LINE_PX: f32 = 16.0;
/// Between the ink of one column and the next.
pub const COLUMN_GAP: f32 = 8.0;
/// Inside the screen's edges: a little nearer them than the lists', for the longest lines.
const EDGE: f32 = 24.0;

/// Where the first column's ink starts, and the next column's after `ink` of the one before.
pub fn column_x(before: Option<(f32, f32)>) -> f32 {
    match before {
        None => EDGE,
        Some((x, ink)) => x + ink + COLUMN_GAP,
    }
}

/// How far a column starting at `x` may run before the screen's right margin.
pub fn column_room(x: f32) -> f32 {
    out_w() as f32 - EDGE - x
}

fn legend_y() -> f32 {
    427.0 + stage_y()
}

impl App {
    pub(crate) fn controls_input(&mut self, action: Action) {
        if matches!(action, Action::GbaDown(Btn::B) | Action::SelectTap) {
            self.phase = Phase::Shelf;
        }
    }

    /// What the Controls screen says, while it is open.
    pub fn controls_view(&self) -> Option<ControlsView> {
        matches!(self.phase, Phase::Controls).then(ControlsView::new)
    }

    pub fn set_controls_faces(&mut self, faces: ControlsFaces) {
        self.controls_faces = Some(faces);
    }

    pub fn controls_faces_ready(&self) -> bool {
        self.controls_faces.is_some()
    }

    pub(crate) fn draw_controls(&self, out: &mut Vec<Draw>) {
        let Some(faces) = &self.controls_faces else {
            return;
        };
        let title = faces.title;
        push(out, title, (out_w() as f32 - title.1 as f32) / 2.0, TITLE_Y);
        for column in &faces.columns {
            // a face's ink starts MENU_PAD in from its edge
            let x = column.x - MENU_PAD as f32;
            push(out, column.heading, x, HEADING_Y);
            for (row, line) in column.lines.iter().enumerate() {
                push(out, *line, x, LINES_Y + (LINE_H as usize * row) as f32);
            }
        }
        for (tex, w, x) in centred_hints(&[faces.back], LEGEND_GAP) {
            push(out, (tex, w, HINT_H), x, legend_y());
        }
    }
}

fn push(out: &mut Vec<Draw>, (tex, w, h): Face, x: f32, y: f32) {
    out.push(Draw::Tex {
        x: x.round(),
        y: y.round(),
        w: w as f32,
        h: h as f32,
        tex,
        alpha: 1.0,
    });
}
