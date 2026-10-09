//! What a settings list in the quick menu's style shows, for its faces to be made from: the
//! Wi-Fi screen and the Labels screen are both drawn from one.

use slot_ui::Keyboard;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum LegendHint {
    Back,
    Change,
    Join,
    Forget,
    SignIn,
    SignOut,
    Start,
    Stop,
    Open,
}

impl LegendHint {
    pub const ALL: [LegendHint; 9] = [
        LegendHint::Back,
        LegendHint::Change,
        LegendHint::Join,
        LegendHint::Forget,
        LegendHint::SignIn,
        LegendHint::SignOut,
        LegendHint::Start,
        LegendHint::Stop,
        LegendHint::Open,
    ];

    pub fn index(self) -> usize {
        self as usize
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum RowLabel {
    Setting(&'static str),
    /// A name that is not slot's to set in capitals, such as a network's.
    Name(String),
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum RowValue {
    /// On or Off, changed with left and right.
    Setting(bool),
    Said(&'static str),
    /// A name kept in its own case, such as an account's.
    Name(String),
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ViewRow {
    pub label: RowLabel,
    pub value: Option<RowValue>,
    /// Bars out of four, and whether it needs a passphrase.
    pub signal: Option<(u8, bool)>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ListView {
    pub rows: Vec<ViewRow>,
    pub lit: Option<usize>,
    pub status: Vec<String>,
    pub legend: Vec<LegendHint>,
}

/// The on-screen keyboard while a screen has it up, and what it is asking for.
pub struct TypingView<'a> {
    pub prompt: String,
    pub keys: &'a Keyboard,
    /// What was typed was refused.
    pub refused: bool,
}
