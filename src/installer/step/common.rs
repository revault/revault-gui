use crate::installer::{message, view};
use iced::{button::State as Button, text_input, Element};

#[derive(Clone)]
pub struct ParticipantXpub {
    pub xpub: String,
    pub warning: bool,

    xpub_input: text_input::State,
    delete_button: Button,
}

impl ParticipantXpub {
    pub fn new() -> Self {
        Self {
            xpub: "".to_string(),
            xpub_input: text_input::State::new(),
            delete_button: Button::new(),
            warning: false,
        }
    }

    pub fn update(&mut self, msg: message::ParticipantXpub) {
        if let message::ParticipantXpub::XpubEdited(xpub) = msg {
            self.xpub = xpub;
            self.warning = false;
        }
    }

    pub fn view(&mut self) -> Element<message::ParticipantXpub> {
        view::participant_xpub(
            &self.xpub,
            &mut self.xpub_input,
            &mut self.delete_button,
            self.warning,
        )
    }
}

pub struct CosignerKey {
    pub key: String,
    pub warning: bool,

    key_input: text_input::State,
    delete_button: Button,
}

impl CosignerKey {
    pub fn new() -> Self {
        Self {
            key: "".to_string(),
            key_input: text_input::State::new(),
            delete_button: Button::new(),
            warning: false,
        }
    }

    pub fn update(&mut self, msg: message::CosignerKey) {
        if let message::CosignerKey::KeyEdited(key) = msg {
            self.key = key;
            self.warning = false;
        }
    }

    pub fn view(&mut self) -> Element<message::CosignerKey> {
        view::cosigner_key(
            &self.key,
            &mut self.key_input,
            &mut self.delete_button,
            self.warning,
        )
    }
}
