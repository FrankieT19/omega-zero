//! Monotonic inactivity policy. The final fade uses frame time bounded independently of
//! emulation stalls; reaching a timeout never skips straight to a black backlight.
use crate::app::{App, Phase};
use crate::background_screen::BackgroundScreen;
use crate::list_view::{LegendHint, ListView, RowLabel, RowValue, ViewRow};
use crate::nav::Dir;
use omega_input::{Action, Btn, Millis};
use omega_ui::SettingsRow;

pub(crate) struct Idle {
    last: Millis,
    context: Option<usize>,
    gain: f32,
}
impl Default for Idle {
    fn default() -> Self {
        Self {
            last: 0,
            context: None,
            gain: 1.0,
        }
    }
}
impl Idle {
    fn touch(&mut self, now: Millis) {
        self.last = now;
    }
    fn step(&mut self, now: Millis, dt: f32, context: usize, dim: u16, sleep: u16) -> bool {
        if self.context != Some(context) {
            self.context = Some(context);
            self.touch(now);
        }
        let age = now.saturating_sub(self.last);
        let sleeping = sleep > 0 && age >= u64::from(sleep) * 1000;
        let target = if sleeping {
            0.0
        } else if dim > 0 && age >= u64::from(dim) * 1000 {
            0.25
        } else {
            1.0
        };
        let step = if dt.is_finite() {
            dt.clamp(0.0, 0.05)
        } else {
            0.0
        };
        self.gain += (target - self.gain).clamp(-step, step);
        sleeping && self.gain <= 0.001
    }
}
impl App {
    pub(crate) fn note_activity(&mut self) {
        self.inactivity.touch(self.now());
    }
    pub fn inactivity_gain(&self) -> f32 {
        self.inactivity.gain
    }
    pub(crate) fn step_inactivity(&mut self, dt: f32) {
        if matches!(self.phase, Phase::Doze { .. }) {
            self.inactivity.gain = 0.0;
            self.inactivity.context = None;
            return;
        }
        if self.power.is_none() || self.powering_off {
            return;
        }
        let context = usize::from(
            matches!(self.phase, Phase::Playing { .. })
                && self.game_menu.is_none()
                && self.picture_menu.is_none()
                && self.power_screen.is_none(),
        );
        let now = self.now();
        if self.inactivity.step(
            now,
            dt,
            context,
            self.state.idle_dim[context],
            self.state.idle_sleep[context],
        ) {
            self.doze();
        }
    }
    pub(crate) fn release_shelf_holds(&mut self) {
        for (_, shelf) in &mut self.shelves {
            shelf.release_hold();
        }
    }
    pub(crate) fn inactivity_input(&mut self, action: Action) {
        let Phase::Background(screen) = &mut self.phase else {
            return;
        };
        let forward = match action {
            Action::GbaDown(Btn::B | Btn::Start) | Action::QuickMenu => {
                self.go(
                    Phase::Settings {
                        row: SettingsRow::Inactivity,
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
        if screen.display_row == 0 {
            screen.display_system = 1 - screen.display_system;
            return;
        }
        let value = if screen.display_row == 1 {
            &mut self.state.idle_dim[screen.display_system]
        } else {
            &mut self.state.idle_sleep[screen.display_system]
        };
        const TIMES: [u16; 10] = [0, 15, 30, 60, 120, 300, 600, 900, 1800, 3600];
        let at = TIMES.iter().position(|v| v == value).unwrap_or(0);
        *value = TIMES[if forward {
            (at + 1) % TIMES.len()
        } else {
            (at + TIMES.len() - 1) % TIMES.len()
        }];
        self.persist();
    }
    pub(crate) fn inactivity_view(&self, screen: &BackgroundScreen) -> ListView {
        let i = screen.display_system;
        let time = |s| {
            if s == 0 {
                "Off".to_string()
            } else if s < 60 {
                format!("{s}s")
            } else {
                format!("{} min", s / 60)
            }
        };
        let row = |label, value| ViewRow {
            label: RowLabel::Setting(label),
            value: Some(RowValue::Name(value)),
            dim: false,
        };
        ListView {
            rows: vec![
                row("Context", if i == 0 { "Menus" } else { "In game" }.into()),
                row("Dim after", time(self.state.idle_dim[i])),
                row("Sleep after", time(self.state.idle_sleep[i])),
            ],
            lit: Some(screen.display_row),
            status: Vec::new(),
            legend: vec![LegendHint::Back, LegendHint::Change],
        }
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn timeouts_fade_before_sleep_and_activity_fades_back() {
        let mut idle = Idle::default();
        idle.step(0, 0.02, 0, 15, 30);
        assert!(!idle.step(15000, 0.02, 0, 15, 30));
        assert!(idle.gain < 1.0 && idle.gain > 0.25);
        for _ in 0..50 {
            idle.step(16000, 0.02, 0, 15, 30);
        }
        assert!((idle.gain - 0.25).abs() < 0.001);
        idle.touch(16000);
        idle.step(16001, 0.02, 0, 15, 30);
        assert!(idle.gain > 0.25 && idle.gain < 1.0);
        assert!(!idle.step(60000, 100.0, 0, 15, 30));
        let mut slept = false;
        for _ in 0..50 {
            slept |= idle.step(60000, 0.02, 0, 15, 30);
        }
        assert!(slept);
    }
    #[test]
    fn contexts_reset_timers_and_off_stays_awake() {
        let mut idle = Idle::default();
        idle.step(0, 0.02, 0, 15, 30);
        assert!(!idle.step(100000, 0.02, 1, 15, 30));
        assert_eq!(idle.gain, 1.0);
        for _ in 0..100 {
            assert!(!idle.step(200000, 0.02, 1, 0, 0));
        }
        assert_eq!(idle.gain, 1.0);
    }
}
