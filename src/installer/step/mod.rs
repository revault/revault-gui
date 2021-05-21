mod common;
pub mod manager;
pub mod stakeholder;

use bitcoin::util::bip32::ExtendedPubKey;
use iced::{button::State as Button, scrollable, Element};
use std::str::FromStr;

use crate::installer::{
    message::{self, Message},
    step::common::ParticipantXpub,
    view,
};

pub trait Step {
    fn check(&mut self) {}
    fn update(&mut self, message: Message);
    fn view(&mut self) -> Element<Message>;
    fn is_correct(&self) -> bool {
        true
    }
}

pub struct Welcome {
    install_button: Button,
}

impl Welcome {
    pub fn new() -> Self {
        Self {
            install_button: Button::new(),
        }
    }
}

impl Step for Welcome {
    fn update(&mut self, _message: Message) {}
    fn view(&mut self) -> Element<Message> {
        view::welcome(&mut self.install_button)
    }
}

impl From<Welcome> for Box<dyn Step> {
    fn from(s: Welcome) -> Box<dyn Step> {
        Box::new(s)
    }
}

pub struct DefineRole {
    stakeholder_button: Button,
    manager_button: Button,
    stakeholder_manager_button: Button,
    scroll: scrollable::State,
}

impl DefineRole {
    pub fn new() -> Self {
        Self {
            stakeholder_button: Button::new(),
            manager_button: Button::new(),
            stakeholder_manager_button: Button::new(),
            scroll: scrollable::State::new(),
        }
    }
}

impl Step for DefineRole {
    fn update(&mut self, _message: Message) {}
    fn view(&mut self) -> Element<Message> {
        view::define_role(
            &mut self.stakeholder_button,
            &mut self.manager_button,
            &mut self.stakeholder_manager_button,
            &mut self.scroll,
        )
    }
}

impl From<DefineRole> for Box<dyn Step> {
    fn from(s: DefineRole) -> Box<dyn Step> {
        Box::new(s)
    }
}

pub struct DefineCpfpDescriptor {
    manager_xpubs: Vec<ParticipantXpub>,
    add_xpub_button: Button,
    scroll: scrollable::State,
    previous_button: Button,
    save_button: Button,
}

impl DefineCpfpDescriptor {
    pub fn new() -> Self {
        Self {
            add_xpub_button: Button::new(),
            manager_xpubs: Vec::new(),
            scroll: scrollable::State::new(),
            previous_button: Button::new(),
            save_button: Button::new(),
        }
    }
}

impl Step for DefineCpfpDescriptor {
    fn is_correct(&self) -> bool {
        !self.manager_xpubs.iter().any(|xpub| xpub.warning)
    }

    fn check(&mut self) {
        for participant in &mut self.manager_xpubs {
            if ExtendedPubKey::from_str(&participant.xpub).is_err() {
                participant.warning = true;
            }
        }
    }

    fn update(&mut self, message: Message) {
        if let Message::DefineCpfpDescriptor(msg) = message {
            match msg {
                message::DefineCpfpDescriptor::ManagerXpub(i, message::ParticipantXpub::Delete) => {
                    self.manager_xpubs.remove(i);
                }
                message::DefineCpfpDescriptor::ManagerXpub(i, msg) => {
                    if let Some(xpub) = self.manager_xpubs.get_mut(i) {
                        xpub.update(msg);
                    }
                }
                message::DefineCpfpDescriptor::AddXpub => {
                    self.manager_xpubs.push(ParticipantXpub::new());
                }
            };
        };
    }

    fn view(&mut self) -> Element<Message> {
        return view::define_cpfp_descriptor(
            &mut self.add_xpub_button,
            self.manager_xpubs
                .iter_mut()
                .enumerate()
                .map(|(i, xpub)| {
                    xpub.view().map(move |msg| {
                        Message::DefineCpfpDescriptor(message::DefineCpfpDescriptor::ManagerXpub(
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

impl From<DefineCpfpDescriptor> for Box<dyn Step> {
    fn from(s: DefineCpfpDescriptor) -> Box<dyn Step> {
        Box::new(s)
    }
}
