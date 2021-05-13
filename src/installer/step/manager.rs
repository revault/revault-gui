use bitcoin::util::bip32::ExtendedPubKey;
use iced::{button::State as Button, scrollable, text_input, Element};
use std::str::FromStr;

use crate::installer::{
    message::{self, Message},
    step::{
        common::{CosignerKey, ParticipantXpub},
        Step,
    },
    view,
};

pub struct DefineStakeholderXpubs {
    stakeholder_xpubs: Vec<ParticipantXpub>,
    add_xpub_button: Button,
    scroll: scrollable::State,
    previous_button: Button,
    save_button: Button,
}

impl DefineStakeholderXpubs {
    pub fn new() -> Self {
        Self {
            add_xpub_button: Button::new(),
            stakeholder_xpubs: Vec::new(),
            scroll: scrollable::State::new(),
            previous_button: Button::new(),
            save_button: Button::new(),
        }
    }
}

impl Step for DefineStakeholderXpubs {
    fn is_correct(&self) -> bool {
        !self.stakeholder_xpubs.iter().any(|xpub| xpub.warning)
    }

    fn check(&mut self) {
        for participant in &mut self.stakeholder_xpubs {
            if ExtendedPubKey::from_str(&participant.xpub).is_err() {
                participant.warning = true;
            }
        }
    }

    fn update(&mut self, message: Message) {
        if let Message::DefineStakeholderXpubs(msg) = message {
            match msg {
                message::DefineStakeholderXpubs::StakeholderXpub(
                    i,
                    message::ParticipantXpub::Delete,
                ) => {
                    self.stakeholder_xpubs.remove(i);
                }
                message::DefineStakeholderXpubs::StakeholderXpub(i, msg) => {
                    if let Some(xpub) = self.stakeholder_xpubs.get_mut(i) {
                        xpub.update(msg);
                    }
                }
                message::DefineStakeholderXpubs::AddXpub => {
                    self.stakeholder_xpubs.push(ParticipantXpub::new());
                }
                _ => (),
            };
        };
    }

    fn view(&mut self) -> Element<Message> {
        return view::define_stakeholder_xpubs_as_manager_only(
            &mut self.add_xpub_button,
            self.stakeholder_xpubs
                .iter_mut()
                .enumerate()
                .map(|(i, xpub)| {
                    xpub.view().map(move |msg| {
                        Message::DefineStakeholderXpubs(
                            message::DefineStakeholderXpubs::StakeholderXpub(i, msg),
                        )
                    })
                })
                .collect(),
            &mut self.scroll,
            &mut self.previous_button,
            &mut self.save_button,
        );
    }
}

impl From<DefineStakeholderXpubs> for Box<dyn Step> {
    fn from(s: DefineStakeholderXpubs) -> Box<dyn Step> {
        Box::new(s)
    }
}

pub struct DefineManagerXpubs {
    cosigners: Vec<CosignerKey>,
    other_xpubs: Vec<ParticipantXpub>,
    our_xpub: String,
    our_xpub_warning: bool,

    add_xpub_button: Button,
    add_cosigner_button: Button,
    our_xpub_input: text_input::State,
    scroll: scrollable::State,
    previous_button: Button,
    save_button: Button,
}

impl DefineManagerXpubs {
    pub fn new() -> Self {
        Self {
            our_xpub: "".to_string(),
            our_xpub_input: text_input::State::new(),
            our_xpub_warning: false,
            other_xpubs: Vec::new(),
            add_xpub_button: Button::new(),
            scroll: scrollable::State::new(),
            previous_button: Button::new(),
            save_button: Button::new(),
            cosigners: Vec::new(),
            add_cosigner_button: Button::new(),
        }
    }
}

impl Step for DefineManagerXpubs {
    fn check(&mut self) {
        for participant in &mut self.other_xpubs {
            if ExtendedPubKey::from_str(&participant.xpub).is_err() {
                participant.warning = true;
            }
        }
        if ExtendedPubKey::from_str(&self.our_xpub).is_err() {
            self.our_xpub_warning = true;
        }
    }

    fn is_correct(&self) -> bool {
        !self.our_xpub_warning && !self.other_xpubs.iter().any(|xpub| xpub.warning)
    }

    fn update(&mut self, message: Message) {
        if let Message::DefineManagerXpubs(msg) = message {
            match msg {
                message::DefineManagerXpubs::OurXpubEdited(xpub) => {
                    self.our_xpub = xpub;
                    self.our_xpub_warning = false;
                }
                message::DefineManagerXpubs::ManagerXpub(i, message::ParticipantXpub::Delete) => {
                    self.other_xpubs.remove(i);
                }
                message::DefineManagerXpubs::ManagerXpub(i, msg) => {
                    if let Some(xpub) = self.other_xpubs.get_mut(i) {
                        xpub.update(msg)
                    };
                }
                message::DefineManagerXpubs::AddXpub => {
                    self.other_xpubs.push(ParticipantXpub::new());
                }
                message::DefineManagerXpubs::CosignerKey(i, message::CosignerKey::Delete) => {
                    self.cosigners.remove(i);
                }
                message::DefineManagerXpubs::CosignerKey(i, msg) => {
                    if let Some(key) = self.cosigners.get_mut(i) {
                        key.update(msg)
                    };
                }
                message::DefineManagerXpubs::AddCosigner => {
                    self.cosigners.push(CosignerKey::new());
                }
            };
        };
    }

    fn view(&mut self) -> Element<Message> {
        return view::define_manager_xpubs_as_manager(
            &self.our_xpub,
            &mut self.our_xpub_input,
            self.our_xpub_warning,
            &mut self.add_xpub_button,
            self.other_xpubs
                .iter_mut()
                .enumerate()
                .map(|(i, xpub)| {
                    xpub.view().map(move |msg| {
                        Message::DefineManagerXpubs(message::DefineManagerXpubs::ManagerXpub(
                            i, msg,
                        ))
                    })
                })
                .collect(),
            &mut self.add_cosigner_button,
            self.cosigners
                .iter_mut()
                .enumerate()
                .map(|(i, xpub)| {
                    xpub.view().map(move |msg| {
                        Message::DefineManagerXpubs(message::DefineManagerXpubs::CosignerKey(
                            i, msg,
                        ))
                    })
                })
                .collect(),
            &mut self.scroll,
            &mut self.previous_button,
            &mut self.save_button,
        );
    }
}

impl From<DefineManagerXpubs> for Box<dyn Step> {
    fn from(s: DefineManagerXpubs) -> Box<dyn Step> {
        Box::new(s)
    }
}
