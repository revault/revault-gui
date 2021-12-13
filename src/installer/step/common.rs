use revault_ui::component::form;

use crate::installer::{message, view};

use iced::{button::State as Button, text_input, Element};

use revaultd::revault_tx::miniscript::DescriptorPublicKey;
use std::str::FromStr;

#[derive(Clone)]
pub struct ParticipantXpub {
    pub xpub: form::Value<String>,

    xpub_input: text_input::State,
    delete_button: Button,
}

impl ParticipantXpub {
    pub fn new() -> Self {
        Self {
            xpub: form::Value::default(),
            xpub_input: text_input::State::new(),
            delete_button: Button::new(),
        }
    }

    pub fn update(&mut self, msg: message::ParticipantXpub) {
        if let message::ParticipantXpub::XpubEdited(xpub) = msg {
            self.xpub.value = xpub;
            self.xpub.valid = true;
        }
    }

    pub fn check_validity(&mut self, network: &bitcoin::Network) {
        if let Ok(DescriptorPublicKey::XPub(xpub)) = DescriptorPublicKey::from_str(&self.xpub.value)
        {
            if *network == bitcoin::Network::Bitcoin {
                self.xpub.valid = xpub.xkey.network == bitcoin::Network::Bitcoin;
            } else {
                self.xpub.valid = xpub.xkey.network == bitcoin::Network::Testnet;
            }
        } else {
            self.xpub.valid = false;
        }
    }

    pub fn view(&mut self) -> Element<message::ParticipantXpub> {
        view::participant_xpub(&self.xpub, &mut self.xpub_input, &mut self.delete_button)
    }
}

#[derive(Clone)]
pub struct RequiredXpub {
    pub xpub: form::Value<String>,

    xpub_input: text_input::State,
}

impl RequiredXpub {
    pub fn new() -> Self {
        Self {
            xpub: form::Value::default(),
            xpub_input: text_input::State::new(),
        }
    }

    pub fn update(&mut self, msg: String) {
        self.xpub.value = msg;
        self.xpub.valid = true;
    }

    pub fn check_validity(&mut self, network: &bitcoin::Network) {
        if let Ok(DescriptorPublicKey::XPub(xpub)) = DescriptorPublicKey::from_str(&self.xpub.value)
        {
            if *network == bitcoin::Network::Bitcoin {
                self.xpub.valid = xpub.xkey.network == bitcoin::Network::Bitcoin;
            } else {
                self.xpub.valid = xpub.xkey.network == bitcoin::Network::Testnet;
            }
        } else {
            self.xpub.valid = false;
        }
    }

    pub fn view(&mut self) -> Element<String> {
        view::required_xpub(&self.xpub, &mut self.xpub_input)
    }
}

pub struct CosignerKey {
    pub key: form::Value<String>,

    key_input: text_input::State,
}

impl CosignerKey {
    pub fn new() -> Self {
        Self {
            key: form::Value::default(),
            key_input: text_input::State::new(),
        }
    }

    pub fn update(&mut self, key: String) {
        self.key.value = key;
        self.key.valid = true;
    }

    pub fn view(&mut self) -> Element<String> {
        view::cosigner_key(&self.key, &mut self.key_input)
    }
}
