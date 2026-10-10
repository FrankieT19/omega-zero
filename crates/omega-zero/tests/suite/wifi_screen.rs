use crate::common;

use std::sync::{Arc, Mutex};

use common::{app_booting_at, tmp_root_with_carts, CLOCK_IS_SET};
use omega_input::{Action, Btn, Millis};
use omega_store::{read_app_state, write_app_state, AppState};
use omega_ui::network::networks_shown;
use omega_ui::SettingsRow;
use omega_zero::app::{App, Phase};
use omega_zero::wifi::{
    Saved, Seen, Status, WifiJob, WifiJobs, WifiNews, GLANCE_MS, JOIN_TIMEOUT_MS,
};
use omega_zero::wifi_screen::{Lit, NetworkLine, NetworkView, Sel, SettingLine, Summary};
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
    fn with(state: AppState, radio: Radio) -> Rig {
        let d = tmp_root_with_carts(&["Emerald", "Fusion"]);
        write_app_state(
            d.path(),
            &AppState {
                clock_set: true,
                ..state
            },
        )
        .expect("write omega-zero.state");
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
        Rig::with(AppState::default(), Radio::default())
    }

    fn press(&mut self, btn: Btn) {
        self.a.apply(Action::GbaDown(btn));
        self.a.apply(Action::GbaUp(btn));
    }

    fn tick(&mut self, ms: Millis) {
        self.now += ms;
        self.a.tick_ms(self.now);
    }

    fn open_menu_at(&mut self, row: SettingsRow) {
        self.press(Btn::Start);
        assert!(self.a.settings_row().is_some(), "START opened no Settings");
        while self.a.settings_row() != Some(row) {
            self.press(Btn::Down);
            assert_ne!(
                self.a.settings_row(),
                Some(SettingsRow::FastForward),
                "the bar went round without reaching {row:?}"
            );
        }
    }

    fn open_wifi(&mut self) {
        self.open_menu_at(SettingsRow::Network);
        self.press(Btn::A);
        assert!(
            matches!(self.a.phase(), Phase::Network(_)),
            "A did not open Wi-Fi"
        );
    }

    /// Opens the Network screen with Wi-Fi on and these networks heard.
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
            Phase::Network(s) => s.sel.clone(),
            other => panic!("not on the Network screen: {other:?}"),
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

    fn view(&mut self) -> NetworkView {
        self.a.network_view().expect("the Network modal is showing")
    }

    fn status(&mut self) -> Vec<String> {
        self.view().notes
    }

    /// The labels of NETWORK's rows, top first.
    fn settings(&mut self) -> Vec<&'static str> {
        self.view().settings.iter().map(|s| s.label).collect()
    }

    /// The names of the networks in view, top first.
    fn names(&mut self) -> Vec<String> {
        self.view().networks.into_iter().map(|n| n.name).collect()
    }

    /// The network's name under the cursor, when it is on one.
    fn lit_name(&mut self) -> Option<String> {
        let view = self.view();
        match view.lit {
            Lit::Network(i) => Some(view.networks[i].name.clone()),
            Lit::Setting(_) => None,
        }
    }
}

/// `n` networks in reach, the first strongest, all locked.
fn many(n: usize) -> Vec<(String, i32, bool)> {
    (0..n)
        .map(|i| (format!("Net {i:02}"), -40 - i as i32, true))
        .collect()
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
    assert_eq!(r.a.settings_row(), Some(SettingsRow::Network));
    r.press(Btn::A);
    r.a.apply(Action::QuickMenu);
    assert_eq!(
        r.a.settings_row(),
        Some(SettingsRow::Network),
        "MENU did not come back"
    );
}

#[test]
fn a_handheld_without_a_radio_has_no_network_row() {
    let mut r = Rig::with(AppState::default(), Radio::absent());
    assert!(!r.a.settings_rows().contains(&SettingsRow::Network));
    r.open_menu_at(SettingsRow::DateTime);
    r.press(Btn::Down);
    assert_eq!(r.a.settings_row(), Some(SettingsRow::Bluetooth));
    r.press(Btn::Up);
    assert_eq!(r.a.settings_row(), Some(SettingsRow::DateTime));
}

#[test]
fn switching_wifi_on_brings_the_radio_up_and_is_remembered() {
    let mut r = Rig::new();
    r.open_wifi();
    r.press(Btn::A);
    assert_eq!(r.radio.asked(), [WifiJob::Up]);
    assert!(read_app_state(r.d.path()).wifi, "Wi-Fi on was not saved");
    r.press(Btn::Left);
    assert_eq!(r.radio.asked(), [WifiJob::Up, WifiJob::Down]);
    assert!(!read_app_state(r.d.path()).wifi, "Wi-Fi off was not saved");
}

#[test]
fn a_handheld_that_left_wifi_on_brings_it_up_at_boot() {
    let r = Rig::with(
        AppState {
            wifi: true,
            ..AppState::default()
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
        AppState {
            wifi: true,
            ..AppState::default()
        },
        Radio::default(),
    );
    r.radio.forget_asked();
    r.tick(5_000);
    assert_eq!(
        r.radio.asked(),
        [WifiJob::Glance],
        "the shelf asked for more than a glance at the link"
    );
    r.radio.forget_asked();
    r.open_menu_at(SettingsRow::Network);
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
        matches!(r.a.phase(), Phase::Network(_)),
        "B left Wi-Fi altogether"
    );
    r.press(Btn::A);
    r.a.apply(Action::QuickMenu);
    assert_eq!(r.typing(), None);
    assert!(
        matches!(r.a.phase(), Phase::Network(_)),
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
    assert_eq!(r.names(), ["Home", "Cabin"]);
    let cabin = r.view().networks[1].clone();
    assert_eq!(
        cabin,
        NetworkLine {
            name: "Cabin".into(),
            value: Some("SAVED"),
            signal: None,
            lock: false,
        },
        "an absent network showed a signal"
    );
    r.go_to("Cabin");
    assert!(r.a.network_forgets(), "the bar would not offer Y FORGET");
    r.press(Btn::Y);
    assert_eq!(r.radio.asked(), [WifiJob::Forget(5)]);
}

#[test]
fn network_holds_wifi_and_only_with_it_on_network_time_and_ssh() {
    let mut r = Rig::new();
    r.open_wifi();
    assert_eq!(r.settings(), ["WI-FI"]);
    assert_eq!(
        r.view().settings[0],
        SettingLine {
            label: "WI-FI",
            value: Some("OFF"),
            carets: true,
            opens: false,
        }
    );
    assert!(r.names().is_empty(), "networks listed with Wi-Fi off");
    assert_eq!(r.status(), ["Wi-Fi is off."]);
    assert_eq!(r.view().lit, Lit::Setting(0));

    let mut r = Rig::new();
    r.on_with(&[("Home", -48, true)], &[]);
    assert_eq!(r.settings(), ["WI-FI", "NETWORK TIME", "SSH"]);
    let view = r.view();
    assert_eq!(view.settings[0].value, Some("ON"));
    let ntp = if r.a.wifi().ntp() { "ON" } else { "OFF" };
    assert_eq!(view.settings[1].value, Some(ntp), "{:?}", view.settings[1]);
    assert!(view.settings[1].carets && !view.settings[1].opens);
    assert_eq!(view.settings[2].value, Some("ON"), "SSH is not on");
    assert!(view.settings[2].carets && !view.settings[2].opens);

    r.press(Btn::Down);
    r.press(Btn::Down);
    assert_eq!(r.view().lit, Lit::Setting(2), "SSH");
    r.press(Btn::Down);
    assert_eq!(r.view().lit, Lit::Network(0), "the first network");
    assert!(
        !r.a.network_forgets(),
        "Y FORGET offered on a network never saved"
    );
    r.press(Btn::Up);
    r.press(Btn::Up);
    r.press(Btn::Up);
    r.press(Btn::Left);
    assert_eq!(r.settings(), ["WI-FI"], "Wi-Fi off kept its other rows");
}

#[test]
fn the_networks_list_the_joined_one_then_by_signal_then_the_saved_out_of_reach() {
    let mut r = Rig::new();
    r.on_with(
        &[
            ("Upstairs 5G", -45, true),
            ("Café Wi-Fi", -60, false),
            ("Home", -70, true),
            ("Attic", -90, true),
        ],
        &[(1, "Home"), (2, "Cabin")],
    );
    r.radio.say(joined("Home", Some("192.168.1.20")));
    r.tick(16);
    let line = |name: &str, value, signal, lock| NetworkLine {
        name: name.into(),
        value,
        signal,
        lock,
    };
    assert_eq!(
        r.view().networks,
        [
            line("Home", Some("CONNECTED"), Some(2), true),
            line("Upstairs 5G", None, Some(3), true),
            line("Café Wi-Fi", None, Some(3), false),
            line("Attic", None, Some(1), true),
            line("Cabin", Some("SAVED"), None, false),
        ],
        "the name keeps its case, accent and all"
    );
    assert_eq!(
        r.status(),
        Vec::<String>::new(),
        "joined and addressed, a note still said something: SSH has the address"
    );
}

#[test]
fn a_list_longer_than_its_panel_scrolls_a_whole_row_at_a_time() {
    let mut r = Rig::new();
    let nets = many(30);
    let nets: Vec<(&str, i32, bool)> = nets.iter().map(|(s, d, l)| (s.as_str(), *d, *l)).collect();
    r.on_with(&nets, &[]);
    let shown = networks_shown(3, r.status().len());
    assert!(shown < 30, "the panel holds all {shown}");
    assert_eq!(r.names().len(), shown, "the panel is not full");
    assert_eq!(r.names()[0], "Net 00");

    // down to the last network in view: the list stays
    r.go_to(&format!("Net {:02}", shown - 1));
    assert_eq!(r.names()[0], "Net 00");
    assert_eq!(r.view().lit, Lit::Network(shown - 1));
    // one more: the list moves up a row, the cursor on the bottom one
    r.press(Btn::Down);
    assert_eq!(r.names()[0], "Net 01");
    assert_eq!(r.lit_name().as_deref(), Some(&*format!("Net {shown:02}")));
    assert_eq!(r.view().lit, Lit::Network(shown - 1));
    assert_eq!(r.names().len(), shown);
    // and back up: it moves only once the cursor passes its top
    for _ in 0..shown - 1 {
        r.press(Btn::Up);
    }
    assert_eq!(r.names()[0], "Net 01");
    assert_eq!(r.view().lit, Lit::Network(0));
    r.press(Btn::Up);
    assert_eq!(r.names()[0], "Net 00");
    assert_eq!(r.view().lit, Lit::Network(0));

    // up from Wi-Fi goes round to the last network, the list at its end
    r.press(Btn::Up);
    r.press(Btn::Up);
    r.press(Btn::Up);
    r.press(Btn::Up);
    assert_eq!(r.view().lit, Lit::Network(shown - 1));
    assert_eq!(r.lit_name().as_deref(), Some("Net 29"));
    assert_eq!(r.names()[0], format!("Net {:02}", 30 - shown));
    // down from there goes round to Wi-Fi, and the list stays where it was
    r.press(Btn::Down);
    assert_eq!(r.view().lit, Lit::Setting(0));
    assert_eq!(r.names()[0], format!("Net {:02}", 30 - shown));
}

#[test]
fn with_nothing_heard_yet_it_says_it_is_looking() {
    let mut r = Rig::new();
    r.on_with(&[], &[]);
    assert_eq!(r.status(), ["Looking for networks…"]);
    r.radio.say(WifiNews::Seen(vec![Seen {
        ssid: b"Home".to_vec(),
        rssi: -50,
        secured: true,
    }]));
    r.tick(16);
    assert_eq!(r.status(), ["Not connected. Pick a network to join."]);
}

#[test]
fn a_long_name_is_cut_short_in_a_note() {
    let mut r = Rig::new();
    let long = "The network with the very long name upstairs";
    r.on_with(&[(long, -60, false)], &[]);
    r.go_to(long);
    r.press(Btn::A);
    r.tick(JOIN_TIMEOUT_MS * 2);
    let notes = r.status();
    assert_eq!(notes.len(), 1);
    assert!(notes[0].starts_with("Could not join The network with") && notes[0].ends_with("…."));
    assert!(
        notes[0].chars().count() <= 43,
        "{:?} is wider than the list's panel at 640x480",
        notes[0]
    );
    assert_eq!(r.names(), [long], "the list cut the name; its row cuts it");
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
    assert_eq!(r.a.network_typing().unwrap().replace, Some(0));
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
fn joined_it_waits_for_an_address_then_leaves_it_to_ssh() {
    let mut r = Rig::new();
    r.on_with(&[("Home", -48, true)], &[(0, "Home")]);
    r.radio.say(joined("Home", None));
    r.tick(16);
    assert_eq!(r.status(), ["Waiting for an address…"]);
    assert_eq!(r.view().networks[0].value, Some("CONNECTED"));
    r.radio.say(joined("Home", Some("192.168.1.20")));
    r.tick(16);
    assert!(
        r.status().is_empty(),
        "the address is SSH's to say: {:?}",
        r.status()
    );
    assert_eq!(r.a.wifi_summary(), Summary::Joined("Home".into()));
}

#[test]
fn the_top_bar_lights_wifi_and_ssh_once_a_network_is_joined() {
    let mut r = Rig::new();
    assert!(
        !r.a.links().wifi && !r.a.links().ssh,
        "lit before Wi-Fi is on"
    );
    r.on_with(&[("Home", -48, true)], &[(0, "Home")]);
    assert!(!r.a.links().wifi, "lit with Wi-Fi on but nothing joined");
    r.radio.say(joined("Home", Some("192.168.1.20")));
    r.tick(16);
    assert!(r.a.links().wifi, "Wi-Fi is not lit once joined");
    assert!(
        r.a.links().ssh,
        "ssh is not lit once the handheld can be reached"
    );
    assert_eq!(
        r.a.links().bluetooth,
        Some(false),
        "Bluetooth is not shown off"
    );
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
    assert_eq!(r.lit_name().as_deref(), Some("Far"));
    assert_eq!(r.names(), ["Far", "Near"]);
    r.press(Btn::A);
    assert!(
        r.a.network_typing().is_some_and(|t| t.ssid == b"Far"),
        "A picked another network"
    );
}

#[test]
fn settings_says_off_on_or_the_network_joined() {
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

#[test]
fn the_shelf_glances_at_the_link_and_asks_its_status_only_when_a_glance_disagrees() {
    let mut r = Rig::with(
        AppState {
            wifi: true,
            ..AppState::default()
        },
        Radio::default(),
    );
    assert!(matches!(r.a.phase(), Phase::Shelf), "{:?}", r.a.phase());
    r.radio.say(joined("Home", Some("192.168.1.20")));
    r.radio.say(WifiNews::Saved(Vec::new()));
    r.tick(16);
    assert_eq!(r.radio.asked(), [WifiJob::Up, WifiJob::Glance]);
    r.radio.forget_asked();
    // a minute of glances that find the link as the status says
    for _ in 0..60 {
        r.radio.say(WifiNews::Glanced(true));
        r.tick(16);
        r.tick(GLANCE_MS);
    }
    let asked = r.radio.asked();
    assert_eq!(
        asked,
        vec![WifiJob::Glance; 60],
        "the shelf asked for more than a glance a second"
    );
    r.radio.forget_asked();
    r.radio.say(WifiNews::Glanced(false));
    r.tick(16);
    assert_eq!(
        r.radio.asked(),
        [WifiJob::Refresh],
        "a glance that found the link down did not ask for the status"
    );
    // a status that goes on saying joined is asked no more often than the menus ask it
    let mut refreshes = 0;
    for _ in 0..10 {
        for job in r.radio.asked() {
            match job {
                WifiJob::Glance => r.radio.say(WifiNews::Glanced(false)),
                WifiJob::Refresh => {
                    refreshes += 1;
                    r.radio.say(joined("Home", Some("192.168.1.20")));
                    r.radio.say(WifiNews::Saved(Vec::new()));
                }
                other => panic!("the shelf asked for {other:?}"),
            }
        }
        r.radio.forget_asked();
        r.tick(16);
        r.tick(GLANCE_MS);
    }
    assert!(
        (3..=6).contains(&refreshes),
        "{refreshes} statuses in ten seconds of glances that disagree"
    );
}
