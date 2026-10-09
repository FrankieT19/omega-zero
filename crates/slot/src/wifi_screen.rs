//! The Wi-Fi screen, opened from the quick menu: switching Wi-Fi and network time, the
//! networks in reach, and typing a passphrase for a new one.

use slot_input::{Action, Btn, Millis};
use slot_ui::{wifi_window, Keyboard};

use crate::list_view::{LegendHint, ListView, RowLabel, RowValue, ViewRow};
use crate::wifi::{
    bars, passphrase_ok, shown, AskFail, Saved, Seen, Status, WifiJob, WifiJobs, WifiNews,
    JOIN_TIMEOUT_MS, REFRESH_MS, SCAN_MS,
};

/// How closely the radio is being watched, which sets how often it is asked.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum Watch {
    Idle,
    /// Something on screen names the network: its status is kept fresh.
    Status,
    /// The Wi-Fi screen is open: the networks in reach are looked for too.
    Networks,
}

/// Why the last attempt went wrong, said under the list until the next one.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Note {
    NoRadio,
    WrongPassphrase(Vec<u8>),
    NoJoin(Vec<u8>),
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum EntryState {
    Joined,
    Joining,
    Saved,
    New,
}

/// A network the list offers: one in reach, one saved, or both.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Entry {
    pub ssid: Vec<u8>,
    pub rssi: Option<i32>,
    pub secured: bool,
    pub saved: Option<u32>,
    pub state: EntryState,
}

/// What the quick menu says beside Wi-Fi.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Summary {
    Off,
    On,
    Joined(String),
}

/// The radio as the app knows it, and the jobs that change it.
pub struct Wifi {
    jobs: Box<dyn WifiJobs>,
    enabled: bool,
    status: Status,
    seen: Vec<Seen>,
    empty_scans: u8,
    saved: Vec<Saved>,
    joining: Option<(Vec<u8>, Millis)>,
    note: Option<Note>,
    refreshing: bool,
    scanning: bool,
    refresh_at: Millis,
    scan_at: Millis,
}

impl Wifi {
    pub fn new(jobs: Box<dyn WifiJobs>) -> Self {
        Wifi {
            jobs,
            enabled: false,
            status: Status::default(),
            seen: Vec::new(),
            empty_scans: 0,
            saved: Vec::new(),
            joining: None,
            note: None,
            refreshing: false,
            scanning: false,
            refresh_at: 0,
            scan_at: 0,
        }
    }

    pub fn present(&self) -> bool {
        self.jobs.present()
    }

    pub fn enabled(&self) -> bool {
        self.enabled
    }

    pub fn status(&self) -> &Status {
        &self.status
    }

    pub fn note(&self) -> Option<&Note> {
        self.note.as_ref()
    }

    /// At boot: brings Wi-Fi up if it was left on.
    pub fn start(&mut self, enabled: bool) {
        self.enabled = enabled && self.present();
        if self.enabled {
            self.jobs.ask(WifiJob::Up);
        }
    }

    pub fn set_enabled(&mut self, on: bool, now: Millis) {
        if on == self.enabled || !self.present() {
            return;
        }
        self.enabled = on;
        self.note = None;
        self.joining = None;
        if on {
            self.jobs.ask(WifiJob::Up);
            self.refresh_at = now;
            self.scan_at = now;
        } else {
            self.jobs.ask(WifiJob::Down);
            self.seen.clear();
            self.status = Status {
                hostname: std::mem::take(&mut self.status.hostname),
                ntp: self.status.ntp,
                ..Status::default()
            };
        }
    }

    pub fn ntp(&self) -> bool {
        self.status.ntp
    }

    pub fn set_ntp(&mut self, on: bool) {
        self.status.ntp = on;
        self.jobs.ask(WifiJob::Clock(on));
    }

    pub fn join(&mut self, ssid: Vec<u8>, psk: String, replace: Option<u32>, now: Millis) {
        self.note = None;
        self.joining = Some((ssid.clone(), now));
        self.jobs.ask(WifiJob::Join { ssid, psk, replace });
    }

    pub fn select(&mut self, id: u32, ssid: Vec<u8>, now: Millis) {
        self.note = None;
        self.joining = Some((ssid, now));
        self.jobs.ask(WifiJob::Select(id));
    }

    pub fn forget(&mut self, id: u32) {
        let gone = self
            .saved
            .iter()
            .find(|s| s.id == id)
            .map(|s| s.ssid.clone());
        if gone.is_some() && self.joining.as_ref().map(|(s, _)| s) == gone.as_ref() {
            self.joining = None;
        }
        self.saved.retain(|s| s.id != id);
        self.jobs.ask(WifiJob::Forget(id));
    }

    /// Takes what the radio has said since the last tick, settles a join that has finished,
    /// and asks again whatever is being watched.
    pub fn tick(&mut self, now: Millis, watch: Watch) {
        for news in self.jobs.news() {
            self.take(news, now);
        }
        if let Some((ssid, since)) = &self.joining {
            let refused = self.saved.iter().any(|s| s.ssid == *ssid && s.refused);
            if self.status.joined() && self.status.ssid.as_ref() == Some(ssid) {
                self.joining = None;
            } else if refused {
                self.note = Some(Note::WrongPassphrase(ssid.clone()));
                self.joining = None;
            } else if now.saturating_sub(*since) > JOIN_TIMEOUT_MS {
                self.note = Some(Note::NoJoin(ssid.clone()));
                self.joining = None;
            }
        }
        if !self.enabled {
            return;
        }
        let watching = watch >= Watch::Status || self.joining.is_some();
        if watching && !self.refreshing && now >= self.refresh_at {
            self.refreshing = true;
            self.jobs.ask(WifiJob::Refresh);
        }
        if watch == Watch::Networks && !self.scanning && now >= self.scan_at {
            self.scanning = true;
            self.jobs.ask(WifiJob::Scan);
        }
    }

    fn take(&mut self, news: WifiNews, now: Millis) {
        match news {
            WifiNews::Started(true) => {
                if self.note == Some(Note::NoRadio) {
                    self.note = None;
                }
            }
            WifiNews::Started(false) => {
                if self.enabled {
                    self.note = Some(Note::NoRadio);
                }
            }
            WifiNews::Status(status) => {
                if self.enabled {
                    self.status = status;
                } else {
                    self.status.hostname = status.hostname;
                    self.status.ntp = status.ntp;
                }
            }
            WifiNews::Saved(saved) => {
                self.saved = saved;
                self.refreshing = false;
                self.refresh_at = now + REFRESH_MS;
            }
            WifiNews::Seen(seen) => {
                self.scanning = false;
                self.scan_at = now + SCAN_MS;
                // A scan the radio was too busy for comes back empty: keep what was heard
                // until a second one agrees.
                if seen.is_empty() && self.empty_scans == 0 && self.enabled {
                    self.empty_scans = 1;
                    return;
                }
                self.empty_scans = 0;
                if self.enabled {
                    self.seen = seen;
                }
            }
            WifiNews::Asked(Ok(())) => {}
            WifiNews::Asked(Err(fail)) => {
                if let Some((ssid, _)) = self.joining.take() {
                    self.note = Some(match fail {
                        AskFail::BadPassphrase => Note::WrongPassphrase(ssid),
                        AskFail::Radio => Note::NoJoin(ssid),
                    });
                }
            }
        }
    }

    fn joined_ssid(&self) -> Option<&Vec<u8>> {
        self.status
            .ssid
            .as_ref()
            .filter(|_| self.enabled && self.status.joined())
    }

    fn joining_ssid(&self) -> Option<&Vec<u8>> {
        let handshaking = matches!(
            self.status.state.as_str(),
            "AUTHENTICATING" | "ASSOCIATING" | "ASSOCIATED" | "4WAY_HANDSHAKE" | "GROUP_HANDSHAKE"
        );
        self.joining
            .as_ref()
            .map(|(s, _)| s)
            .or(self.status.ssid.as_ref().filter(|_| handshaking))
    }

    /// The networks to offer: the one joined or being joined first, then those in reach,
    /// strongest first, then the saved ones out of reach.
    pub fn entries(&self) -> Vec<Entry> {
        if !self.enabled {
            return Vec::new();
        }
        let saved_id = |ssid: &[u8]| self.saved.iter().find(|s| s.ssid == ssid).map(|s| s.id);
        let state = |ssid: &[u8], saved: Option<u32>| {
            if self.joining_ssid().is_some_and(|s| s == ssid) {
                EntryState::Joining
            } else if self.joined_ssid().is_some_and(|s| s == ssid) {
                EntryState::Joined
            } else if saved.is_some() {
                EntryState::Saved
            } else {
                EntryState::New
            }
        };
        let mut out: Vec<Entry> = self
            .seen
            .iter()
            .map(|s| {
                let saved = saved_id(&s.ssid);
                Entry {
                    ssid: s.ssid.clone(),
                    rssi: Some(s.rssi),
                    secured: s.secured,
                    saved,
                    state: state(&s.ssid, saved),
                }
            })
            .collect();
        for s in &self.saved {
            if !out.iter().any(|e| e.ssid == s.ssid) {
                out.push(Entry {
                    ssid: s.ssid.clone(),
                    rssi: None,
                    secured: true,
                    saved: Some(s.id),
                    state: state(&s.ssid, Some(s.id)),
                });
            }
        }
        out.sort_by_key(|e| match e.state {
            EntryState::Joined | EntryState::Joining => 0,
            _ if e.rssi.is_some() => 1,
            _ => 2,
        });
        out
    }

    pub fn summary(&self) -> Summary {
        match (self.enabled, self.joined_ssid()) {
            (false, _) => Summary::Off,
            (true, Some(ssid)) => Summary::Joined(shown(ssid)),
            (true, None) => Summary::On,
        }
    }

    /// The lines under the list.
    pub fn status_lines(&self) -> Vec<String> {
        if !self.present() {
            return vec!["This handheld has no Wi-Fi.".into()];
        }
        if !self.enabled {
            return vec!["Wi-Fi is off.".into()];
        }
        match &self.note {
            Some(Note::NoRadio) => return vec!["The Wi-Fi radio did not start.".into()],
            Some(Note::WrongPassphrase(s)) => {
                return vec![format!("Could not join {}. Wrong passphrase?", shown(s))]
            }
            Some(Note::NoJoin(s)) => return vec![format!("Could not join {}.", shown(s))],
            None => {}
        }
        if let Some(ssid) = self.joining_ssid() {
            return vec![format!("Joining {}…", shown(ssid))];
        }
        let Some(ssid) = self.joined_ssid() else {
            return vec!["Not connected. Pick a network to join.".into()];
        };
        let Some(ip) = &self.status.ip else {
            return vec![format!("Joined {}, waiting for an address…", shown(ssid))];
        };
        let mut lines = vec![format!("Connected to {} as {ip}", shown(ssid))];
        let host = &self.status.hostname;
        lines.push(if host.is_empty() {
            format!("SSH and SFTP: root@{ip}")
        } else {
            format!("SSH and SFTP: root@{host}.local or root@{ip}")
        });
        lines
    }
}

/// A row of the Wi-Fi screen, held by what it is rather than where it is, so a scan that
/// reorders the list leaves the cursor on the same network.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Sel {
    Wifi,
    Clock,
    Labels,
    Network(Vec<u8>),
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Typing {
    pub ssid: Vec<u8>,
    pub replace: Option<u32>,
    pub keys: Keyboard,
    /// The last Join was refused for being too short or too long.
    pub refused: bool,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct WifiScreen {
    pub sel: Sel,
    /// Where the cursor was, for when its network drops out of the list.
    pub at: usize,
    /// The first row in view.
    pub first: usize,
    pub typing: Option<Typing>,
}

impl Default for WifiScreen {
    fn default() -> Self {
        WifiScreen {
            sel: Sel::Wifi,
            at: 0,
            first: 0,
            typing: None,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Outcome {
    Stay,
    Close,
    /// Open the Labels screen, which fetches cart labels over Wi-Fi.
    Labels,
}

impl WifiScreen {
    fn rows(&self, wifi: &Wifi) -> Vec<Sel> {
        let mut rows = vec![Sel::Wifi];
        if wifi.enabled() {
            rows.push(Sel::Clock);
            rows.push(Sel::Labels);
            rows.extend(wifi.entries().into_iter().map(|e| Sel::Network(e.ssid)));
        }
        rows
    }

    /// The cursor's row now, settling it on a neighbour if its network has gone.
    fn settle(&mut self, rows: &[Sel]) -> usize {
        let at = rows
            .iter()
            .position(|r| *r == self.sel)
            .unwrap_or(self.at.min(rows.len() - 1));
        self.sel = rows[at].clone();
        self.at = at;
        self.first = wifi_window(at, rows.len(), self.first);
        at
    }

    fn entry(&self, wifi: &Wifi) -> Option<Entry> {
        let Sel::Network(ssid) = &self.sel else {
            return None;
        };
        wifi.entries().into_iter().find(|e| e.ssid == *ssid)
    }

    pub fn input(&mut self, wifi: &mut Wifi, action: Action, now: Millis) -> Outcome {
        if self.typing.is_some() {
            self.type_input(wifi, action, now);
            return Outcome::Stay;
        }
        let rows = self.rows(wifi);
        let at = self.settle(&rows);
        match action {
            Action::GbaDown(Btn::B) | Action::QuickMenu => return Outcome::Close,
            Action::GbaDown(Btn::Up) => self.sel = rows[(at + rows.len() - 1) % rows.len()].clone(),
            Action::GbaDown(Btn::Down) => self.sel = rows[(at + 1) % rows.len()].clone(),
            Action::GbaDown(Btn::Left | Btn::Right | Btn::A) if self.sel == Sel::Wifi => {
                wifi.set_enabled(!wifi.enabled(), now);
            }
            Action::GbaDown(Btn::Left | Btn::Right | Btn::A) if self.sel == Sel::Clock => {
                wifi.set_ntp(!wifi.ntp());
            }
            Action::GbaDown(Btn::A) if self.sel == Sel::Labels => return Outcome::Labels,
            Action::GbaDown(Btn::A) => self.pick(wifi, now),
            Action::GbaDown(Btn::Y) => {
                if let Some(id) = self.entry(wifi).and_then(|e| e.saved) {
                    wifi.forget(id);
                }
            }
            _ => {}
        }
        let rows = self.rows(wifi);
        self.settle(&rows);
        Outcome::Stay
    }

    fn pick(&mut self, wifi: &mut Wifi, now: Millis) {
        let Some(e) = self.entry(wifi) else {
            return;
        };
        let turned_away = wifi.note() == Some(&Note::WrongPassphrase(e.ssid.clone()));
        match (e.state, e.saved) {
            (EntryState::Joined | EntryState::Joining, _) => {}
            (_, Some(id)) if !turned_away => wifi.select(id, e.ssid, now),
            _ if !e.secured => wifi.join(e.ssid, String::new(), e.saved, now),
            _ => {
                self.typing = Some(Typing {
                    ssid: e.ssid,
                    replace: e.saved,
                    keys: Keyboard::default(),
                    refused: false,
                })
            }
        }
    }

    fn type_input(&mut self, wifi: &mut Wifi, action: Action, now: Millis) {
        let Some(t) = &mut self.typing else {
            return;
        };
        let keys = &mut t.keys;
        let mut submit = false;
        match action {
            Action::GbaDown(Btn::Up) => keys.up(),
            Action::GbaDown(Btn::Down) => keys.down(),
            Action::GbaDown(Btn::Left) => keys.left(),
            Action::GbaDown(Btn::Right) => keys.right(),
            Action::GbaDown(Btn::A) => submit = keys.press(),
            Action::GbaDown(Btn::B) if keys.text.is_empty() => {
                self.typing = None;
                return;
            }
            Action::GbaDown(Btn::B) => keys.delete(),
            Action::GbaDown(Btn::X) => keys.shift(),
            Action::GbaDown(Btn::Y) => keys.type_char(' '),
            Action::GbaDown(Btn::L1 | Btn::R1) => keys.symbols(),
            Action::GbaDown(Btn::Start) => submit = true,
            Action::QuickMenu => {
                self.typing = None;
                return;
            }
            _ => return,
        }
        t.refused = false;
        if !submit {
            return;
        }
        if !passphrase_ok(&t.keys.text) {
            t.refused = true;
            return;
        }
        let Some(t) = self.typing.take() else {
            return;
        };
        self.sel = Sel::Network(t.ssid.clone());
        wifi.join(t.ssid, t.keys.text, t.replace, now);
    }

    pub fn view(&mut self, wifi: &Wifi) -> ListView {
        let rows = self.rows(wifi);
        let at = self.settle(&rows);
        let entries = wifi.entries();
        let shown_rows = slot_ui::wifi_rows_shown();
        let view_rows = rows
            .iter()
            .skip(self.first)
            .take(shown_rows)
            .map(|r| match r {
                Sel::Wifi => ViewRow {
                    label: RowLabel::Setting("Wi-Fi"),
                    value: wifi.present().then_some(RowValue::Setting(wifi.enabled())),
                    signal: None,
                },
                Sel::Clock => ViewRow {
                    label: RowLabel::Setting("Network Time"),
                    value: Some(RowValue::Setting(wifi.ntp())),
                    signal: None,
                },
                Sel::Labels => ViewRow {
                    label: RowLabel::Setting("Cart Labels"),
                    value: None,
                    signal: None,
                },
                Sel::Network(ssid) => {
                    let e = entries.iter().find(|e| e.ssid == *ssid);
                    ViewRow {
                        label: RowLabel::Name(shown(ssid)),
                        value: e.and_then(|e| match e.state {
                            EntryState::Joined => Some(RowValue::Said("Connected")),
                            EntryState::Joining => Some(RowValue::Said("Joining")),
                            EntryState::Saved => Some(RowValue::Said("Saved")),
                            EntryState::New => None,
                        }),
                        signal: e.and_then(|e| e.rssi.map(|r| (bars(r), e.secured))),
                    }
                }
            })
            .collect();
        let mut legend = vec![LegendHint::Back];
        match &self.sel {
            Sel::Wifi | Sel::Clock => legend.push(LegendHint::Change),
            Sel::Labels => legend.push(LegendHint::Open),
            Sel::Network(_) => {
                let e = self.entry(wifi);
                if e.as_ref().is_some_and(|e| e.saved.is_some()) {
                    legend.push(LegendHint::Forget);
                }
                if e.is_some_and(|e| !matches!(e.state, EntryState::Joined | EntryState::Joining)) {
                    legend.push(LegendHint::Join);
                }
            }
        }
        ListView {
            rows: view_rows,
            lit: Some(at - self.first),
            status: wifi.status_lines(),
            legend,
        }
    }
}
