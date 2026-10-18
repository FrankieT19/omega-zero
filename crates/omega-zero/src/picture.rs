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
}
impl Default for Picture {
    fn default() -> Self {
        Self {
            zoom: 100,
            edges: [0; 4],
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
        if n.len() != 5 || !(100..=200).contains(&n[0]) || n[1..].iter().any(|v| *v > 20) {
            return None;
        }
        Some(Self {
            zoom: n[0],
            edges: [n[1] as u8, n[2] as u8, n[3] as u8, n[4] as u8],
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
        self.picture_menu = None;
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
                self.picture_menu = Some((row + 6) % 7);
                return;
            }
            Action::GbaDown(Btn::Down) => {
                self.picture_menu = Some((row + 1) % 7);
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
                        self.picture_adjust.zoom =
                            (self.picture_adjust.zoom as i32 + by * 5).clamp(100, 200) as u16
                    }
                    1..=4 => {
                        self.picture_adjust.edges[row - 1] =
                            (self.picture_adjust.edges[row - 1] as i32 + by).clamp(0, 20) as u8
                    }
                    5 => self.picture_adjust = Picture::default(),
                    _ => {
                        self.picture_menu = None;
                        return;
                    }
                }
            }
            _ => return,
        }
        if let (Some(root), Phase::Playing { cart }) = (self.root(), &self.phase) {
            let p = self.picture_adjust;
            let value = format!(
                "{},{},{},{},{}",
                p.zoom, p.edges[0], p.edges[1], p.edges[2], p.edges[3]
            );
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
            "Zoom",
            "Crop left",
            "Crop right",
            "Crop top",
            "Crop bottom",
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
                        0 => Some(RowValue::Name(format!("{}%", self.picture_adjust.zoom))),
                        1..=4 => Some(RowValue::Name(format!(
                            "{}%",
                            self.picture_adjust.edges[i - 1]
                        ))),
                        _ => None,
                    },
                    dim: false,
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
