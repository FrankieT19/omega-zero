//! Per-game source cropping and centred zoom. Values are percentages of the visible frame.
use crate::{
    app::{App, Phase},
    list_view::{LegendHint, ListView, RowLabel, RowValue, ViewRow},
};
use omega_gfx::GameView;
use omega_input::{Action, Btn};
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Picture {
    pub zoom: u16,
    pub edges: [u8; 4],
    pub scaling: Option<u8>,
}
impl Default for Picture {
    fn default() -> Self {
        Self {
            zoom: 100,
            edges: [0; 4],
            scaling: None,
        }
    }
}
impl Picture {
    pub fn apply(self, view: &mut GameView) {
        let [left, right, top, bottom] = self.edges.map(|v| v as f32 / 100.0);
        let [u, v, w, h] = view.uv;
        let cw = w * (1.0 - left - right);
        let ch = h * (1.0 - top - bottom);
        let zoom = self.zoom as f32 / 100.0;
        view.uv = [
            u + w * left + cw * (1.0 - 1.0 / zoom) / 2.0,
            v + h * top + ch * (1.0 - 1.0 / zoom) / 2.0,
            cw / zoom,
            ch / zoom,
        ];
    }
    pub fn parse(value: &str) -> Option<Self> {
        let n: Vec<u16> = value
            .split(',')
            .map(str::parse)
            .collect::<Result<_, _>>()
            .ok()?;
        if !(5..=6).contains(&n.len())
            || !(100..=200).contains(&n[0])
            || n[1..5].iter().any(|v| *v > 20)
            || n.get(5).is_some_and(|v| *v > 3)
        {
            return None;
        }
        Some(Self {
            zoom: n[0],
            edges: [n[1] as u8, n[2] as u8, n[3] as u8, n[4] as u8],
            scaling: n.get(5).map(|v| *v as u8),
        })
    }
}
impl App {
    pub fn load_picture(&mut self, stem: &str) {
        self.picture_adjust = self
            .root()
            .and_then(|root| {
                omega_store::ini::value(
                    root,
                    "Config/picture.ini",
                    &format!("{}/{}", self.platform().dir_name(), stem),
                )
            })
            .as_deref()
            .and_then(Picture::parse)
            .unwrap_or_default();
        if self.platform() == omega_store::Platform::Snes {
            self.picture_adjust.scaling = None;
        }
        self.video_mode = self
            .picture_adjust
            .scaling
            .map(|n| crate::video_mode::VideoMode::from_scale(n, self.platform()))
            .unwrap_or_else(|| self.preferred_video_mode());
        self.shader_override = self
            .root()
            .and_then(|root| {
                omega_store::ini::value(
                    root,
                    "Config/shaders.ini",
                    &format!("{}/{}", self.platform().dir_name(), stem),
                )
            })
            .and_then(|value| value.parse::<u8>().ok())
            .filter(|value| omega_store::shader_choices(self.platform()).contains(value));
        self.picture_menu = None;
    }
    pub(crate) fn persist_shader_override(&self) {
        if let (Some(root), Phase::Playing { cart }) = (self.root(), &self.phase) {
            // A non-numeric inherit marker disables this entry without touching other games.
            let value = self
                .shader_override
                .map(|n| n.to_string())
                .unwrap_or_else(|| "inherit".into());
            if let Err(e) = omega_store::ini::write(
                root,
                "Config/shaders.ini",
                &format!("{}/{}", self.platform().dir_name(), cart),
                &value,
            ) {
                eprintln!("omega-zero: shader override: {e}");
            }
        }
    }
    pub fn picture_menu_open(&self) -> bool {
        self.picture_menu.is_some()
    }
    pub(crate) fn picture_input(&mut self, action: Action) {
        let Some(row) = self.picture_menu else {
            return;
        };
        match action {
            Action::PictureMenu | Action::QuickMenu | Action::GbaDown(Btn::B | Btn::Start) => {
                self.picture_menu = None;
                return;
            }
            Action::GbaDown(Btn::Up) => {
                self.picture_menu = Some((row + 9) % 10);
                return;
            }
            Action::GbaDown(Btn::Down) => {
                self.picture_menu = Some((row + 1) % 10);
                return;
            }
            Action::GbaDown(Btn::Left | Btn::Right | Btn::A) => {
                let by = if action == Action::GbaDown(Btn::Left) {
                    -1
                } else {
                    1
                };
                match row {
                    0 => {
                        if self.platform() != omega_store::Platform::Gba {
                            let next = self.video_mode.cycle(self.platform(), by);
                            let index = next.scale();
                            if self.platform() == omega_store::Platform::Snes {
                                self.state.scales[omega_store::Platform::Snes as usize] = index;
                                self.picture_adjust.scaling = None;
                                self.persist();
                            } else {
                                self.picture_adjust.scaling = Some(index);
                            }
                            self.video_mode = next;
                        }
                    }
                    1 => {
                        self.picture_adjust.zoom =
                            (self.picture_adjust.zoom as i32 + by * 5).clamp(100, 200) as u16
                    }
                    2..=5 => {
                        self.picture_adjust.edges[row - 2] =
                            (self.picture_adjust.edges[row - 2] as i32 + by).clamp(0, 20) as u8
                    }
                    6 => {
                        self.shader_override = if self.shader_override.is_some() {
                            None
                        } else {
                            Some(self.shader_settings().0)
                        };
                        self.persist_shader_override();
                    }
                    7 => {
                        let next =
                            omega_store::next_shader(self.platform(), self.shader_settings().0, by);
                        if self.shader_override.is_some() {
                            self.shader_override = Some(next);
                            self.persist_shader_override();
                        } else {
                            self.state.shaders[self.platform() as usize] = next;
                            self.persist();
                        }
                    }
                    8 => {
                        self.picture_adjust = Picture::default();
                        self.video_mode = self.preferred_video_mode();
                    }
                    _ => {
                        self.picture_menu = None;
                        return;
                    }
                }
            }
            _ => return,
        }
        self.persist_picture();
    }
    pub(crate) fn persist_picture(&self) {
        if let (Some(root), Phase::Playing { cart }) = (self.root(), &self.phase) {
            let p = self.picture_adjust;
            let mut value = format!(
                "{},{},{},{},{}",
                p.zoom, p.edges[0], p.edges[1], p.edges[2], p.edges[3]
            );
            if let Some(scale) = p.scaling {
                value.push_str(&format!(",{scale}"));
            }
            if let Err(e) = omega_store::ini::write(
                root,
                "Config/picture.ini",
                &format!("{}/{}", self.platform().dir_name(), cart),
                &value,
            ) {
                eprintln!("omega-zero: picture: {e}");
            }
        }
    }
    pub(crate) fn picture_view(&self) -> Option<ListView> {
        let row = self.picture_menu?;
        let labels = [
            "Scaling",
            "Zoom",
            "Crop left",
            "Crop right",
            "Crop top",
            "Crop bottom",
            "Shader override",
            "Shader",
            "Reset",
            "Close",
        ];
        Some(ListView {
            rows: labels
                .iter()
                .enumerate()
                .map(|(i, label)| ViewRow {
                    label: RowLabel::Setting(label),
                    value: match i {
                        0 => Some(RowValue::Choice(self.video_mode.label(self.platform()))),
                        1 => Some(RowValue::Name(format!("{}%", self.picture_adjust.zoom))),
                        2..=5 => Some(RowValue::Name(format!(
                            "{}%",
                            self.picture_adjust.edges[i - 2]
                        ))),
                        6 => Some(RowValue::Checkbox(self.shader_override.is_some())),
                        7 => Some(RowValue::Choice(
                            omega_store::SHADER_NAMES[self.shader_settings().0 as usize],
                        )),
                        _ => None,
                    },
                    dim: i == 0 && self.platform() == omega_store::Platform::Gba,
                })
                .collect(),
            lit: Some(row),
            status: Vec::new(),
            legend: vec![LegendHint::Back, LegendHint::Change],
        })
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn crop_then_zoom_stays_inside_original_window() {
        let p = Picture::parse("125,10,5,2,8").unwrap();
        let mut view = GameView::canvas([0.2, 0.1, 0.6, 0.8]);
        p.apply(&mut view);
        assert!((view.uv[2] - 0.408).abs() < 0.0001);
        assert!((view.uv[3] - 0.576).abs() < 0.0001);
        assert!(view.uv[0] > 0.26 && view.uv[1] > 0.116);
        assert!(Picture::parse("99,0,0,0,0").is_none());
        assert!(Picture::parse("100,21,0,0,0").is_none());
    }
}
