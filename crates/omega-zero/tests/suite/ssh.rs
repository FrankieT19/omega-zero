use crate::common;

use std::sync::{Arc, Mutex};

use common::{app_booting_at, tmp_root_with_carts, CLOCK_IS_SET};
use omega_input::{Action, Btn, Millis};
use omega_store::{write_app_state, AppState};
use omega_ui::SettingsRow;
use omega_zero::app::{App, Phase};
use omega_zero::screens::ssh::{ssh_password, SshScreen, SshView};
use omega_zero::wifi::{Status, WifiJob, WifiJobs, WifiNews};
use omega_zero::wifi_screen::{Sel, Watch};
use tempfile::TempDir;

/// A radio that is on, with whatever network the test says joined.
#[derive(Clone, Default)]
struct Radio(Arc<Mutex<Vec<WifiNews>>>);

impl WifiJobs for Radio {
    fn present(&self) -> bool {
        true
    }

    fn ask(&mut self, job: WifiJob) {
        if job == WifiJob::Up {
            self.0.lock().unwrap().push(WifiNews::Started(true));
        }
    }

    fn news(&mut self) -> Vec<WifiNews> {
        std::mem::take(&mut self.0.lock().unwrap())
    }
}

struct Rig {
    d: TempDir,
    a: App,
    radio: Radio,
    now: Millis,
}

impl Rig {
    fn new() -> Rig {
        let d = tmp_root_with_carts(&["Emerald", "Fusion"]);
        write_app_state(
            d.path(),
            &AppState {
                clock_set: true,
                wifi: true,
                ..AppState::default()
            },
        )
        .unwrap();
        let (mut a, _) = app_booting_at(d.path(), CLOCK_IS_SET);
        let radio = Radio::default();
        a.set_wifi_jobs(Box::new(radio.clone()));
        let mut r = Rig {
            d,
            a,
            radio,
            now: 1_000,
        };
        r.tick(16);
        r
    }

    fn press(&mut self, btn: Btn) {
        self.a.apply(Action::GbaDown(btn));
        self.a.apply(Action::GbaUp(btn));
    }

    fn tick(&mut self, ms: Millis) {
        self.now += ms;
        self.a.tick_ms(self.now);
    }

    fn join(&mut self, hostname: &str) {
        self.radio
            .0
            .lock()
            .unwrap()
            .push(WifiNews::Status(status(hostname)));
        self.tick(16);
        assert!(self.a.wifi().status().joined(), "Wi-Fi did not join");
    }

    fn open_network(&mut self) {
        self.press(Btn::Start);
        assert!(self.a.settings_row().is_some(), "START opened no Settings");
        while self.a.settings_row() != Some(SettingsRow::Network) {
            self.press(Btn::Down);
            assert_ne!(
                self.a.settings_row(),
                Some(SettingsRow::FastForward),
                "the bar went round without reaching Network"
            );
        }
        self.press(Btn::A);
        assert!(matches!(self.a.phase(), Phase::Network(_)), "no Network");
    }

    fn open_ssh(&mut self) {
        self.open_network();
        for _ in 0..4 {
            if matches!(self.a.phase(), Phase::Network(s) if s.sel == Sel::Ssh) {
                break;
            }
            self.press(Btn::Down);
        }
        self.press(Btn::A);
        assert!(
            matches!(self.a.phase(), Phase::Ssh(_)),
            "A on SSH did not open it: {:?}",
            self.a.phase()
        );
    }
}

fn status(hostname: &str) -> Status {
    Status {
        enabled: true,
        state: "COMPLETED".into(),
        ssid: Some(b"Home".to_vec()),
        ip: Some("192.168.1.20".into()),
        rssi: Some(-50),
        hostname: hostname.into(),
        ntp: true,
        ntp_synced: true,
    }
}

#[test]
fn ssh_opens_from_network_and_b_comes_back_to_its_row() {
    let mut r = Rig::new();
    r.open_ssh();
    r.press(Btn::B);
    assert!(
        matches!(r.a.phase(), Phase::Network(s) if s.sel == Sel::Ssh),
        "B did not go back to the SSH row: {:?}",
        r.a.phase()
    );
    r.press(Btn::B);
    assert_eq!(r.a.settings_row(), Some(SettingsRow::Network));
}

#[test]
fn joined_it_says_how_to_log_in() {
    let mut r = Rig::new();
    r.join("rg35xxsp");
    r.open_ssh();
    let view = r.a.ssh_view().expect("the SSH screen is not showing");
    assert_eq!(
        view,
        SshView {
            lines: vec![
                ("IP", "192.168.1.20".to_string()),
                ("Host", "rg35xxsp.local".to_string()),
                ("User", "root".to_string()),
                ("Password", "root".to_string()),
                ("Port", "22".to_string()),
            ],
            note: "SFTP uses the same login.",
        }
    );
}

#[test]
fn offline_it_asks_for_a_network_first() {
    let mut r = Rig::new();
    r.open_ssh();
    let view = r.a.ssh_view().expect("the SSH screen is not showing");
    assert!(view.lines.is_empty(), "{view:?}");
    assert_eq!(view.note, "Join a Wi-Fi network first.");
    assert_eq!(view.list().status, ["Join a Wi-Fi network first."]);
}

#[test]
fn a_handheld_with_no_name_is_reached_by_its_address() {
    let screen = SshScreen {
        password: "root".into(),
    };
    let view = screen.view(&status(""));
    assert_eq!(view.lines[1], ("Host", "192.168.1.20".to_string()));
}

#[test]
fn the_password_is_read_from_the_cards_baseos_conf_when_the_screen_opens() {
    let mut r = Rig::new();
    std::fs::write(
        r.d.path().join("baseos.conf"),
        "# baseos.conf\nhostname=rg35xxsp\nssh_password=hunter2\n",
    )
    .unwrap();
    r.join("rg35xxsp");
    r.open_ssh();
    let Phase::Ssh(screen) = r.a.phase() else {
        panic!("not on the SSH screen");
    };
    assert_eq!(screen.password, "hunter2");
    let view = r.a.ssh_view().unwrap();
    assert_eq!(view.lines[3], ("Password", "hunter2".to_string()));
}

#[test]
fn ssh_password_is_roots_own_unless_baseos_conf_sets_one() {
    assert_eq!(ssh_password(""), "root", "absent");
    assert_eq!(ssh_password("hostname=rg35xxsp\n"), "root", "absent");
    assert_eq!(
        ssh_password("# ssh_password=secret\n"),
        "root",
        "commented out"
    );
    assert_eq!(
        ssh_password("  #ssh_password=secret\n"),
        "root",
        "commented out after spaces"
    );
    assert_eq!(ssh_password("ssh_password=secret\n"), "secret", "set");
    assert_eq!(
        ssh_password("ssh_password = secret  \n"),
        "secret",
        "set with spaces"
    );
    assert_eq!(
        ssh_password("ssh_password=\"two words\"\n"),
        "two words",
        "quoted"
    );
    assert_eq!(
        ssh_password("ssh_password='secret'\n"),
        "secret",
        "single quoted"
    );
    assert_eq!(ssh_password("ssh_password=\n"), "root", "empty");
    assert_eq!(ssh_password("ssh_password=\"\"\n"), "root", "empty quotes");
    assert_eq!(
        ssh_password("# ssh_password=old\nssh_password=new\n"),
        "new",
        "the line that is not commented out"
    );
}

#[test]
fn how_closely_the_radio_is_watched_follows_the_screen() {
    let mut r = Rig::new();
    assert_eq!(r.a.wifi_watch(), Watch::Glance, "on the shelf");
    r.open_network();
    assert_eq!(r.a.wifi_watch(), Watch::Networks, "on Network");
    r.press(Btn::B);
    assert_eq!(r.a.wifi_watch(), Watch::Status, "on Settings");
    r.press(Btn::B);
    r.open_ssh();
    assert_eq!(r.a.wifi_watch(), Watch::Status, "on SSH");
    r.press(Btn::B);
    r.press(Btn::B);
    while r.a.settings_row() != Some(SettingsRow::Labels) {
        r.press(Btn::Up);
    }
    r.press(Btn::A);
    assert!(matches!(r.a.phase(), Phase::Labels(_)));
    assert_eq!(r.a.wifi_watch(), Watch::Glance, "on Labels");
    r.press(Btn::B);
    while r.a.settings_row() != Some(SettingsRow::Background) {
        r.press(Btn::Down);
    }
    r.press(Btn::A);
    assert!(matches!(r.a.phase(), Phase::Background(_)));
    assert_eq!(r.a.wifi_watch(), Watch::Idle, "on Background");
    r.press(Btn::B);
    r.press(Btn::B);
    assert!(matches!(r.a.phase(), Phase::Shelf));
    r.a.apply(Action::SelectTap);
    assert!(matches!(r.a.phase(), Phase::Controls));
    assert_eq!(r.a.wifi_watch(), Watch::Idle, "on Controls");
}
