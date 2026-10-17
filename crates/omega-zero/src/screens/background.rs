//! The THEME screen, opened from Settings: a lone panel, as flow.html's `#screen=background`,
//! its rows BACKGROUND, MOTION and COLOUR drawn over the background they are choosing, and FONT
//! over the bar it is choosing for. A new colour re-tints the panel, and every screen after it,
//! the next frame; a new font redraws the bar's actions under it in that frame too.

use omega_input::Action;
use omega_ui::kit::palette::Palette;
use omega_ui::kit::panel::panel_face;
use omega_ui::kit::row::panel_height;
use omega_ui::{Draw, SettingsRow, UndoFace};

use crate::app::{App, Phase};
use crate::background_screen::Outcome as BackgroundOutcome;
use crate::list_view::ListView;
use crate::nav::Dir;
use crate::screens::lone::{hold, rows_height, Lone};

/// How wide the panel is, in art pixels: flow.css's .lone.small.
pub const WIDTH: u32 = 200;

/// The panel's title.
pub const TITLE: &str = "THEME";

/// The THEME panel holding `view`'s rows, in `pal`.
pub fn background_face(view: &ListView, pal: &Palette) -> UndoFace {
    let content = rows_height(view.rows.len());
    let title = if view
        .rows
        .first()
        .is_some_and(|r| r.label == crate::list_view::RowLabel::Setting("System"))
    {
        "DISPLAY"
    } else {
        TITLE
    };
    let mut panel = panel_face(WIDTH, panel_height(content), title, true, pal);
    hold(&mut panel, &view.rows, view.lit, pal);
    panel.face
}

impl App {
    pub(crate) fn background_input(&mut self, action: Action) {
        if matches!(&self.phase, Phase::Background(s) if s.display) {
            return self.display_input(action);
        }
        let (look, font, wallpaper) = (self.backdrop(), self.state.font, self.wallpaper.is_some());
        let Phase::Background(screen) = &mut self.phase else {
            return;
        };
        match screen.input(look, font, wallpaper, action) {
            BackgroundOutcome::Close => self.go(
                Phase::Settings {
                    row: SettingsRow::Background,
                },
                Dir::Back,
            ),
            BackgroundOutcome::Changed(b) => {
                self.state.backdrop = Some(b.style);
                self.state.backdrop_moving = b.moving;
                self.state.backdrop_colour = b.colour;
                self.persist();
            }
            BackgroundOutcome::Font(font) => {
                self.state.font = font;
                self.persist();
            }
            BackgroundOutcome::Stay => {}
        }
    }

    /// The THEME screen's list, while it is open.
    pub fn background_view(&self) -> Option<ListView> {
        match &self.phase {
            Phase::Background(screen) if screen.display => Some(self.display_view(screen)),
            Phase::Background(screen) => Some(screen.view(self.backdrop(), self.state.font)),
            _ => None,
        }
    }

    pub fn set_background_panel(&mut self, panel: Lone) {
        self.background_panel = Some(panel);
    }

    /// The panel, over the background drawn under every screen, with nothing between.
    pub(crate) fn draw_background_screen(&self, out: &mut Vec<Draw>) {
        if let Some(panel) = &self.background_panel {
            panel.draw(out);
        }
    }
}

impl App {
    pub fn low_poly_labels(&self) -> bool {
        self.state.low_poly_labels
    }

    pub fn shader_settings(&self) -> (u8, f32, f32) {
        let i = self.platform() as usize;
        (self.state.shaders[i], 1.0, 1.0)
    }

    pub(crate) fn cycle_shader(&mut self) {
        let i = self.platform() as usize;
        self.state.shaders[i] = (self.state.shaders[i] + 1) % 8;
        self.hud
            .toast(omega_ui::Toast::Shader(self.state.shaders[i]), self.now());
        self.persist();
    }

    fn display_input(&mut self, action: Action) {
        use omega_input::Btn;
        let Phase::Background(screen) = &mut self.phase else {
            return;
        };
        let forward = match action {
            Action::GbaDown(Btn::B | Btn::Start) | Action::QuickMenu => {
                self.go(
                    Phase::Settings {
                        row: SettingsRow::Display,
                    },
                    Dir::Back,
                );
                return;
            }
            Action::GbaDown(Btn::Up) => {
                screen.display_row = (screen.display_row + 2) % 3;
                return;
            }
            Action::GbaDown(Btn::Down) => {
                screen.display_row = (screen.display_row + 1) % 3;
                return;
            }
            Action::GbaDown(Btn::Left) => false,
            Action::GbaDown(Btn::Right | Btn::A) => true,
            _ => return,
        };
        let i = screen.display_system;
        let step = |value: u8, count: u8| {
            if forward {
                (value + 1) % count
            } else {
                (value + count - 1) % count
            }
        };
        match screen.display_row {
            0 => screen.display_system = step(i as u8, 5) as usize,
            1 => self.state.shaders[i] = step(self.state.shaders[i], 8),
            2 => {
                self.state.low_poly_labels = !self.state.low_poly_labels;
                self.stale_faces = self.carts().map(|c| (c.platform, c.stem.clone())).collect();
            }
            _ => {}
        }
        self.persist();
    }

    fn display_view(&self, screen: &crate::background_screen::BackgroundScreen) -> ListView {
        use crate::list_view::{LegendHint, RowLabel, RowValue, ViewRow};
        let i = screen.display_system;
        let row = |label, value| ViewRow {
            label: RowLabel::Setting(label),
            value: Some(value),
            dim: false,
        };
        ListView {
            rows: vec![
                row(
                    "System",
                    RowValue::Choice(omega_store::Platform::ALL[i].dir_name()),
                ),
                row(
                    "Shader",
                    RowValue::Choice(omega_store::SHADER_NAMES[self.state.shaders[i] as usize]),
                ),
                row(
                    "Low poly labels",
                    RowValue::Setting(self.state.low_poly_labels),
                ),
            ],
            lit: Some(screen.display_row),
            status: Vec::new(),
            legend: vec![LegendHint::Back, LegendHint::Change],
        }
    }
}
