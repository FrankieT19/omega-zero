//! SSH, opened from Network's SSH row on INFO: how to log in to the handheld from a computer on
//! the same network. The row switches the server on and off; SFTP comes with it.

use omega_input::{Action, Btn};
use omega_ui::network::SshFaces;
use omega_ui::Draw;

use crate::app::{App, Phase};
use crate::nav::Dir;
use crate::ssh::{read_ssh_password, Ssh, SshServer, Trouble, SSH_PORT};
use crate::wifi::Status;
use crate::wifi_screen::{Sel as WifiSel, WifiScreen};

/// What the SSH screen shows: each line a name and its value, then a note under them. With SSH
/// off or no network joined there are no lines, only the note.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SshView {
    pub lines: Vec<(&'static str, String)>,
    pub note: &'static str,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SshScreen {
    /// root's password, read from baseos.conf when the screen opened.
    pub password: String,
    /// SSH was on when the screen opened.
    pub on: bool,
    /// What went wrong giving root a password of its own, when something did.
    pub trouble: Option<Trouble>,
}

impl SshScreen {
    /// How to log in, as the radio's `status` says the handheld can be reached.
    pub fn view(&self, status: &Status) -> SshView {
        if !self.on {
            return SshView {
                lines: Vec::new(),
                note: "Turn SSH on first.",
            };
        }
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
                ("HOST", host),
                ("USER", "root".to_string()),
                ("PASSWORD", self.password.clone()),
                ("PORT", SSH_PORT.to_string()),
            ],
            note: self
                .trouble
                .map_or("SFTP uses the same login", Trouble::note),
        }
    }
}

impl App {
    pub(crate) fn open_ssh(&mut self) {
        let screen = SshScreen {
            password: read_ssh_password(self.root.as_deref()),
            on: self.state.ssh,
            trouble: self.ssh.trouble(),
        };
        self.go(Phase::Ssh(screen), Dir::Deeper);
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

    /// SSH is switched on.
    pub fn ssh_on(&self) -> bool {
        self.state.ssh
    }

    /// Switches SSH on or off: the server starts or stops, and the card remembers.
    pub(crate) fn set_ssh(&mut self, on: bool) {
        self.ssh.switch(on);
        if self.state.ssh != on {
            self.state.ssh = on;
            self.persist();
        }
    }

    /// Puts `server` in place of the system's, and does with it what omega-zero does as it
    /// starts.
    pub fn set_ssh_server(&mut self, server: Box<dyn SshServer>) {
        self.ssh = Ssh::new(server);
        self.ssh.start(self.state.ssh, self.root.as_deref());
    }

    /// The SSH screen's lines, while it is open.
    pub fn ssh_view(&self) -> Option<SshView> {
        match &self.phase {
            Phase::Ssh(screen) => Some(screen.view(self.wifi.status())),
            _ => None,
        }
    }

    pub fn set_ssh_faces(&mut self, faces: SshFaces) {
        self.ssh_faces = Some(faces);
    }

    pub(crate) fn draw_ssh(&self, out: &mut Vec<Draw>) {
        if let Some(faces) = &self.ssh_faces {
            faces.draw(out);
        }
    }
}
