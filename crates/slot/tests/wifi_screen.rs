mod common;

use std::sync::{Arc, Mutex};

use common::{app_booting_at, tmp_root_with_carts, CLOCK_IS_SET};
use slot::app::{App, Phase};
use slot::list_view::RowLabel;
use slot::wifi::{Saved, Seen, Status, WifiJob, WifiJobs, WifiNews, JOIN_TIMEOUT_MS};
use slot::wifi_screen::{Sel, Summary};
use slot_input::{Action, Btn, Millis};
use slot_store::{read_slot_state, write_slot_state, SlotState};
use slot_ui::QuickRow;
use tempfile::TempDir;

#[derive(Default)]
struct RadioState {
    absent: bool,
    asked: Vec<WifiJob>,
    news: Vec<WifiNews>,
}

/// A radio that answers only what the test says, and remembers what it was asked.
#[derive(Clone, Default)]
struct Radio(Arc<Mutex<RadioState>>);

impl Radio {
    fn absent() -> Self {
        let r = Radio::default();
        r.0.lock().unwrap().absent = true;
        r
    }

    fn asked(&self) -> Vec<WifiJob> {
        self.0.lock().unwrap().asked.clone()
    }

    fn forget_asked(&self) {
        self.0.lock().unwrap().asked.clear();
    }

    fn say(&self, news: WifiNews) {
        self.0.lock().unwrap().news.push(news);
    }
}

impl WifiJobs for Radio {
    fn present(&self) -> bool {
        !self.0.lock().unwrap().absent
    }

    fn ask(&mut self, job: WifiJob) {
        self.0.lock().unwrap().asked.push(job);
    }

    fn news(&mut self) -> Vec<WifiNews> {
        std::mem::take(&mut self.0.lock().unwrap().news)
    }
}

struct Rig {
    d: TempDir,
    a: App,
    radio: Radio,
    now: Millis,
}

impl Rig {
    fn with(state: SlotState, radio: Radio) -> Rig {
        let d = tmp_root_with_carts(&["Emerald", "Fusion"]);
        write_slot_state(
            d.path(),
            &SlotState {
                clock_set: true,
                ..state
            },
        )
        .expect("write slot.state");
        let (mut a, _) = app_booting_at(d.path(), CLOCK_IS_SET);
        a.set_wifi_jobs(Box::new(radio.clone()));
        Rig {
            d,
            a,
            radio,
            now: 1_000,
        }
    }

    fn new() -> Rig {
        Rig::with(SlotState::default(), Radio::default())
    }

    fn press(&mut self, btn: Btn) {
        self.a.apply(Action::GbaDown(btn));
        self.a.apply(Action::GbaUp(btn));
    }

    fn tick(&mut self, ms: Millis) {
        self.now += ms;
        self.a.tick_ms(self.now);
    }

    fn open_menu_at(&mut self, row: QuickRow) {
        self.a.apply(Action::QuickMenu);
        assert!(self.a.quick_menu().is_some(), "MENU opened no quick menu");
        while self.a.quick_menu() != Some(row) {
            self.press(Btn::Down);
            assert_ne!(
                self.a.quick_menu(),
                Some(QuickRow::FastForward),
                "the bar went round without reaching {row:?}"
            );
        }
    }

    fn open_wifi(&mut self) {
        self.open_menu_at(QuickRow::Wifi);
        self.press(Btn::A);
        assert!(
            matches!(self.a.phase(), Phase::Wifi(_)),
            "A did not open Wi-Fi"
        );
    }

    /// Opens the Wi-Fi screen with Wi-Fi on and these networks heard.
    fn on_with(&mut self, seen: &[(&str, i32, bool)], saved: &[(u32, &str)]) {
        self.open_wifi();
        if !self.a.wifi().enabled() {
            self.press(Btn::A);
        }
        self.radio.say(WifiNews::Seen(
            seen.iter()
                .map(|(s, rssi, secured)| Seen {
                    ssid: s.as_bytes().to_vec(),
                    rssi: *rssi,
                    secured: *secured,
                })
                .collect(),
        ));
        self.radio.say(WifiNews::Saved(
            saved
                .iter()
                .map(|(id, s)| Saved {
                    id: *id,
                    ssid: s.as_bytes().to_vec(),
                    current: false,
                    refused: false,
                })
                .collect(),
        ));
        self.tick(16);
        self.radio.forget_asked();
    }

    fn sel(&self) -> Sel {
        match self.a.phase() {
            Phase::Wifi(s) => s.sel.clone(),
            other => panic!("not on the Wi-Fi screen: {other:?}"),
        }
    }

    fn go_to(&mut self, ssid: &str) {
        let want = Sel::Network(ssid.as_bytes().to_vec());
        for _ in 0..20 {
            if self.sel() == want {
                return;
            }
            self.press(Btn::Down);
        }
        panic!("{ssid} is not in the list");
    }

    fn typing(&self) -> Option<String> {
        self.a.typing().map(|t| t.keys.text.clone())
    }

    fn status(&mut self) -> Vec<String> {
        self.a.wifi_view().expect("the list is showing").status
    }
}

fn joined(ssid: &str, ip: Option<&str>) -> WifiNews {
    WifiNews::Status(Status {
        enabled: true,
        state: "COMPLETED".into(),
        ssid: Some(ssid.as_bytes().to_vec()),
        ip: ip.map(str::to_string),
        rssi: Some(-50),
        hostname: "rg35xxsp".into(),
        ntp: true,
        ntp_synced: true,
    })
}

#[test]
fn a_on_wifi_opens_its_screen_and_b_comes_back_to_the_row() {
    let mut r = Rig::new();
    r.open_wifi();
    assert_eq!(r.sel(), Sel::Wifi);
    r.press(Btn::B);
    assert_eq!(r.a.quick_menu(), Some(QuickRow::Wifi));
    r.press(Btn::A);
    r.a.apply(Action::QuickMenu);
    assert_eq!(
        r.a.quick_menu(),
        Some(QuickRow::Wifi),
        "MENU did not come back"
    );
}

#[test]
fn a_handheld_without_a_radio_has_no_wifi_row() {
    let mut r = Rig::with(SlotState::default(), Radio::absent());
    assert!(!r.a.quick_rows().contains(&QuickRow::Wifi));
    r.open_menu_at(QuickRow::Rumble);
    r.press(Btn::Down);
    assert_eq!(r.a.quick_menu(), Some(QuickRow::DateTime));
    r.press(Btn::Up);
    assert_eq!(r.a.quick_menu(), Some(QuickRow::Rumble));
}

#[test]
fn switching_wifi_on_brings_the_radio_up_and_is_remembered() {
    let mut r = Rig::new();
    r.open_wifi();
    r.press(Btn::A);
    assert_eq!(r.radio.asked(), [WifiJob::Up]);
    assert!(read_slot_state(r.d.path()).wifi, "Wi-Fi on was not saved");
    r.press(Btn::Left);
    assert_eq!(r.radio.asked(), [WifiJob::Up, WifiJob::Down]);
    assert!(!read_slot_state(r.d.path()).wifi, "Wi-Fi off was not saved");
}

#[test]
fn a_handheld_that_left_wifi_on_brings_it_up_at_boot() {
    let r = Rig::with(
        SlotState {
            wifi: true,
            ..SlotState::default()
        },
        Radio::default(),
    );
    assert_eq!(r.radio.asked(), [WifiJob::Up]);
    let off = Rig::new();
    assert_eq!(off.radio.asked(), [], "Wi-Fi off came up at boot");
}

#[test]
fn the_screen_looks_for_networks_and_the_menu_only_asks_how_it_is_joined() {
    let mut r = Rig::with(
        SlotState {
            wifi: true,
            ..SlotState::default()
        },
        Radio::default(),
    );
    r.radio.forget_asked();
    r.tick(5_000);
    assert_eq!(r.radio.asked(), [], "the shelf kept the radio busy");
    r.open_menu_at(QuickRow::Wifi);
    r.tick(16);
    assert_eq!(r.radio.asked(), [WifiJob::Refresh]);
    r.tick(5_000);
    assert_eq!(
        r.radio.asked(),
        [WifiJob::Refresh],
        "asked again before the last one was answered"
    );
    r.radio.say(WifiNews::Saved(Vec::new()));
    r.tick(16);
    r.tick(3_000);
    assert_eq!(r.radio.asked(), [WifiJob::Refresh, WifiJob::Refresh]);
    r.radio.forget_asked();
    r.press(Btn::A);
    r.tick(16);
    assert!(
        r.radio.asked().contains(&WifiJob::Scan),
        "the screen did not look"
    );
}

#[test]
fn a_new_network_with_a_lock_asks_for_its_passphrase_and_joins_with_it() {
    let mut r = Rig::new();
    r.on_with(&[("Home", -48, true)], &[]);
    r.go_to("Home");
    r.press(Btn::A);
    assert_eq!(r.typing().as_deref(), Some(""), "no keyboard");
    for _ in 0..8 {
        r.press(Btn::A);
    }
    r.press(Btn::Start);
    assert_eq!(r.typing(), None, "the keyboard stayed up");
    assert_eq!(
        r.radio.asked(),
        [WifiJob::Join {
            ssid: b"Home".to_vec(),
            psk: "qqqqqqqq".into(),
            replace: None
        }]
    );
    assert_eq!(r.status(), ["Joining Home…"]);
    assert_eq!(r.sel(), Sel::Network(b"Home".to_vec()));
}

#[test]
fn a_passphrase_too_short_is_refused_without_asking_the_radio() {
    let mut r = Rig::new();
    r.on_with(&[("Home", -48, true)], &[]);
    r.go_to("Home");
    r.press(Btn::A);
    for _ in 0..3 {
        r.press(Btn::A);
    }
    r.press(Btn::Start);
    assert_eq!(r.typing().as_deref(), Some("qqq"));
    assert!(r.a.typing().unwrap().refused, "the field did not say no");
    assert_eq!(r.radio.asked(), []);
    r.press(Btn::Y);
    assert!(
        !r.a.typing().unwrap().refused,
        "typing on left the field refusing"
    );
    assert_eq!(r.typing().as_deref(), Some("qqq "));
}

#[test]
fn the_keyboard_types_capitals_and_symbols_and_deletes() {
    let mut r = Rig::new();
    r.on_with(&[("Home", -48, true)], &[]);
    r.go_to("Home");
    r.press(Btn::A);
    r.press(Btn::X);
    r.press(Btn::A);
    r.press(Btn::X);
    r.press(Btn::A);
    r.press(Btn::R1);
    r.press(Btn::A);
    r.press(Btn::Up);
    r.press(Btn::A);
    assert_eq!(r.typing().as_deref(), Some("Qq`!"));
    r.press(Btn::B);
    assert_eq!(r.typing().as_deref(), Some("Qq`"));
}

#[test]
fn b_on_an_empty_field_and_menu_go_back_to_the_list() {
    let mut r = Rig::new();
    r.on_with(&[("Home", -48, true)], &[]);
    r.go_to("Home");
    r.press(Btn::A);
    r.press(Btn::A);
    r.press(Btn::B);
    assert_eq!(
        r.typing().as_deref(),
        Some(""),
        "B with text left the keyboard"
    );
    r.press(Btn::B);
    assert_eq!(r.typing(), None);
    assert!(
        matches!(r.a.phase(), Phase::Wifi(_)),
        "B left Wi-Fi altogether"
    );
    r.press(Btn::A);
    r.a.apply(Action::QuickMenu);
    assert_eq!(r.typing(), None);
    assert!(
        matches!(r.a.phase(), Phase::Wifi(_)),
        "MENU left Wi-Fi altogether"
    );
    assert_eq!(r.radio.asked(), []);
}

#[test]
fn an_open_network_is_joined_without_a_passphrase() {
    let mut r = Rig::new();
    r.on_with(&[("Café", -60, false)], &[]);
    r.go_to("Café");
    r.press(Btn::A);
    assert_eq!(r.typing(), None);
    assert_eq!(
        r.radio.asked(),
        [WifiJob::Join {
            ssid: "Café".as_bytes().to_vec(),
            psk: String::new(),
            replace: None
        }]
    );
}

#[test]
fn a_saved_network_is_joined_by_its_id_and_y_forgets_it() {
    let mut r = Rig::new();
    r.on_with(&[("Home", -48, true)], &[(3, "Home")]);
    r.go_to("Home");
    r.press(Btn::A);
    assert_eq!(r.typing(), None, "a saved network asked for its passphrase");
    assert_eq!(r.radio.asked(), [WifiJob::Select(3)]);
    r.press(Btn::Y);
    assert_eq!(r.radio.asked(), [WifiJob::Select(3), WifiJob::Forget(3)]);
}

#[test]
fn a_saved_network_out_of_reach_is_still_listed_to_be_forgotten() {
    let mut r = Rig::new();
    r.on_with(&[("Home", -48, true)], &[(5, "Cabin")]);
    let view = r.a.wifi_view().unwrap();
    let labels: Vec<_> = view.rows.iter().map(|row| row.label.clone()).collect();
    assert_eq!(
        labels,
        [
            RowLabel::Setting("Wi-Fi"),
            RowLabel::Setting("Network Time"),
            RowLabel::Setting("Cart Labels"),
            RowLabel::Name("Home".into()),
            RowLabel::Name("Cabin".into()),
        ]
    );
    let cabin = view
        .rows
        .iter()
        .find(|row| row.label == RowLabel::Name("Cabin".into()))
        .unwrap();
    assert_eq!(cabin.signal, None, "an absent network showed a signal");
    r.go_to("Cabin");
    r.press(Btn::Y);
    assert_eq!(r.radio.asked(), [WifiJob::Forget(5)]);
}

#[test]
fn a_passphrase_turned_away_says_so_and_is_asked_for_again() {
    let mut r = Rig::new();
    r.on_with(&[("Home", -48, true)], &[]);
    r.go_to("Home");
    r.press(Btn::A);
    for _ in 0..8 {
        r.press(Btn::A);
    }
    r.press(Btn::Start);
    r.radio.say(WifiNews::Saved(vec![Saved {
        id: 0,
        ssid: b"Home".to_vec(),
        current: false,
        refused: true,
    }]));
    r.tick(16);
    assert_eq!(r.status(), ["Could not join Home.", "Wrong passphrase?"]);
    r.radio.forget_asked();
    r.press(Btn::A);
    assert_eq!(
        r.typing().as_deref(),
        Some(""),
        "a turned-away network was retried as it was"
    );
    assert_eq!(r.a.wifi_typing().unwrap().replace, Some(0));
    assert_eq!(r.radio.asked(), []);
}

#[test]
fn a_join_that_never_finishes_gives_up() {
    let mut r = Rig::new();
    r.on_with(&[("Café", -60, false)], &[]);
    r.go_to("Café");
    r.press(Btn::A);
    r.tick(JOIN_TIMEOUT_MS / 2);
    assert_eq!(r.status(), ["Joining Café…"]);
    r.tick(JOIN_TIMEOUT_MS);
    assert_eq!(r.status(), ["Could not join Café."]);
}

#[test]
fn joined_it_says_where_to_reach_it_over_ssh() {
    let mut r = Rig::new();
    r.on_with(&[("Home", -48, true)], &[(0, "Home")]);
    r.radio.say(joined("Home", None));
    r.tick(16);
    assert_eq!(r.status(), ["Joined Home.", "Waiting for an address…"]);
    r.radio.say(joined("Home", Some("192.168.1.20")));
    r.tick(16);
    assert_eq!(
        r.status(),
        [
            "Connected to Home as 192.168.1.20",
            "SSH and SFTP: root@rg35xxsp.local"
        ]
    );
    assert_eq!(r.a.wifi_summary(), Summary::Joined("Home".into()));
}

#[test]
fn the_cursor_stays_on_its_network_when_a_scan_reorders_the_list() {
    let mut r = Rig::new();
    r.on_with(&[("Near", -40, true), ("Far", -80, true)], &[]);
    r.go_to("Far");
    r.radio.say(WifiNews::Seen(vec![
        Seen {
            ssid: b"Far".to_vec(),
            rssi: -45,
            secured: true,
        },
        Seen {
            ssid: b"Near".to_vec(),
            rssi: -70,
            secured: true,
        },
    ]));
    r.tick(16);
    let view = r.a.wifi_view().unwrap();
    let lit = view.lit.unwrap();
    assert_eq!(view.rows[lit].label, RowLabel::Name("Far".into()));
    r.press(Btn::A);
    assert!(
        r.a.wifi_typing().is_some_and(|t| t.ssid == b"Far"),
        "A picked another network"
    );
}

#[test]
fn the_quick_menu_says_off_on_or_the_network_joined() {
    let mut r = Rig::new();
    assert_eq!(r.a.wifi_summary(), Summary::Off);
    r.on_with(&[], &[]);
    assert_eq!(r.a.wifi_summary(), Summary::On);
    r.radio.say(joined("Home", Some("10.0.0.2")));
    r.tick(16);
    assert_eq!(r.a.wifi_summary(), Summary::Joined("Home".into()));
}

#[test]
fn network_time_switches_on_the_clock_row() {
    let mut r = Rig::new();
    r.on_with(&[], &[]);
    r.press(Btn::Down);
    assert_eq!(r.sel(), Sel::Clock);
    let was = r.a.wifi().ntp();
    r.press(Btn::Right);
    assert_eq!(r.radio.asked(), [WifiJob::Clock(!was)]);
    assert_eq!(r.a.wifi().ntp(), !was);
}

#[test]
fn a_radio_that_never_appears_says_so() {
    let mut r = Rig::new();
    r.open_wifi();
    r.press(Btn::A);
    r.radio.say(WifiNews::Started(false));
    r.tick(16);
    assert_eq!(r.status(), ["The Wi-Fi radio did not start."]);
}
