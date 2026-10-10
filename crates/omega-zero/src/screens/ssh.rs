//! SSH, opened from Network: how to log in to the handheld from a computer on the same
//! network. BaseOS runs SSH and SFTP whenever a network is joined.

use std::path::Path;

use omega_input::{Action, Btn};
use omega_ui::{Draw, WifiMenuFaces};

use crate::app::{App, Phase};
use crate::list_view::{LegendHint, ListView, RowLabel, RowValue, ViewRow};
use crate::nav::Dir;
use crate::wifi::Status;
use crate::wifi_screen::{Sel as WifiSel, WifiScreen};

/// The file on the card BaseOS reads its settings from, `ssh_password` among them.
pub const BASEOS_CONF: &str = "baseos.conf";

/// root's password when baseos.conf sets none.
pub const DEFAULT_PASSWORD: &str = "root";

pub const SSH_PORT: u16 = 22;

/// The password BaseOS gives root, from the text of its baseos.conf: what `ssh_password=` says,
/// or root's own when the line is missing, commented out or left empty. Spaces around the
/// value and a pair of quotes around it are not part of it.
pub fn ssh_password(conf: &str) -> String {
    let mut password = None;
    for line in conf.lines() {
        let line = line.trim();
        if line.starts_with('#') {
            continue;
        }
        let Some((key, value)) = line.split_once('=') else {
            continue;
        };
        if key.trim() == "ssh_password" {
            password = Some(unquoted(value.trim()).to_string());
        }
    }
    password
        .filter(|p| !p.is_empty())
        .unwrap_or_else(|| DEFAULT_PASSWORD.to_string())
}

fn unquoted(value: &str) -> &str {
    for quote in ['"', '\''] {
        if let Some(inner) = value
            .strip_prefix(quote)
            .and_then(|v| v.strip_suffix(quote))
        {
            return inner;
        }
    }
    value
}

/// root's password on the card at `root`.
pub fn read_ssh_password(root: Option<&Path>) -> String {
    root.and_then(|root| std::fs::read_to_string(root.join(BASEOS_CONF)).ok())
        .map_or_else(|| DEFAULT_PASSWORD.to_string(), |conf| ssh_password(&conf))
}

/// What the SSH screen shows: each line a name and its value, then a note under them. With
/// no network joined there are no lines, only the note.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SshView {
    pub lines: Vec<(&'static str, String)>,
    pub note: &'static str,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SshScreen {
    /// root's password, read from baseos.conf when the screen opened.
    pub password: String,
}

impl SshScreen {
    /// How to log in, as the radio's `status` says the handheld can be reached.
    pub fn view(&self, status: &Status) -> SshView {
        if !status.joined() {
            return SshView {
                lines: Vec::new(),
                note: "Join a Wi-Fi network first.",
            };
        }
        let Some(ip) = &status.ip else {
            return SshView {
                lines: Vec::new(),
                note: "Waiting for an address…",
            };
        };
        let host = match status.hostname.as_str() {
            "" => ip.clone(),
            name => format!("{name}.local"),
        };
        SshView {
            lines: vec![
                ("IP", ip.clone()),
                ("Host", host),
                ("User", "root".to_string()),
                ("Password", self.password.clone()),
                ("Port", SSH_PORT.to_string()),
            ],
            note: "SFTP uses the same login.",
        }
    }
}

impl SshView {
    /// The view as a list, for the list faces the other screens are drawn with.
    pub fn list(&self) -> ListView {
        ListView {
            rows: self
                .lines
                .iter()
                .map(|(label, value)| ViewRow {
                    label: RowLabel::Setting(label),
                    value: Some(RowValue::Name(value.clone())),
                    signal: None,
                })
                .collect(),
            lit: None,
            status: vec![self.note.to_string()],
            legend: vec![LegendHint::Back],
        }
    }
}

impl App {
    pub(crate) fn open_ssh(&mut self) {
        let password = read_ssh_password(self.root.as_deref());
        self.go(Phase::Ssh(SshScreen { password }), Dir::Deeper);
    }

    pub(crate) fn ssh_input(&mut self, action: Action) {
        if matches!(action, Action::GbaDown(Btn::B) | Action::QuickMenu) {
            let back = Phase::Network(WifiScreen {
                sel: WifiSel::Ssh,
                ..WifiScreen::default()
            });
            self.go(back, Dir::Back);
        }
    }

    /// The SSH screen's lines, while it is open.
    pub fn ssh_view(&self) -> Option<SshView> {
        match &self.phase {
            Phase::Ssh(screen) => Some(screen.view(self.wifi.status())),
            _ => None,
        }
    }

    pub fn set_ssh_faces(&mut self, faces: WifiMenuFaces) {
        self.ssh_faces = Some(faces);
    }

    pub(crate) fn draw_ssh(&self, out: &mut Vec<Draw>) {
        if let Some(faces) = &self.ssh_faces {
            faces.draw_over(out);
        }
    }
}
