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
    let title = match view.rows.first().map(|r| &r.label) {
        Some(crate::list_view::RowLabel::Setting("System")) => "DISPLAY",
        Some(crate::list_view::RowLabel::Setting("Scaling")) => "PICTURE",
        _ => TITLE,
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
        if let Some(view) = self.picture_view() {
            return Some(view);
        }
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

    /// Palettes are supplied by Gambatte, not a post-processing colour remap.
    pub fn game_palette(&self) -> u8 {
        0
    }
    pub fn gb_palette(&self) -> u8 {
        self.state.palettes[omega_store::Platform::Gb as usize]
    }
    pub fn take_palette(&mut self) -> Option<u8> {
        if std::mem::take(&mut self.palette_pending) {
            Some(self.gb_palette())
        } else {
            None
        }
    }
    pub(crate) fn cycle_palette(&mut self) {
        if self.platform() != omega_store::Platform::Gb
            || self.core() != omega_store::Core::Gambatte
        {
            return;
        }
        let i = omega_store::Platform::Gb as usize;
        self.state.palettes[i] =
            (self.state.palettes[i] + 1) % omega_store::GB_PALETTES.len() as u8;
        self.palette_pending = true;
        self.hud
            .toast(omega_ui::Toast::Palette(self.state.palettes[i]), self.now());
        self.persist();
    }
    pub fn shader_saturation(&self) -> f32 {
        self.state.saturation[self.platform() as usize] as f32 / 100.0
    }
    pub fn lcd_darkness(&self) -> f32 {
        self.state.lcd_darkness[self.platform() as usize] as f32 / 100.0
    }

    pub fn shader_shadow(&self) -> (f32, f32) {
        let i = self.platform() as usize;
        (
            self.state.shadow_x[i] as f32 / 2.0,
            self.state.shadow_y[i] as f32 / 2.0,
        )
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
                screen.display_row = (screen.display_row + 7) % 8;
                return;
            }
            Action::GbaDown(Btn::Down) => {
                screen.display_row = (screen.display_row + 1) % 8;
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
            2 if i > 0 => self.state.scales[i] = step(self.state.scales[i], 3),
            3 => {
                self.state.shadow_x[i] =
                    (self.state.shadow_x[i] + if forward { 1 } else { -1 }).clamp(-60, 60)
            }
            4 => {
                self.state.shadow_y[i] =
                    (self.state.shadow_y[i] + if forward { 1 } else { -1 }).clamp(-60, 60)
            }
            7 => {
                self.state.lcd_darkness[i] = (self.state.lcd_darkness[i] as i16
                    + if forward { 10 } else { -10 })
                .clamp(0, 200) as u8;
            }
            6 => {
                self.state.saturation[i] = (self.state.saturation[i] as i16
                    + if forward { 10 } else { -10 })
                .clamp(0, 200) as u8;
            }
            5 => {
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
                ViewRow {
                    label: RowLabel::Setting("Scaling"),
                    value: Some(RowValue::Choice(
                        ["Integer", "Fit", "Stretch"][if i == 0 {
                            0
                        } else {
                            self.state.scales[i] as usize
                        }],
                    )),
                    dim: i == 0,
                },
                row(
                    "Shadow X",
                    RowValue::Name(format!("{:.1}", self.state.shadow_x[i] as f32 / 2.0)),
                ),
                row(
                    "Shadow Y",
                    RowValue::Name(format!("{:.1}", self.state.shadow_y[i] as f32 / 2.0)),
                ),
                row(
                    "Low poly labels",
                    RowValue::Setting(self.state.low_poly_labels),
                ),
                ViewRow {
                    label: RowLabel::Setting("Saturation"),
                    value: Some(RowValue::Name(format!("{}%", self.state.saturation[i]))),
                    dim: false,
                },
                row(
                    "LCD darkness",
                    RowValue::Name(format!("{}%", self.state.lcd_darkness[i])),
                ),
            ],
            lit: Some(screen.display_row),
            status: Vec::new(),
            legend: vec![LegendHint::Back, LegendHint::Change],
        }
    }
}
