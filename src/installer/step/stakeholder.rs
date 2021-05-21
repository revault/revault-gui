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
    other_xpubs: Vec<ParticipantXpub>,
    our_xpub: String,
    our_xpub_warning: bool,

    our_xpub_input: text_input::State,
    previous_button: Button,
    save_button: Button,
    add_xpub_button: Button,
    scroll: scrollable::State,
}

impl DefineStakeholderXpubs {
    pub fn new() -> Self {
        Self {
            our_xpub: "".to_string(),
            our_xpub_warning: false,
            other_xpubs: Vec::new(),
            our_xpub_input: text_input::State::new(),
            add_xpub_button: Button::new(),
            scroll: scrollable::State::new(),
            previous_button: Button::new(),
            save_button: Button::new(),
        }
    }
}

impl Step for DefineStakeholderXpubs {
    fn is_correct(&self) -> bool {
        !self.our_xpub_warning && !self.other_xpubs.iter().any(|xpub| xpub.warning)
    }

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

    fn update(&mut self, message: Message) {
        if let Message::DefineStakeholderXpubs(msg) = message {
            match msg {
                message::DefineStakeholderXpubs::OurXpubEdited(xpub) => {
                    self.our_xpub = xpub;
                    self.our_xpub_warning = false;
                }
                message::DefineStakeholderXpubs::StakeholderXpub(
                    i,
                    message::ParticipantXpub::Delete,
                ) => {
                    self.other_xpubs.remove(i);
                }
                message::DefineStakeholderXpubs::StakeholderXpub(i, msg) => {
                    if let Some(xpub) = self.other_xpubs.get_mut(i) {
                        xpub.update(msg)
                    };
                }
                message::DefineStakeholderXpubs::AddXpub => {
                    self.other_xpubs.push(ParticipantXpub::new());
                }
            };
        };
    }

    fn view(&mut self) -> Element<Message> {
        return view::define_stakeholder_xpubs_as_stakeholder(
            &self.our_xpub,
            &mut self.our_xpub_input,
            self.our_xpub_warning,
            &mut self.add_xpub_button,
            self.other_xpubs
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
    managers_treshold: u32,
    spending_delay: u32,
    manager_xpubs: Vec<ParticipantXpub>,
    cosigners: Vec<CosignerKey>,
    view: view::DefineManagerXpubsAsStakeholderOnly,
}

impl DefineManagerXpubs {
    pub fn new() -> Self {
        Self {
            managers_treshold: 0,
            spending_delay: 0,
            manager_xpubs: Vec::new(),
            cosigners: Vec::new(),
            view: view::DefineManagerXpubsAsStakeholderOnly::new(),
        }
    }
}
impl Step for DefineManagerXpubs {
    fn is_correct(&self) -> bool {
        self.manager_xpubs.iter().any(|xpub| xpub.warning)
            || self.cosigners.iter().any(|key| key.warning)
    }

    fn check(&mut self) {
        for participant in &mut self.manager_xpubs {
            if ExtendedPubKey::from_str(&participant.xpub).is_err() {
                participant.warning = true;
            }
        }
    }

    fn update(&mut self, message: Message) {
        if let Message::DefineManagerXpubs(msg) = message {
            match msg {
                message::DefineManagerXpubs::ManagerXpub(i, message::ParticipantXpub::Delete) => {
                    self.manager_xpubs.remove(i);
                }
                message::DefineManagerXpubs::ManagerXpub(i, msg) => {
                    if let Some(xpub) = self.manager_xpubs.get_mut(i) {
                        xpub.update(msg)
                    };
                }
                message::DefineManagerXpubs::AddXpub => {
                    self.manager_xpubs.push(ParticipantXpub::new());
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
                message::DefineManagerXpubs::ManagersTreshold(action) => match action {
                    message::Action::Increment => {
                        self.managers_treshold = self.managers_treshold + 1;
                    }
                    message::Action::Decrement => {
                        if self.managers_treshold > 0 {
                            self.managers_treshold = self.managers_treshold - 1;
                        }
                    }
                },
                message::DefineManagerXpubs::SpendingDelay(action) => match action {
                    message::Action::Increment => {
                        self.spending_delay = self.spending_delay + 1;
                    }
                    message::Action::Decrement => {
                        if self.spending_delay > 0 {
                            self.spending_delay = self.spending_delay - 1;
                        }
                    }
                },
                _ => {}
            };
        };
    }

    fn view(&mut self) -> Element<Message> {
        self.view.render(
            self.managers_treshold,
            self.spending_delay,
            self.manager_xpubs
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
        )
    }
}

impl From<DefineManagerXpubs> for Box<dyn Step> {
    fn from(s: DefineManagerXpubs) -> Box<dyn Step> {
        Box::new(s)
    }
}

pub struct DefineEmergencyAddress {
    address: String,
    warning: bool,

    view: view::DefineEmergencyAddress,
}

impl DefineEmergencyAddress {
    pub fn new() -> Self {
        Self {
            address: "".to_string(),
            warning: false,
            view: view::DefineEmergencyAddress::new(),
        }
    }
}

impl Step for DefineEmergencyAddress {
    fn is_correct(&self) -> bool {
        !self.warning
    }

    fn check(&mut self) {
        self.warning = bitcoin::Address::from_str(&self.address).is_err()
    }

    fn update(&mut self, message: Message) {
        if let Message::DefineEmergencyAddress(address) = message {
            self.address = address;
            self.warning = false;
        };
    }

    fn view(&mut self) -> Element<Message> {
        self.view.render(&self.address, self.warning)
    }
}

impl From<DefineEmergencyAddress> for Box<dyn Step> {
    fn from(s: DefineEmergencyAddress) -> Box<dyn Step> {
        Box::new(s)
    }
}

pub struct Watchtower {
    pub host: String,
    pub noise_key: String,
    warning_host: bool,
    warning_noise_key: bool,

    view: view::Watchtower,
}

impl Watchtower {
    pub fn new() -> Self {
        Self {
            host: "".to_string(),
            noise_key: "".to_string(),
            warning_host: false,
            warning_noise_key: false,
            view: view::Watchtower::new(),
        }
    }

    pub fn update(&mut self, msg: message::DefineWatchtower) {
        match msg {
            message::DefineWatchtower::HostEdited(host) => {
                self.host = host;
                self.warning_host = false;
            }
            message::DefineWatchtower::NoiseKeyEdited(key) => {
                self.noise_key = key;
                self.warning_noise_key = false;
            }
            _ => {}
        }
    }

    pub fn view(&mut self) -> Element<message::DefineWatchtower> {
        self.view.render(
            &self.host,
            &self.noise_key,
            self.warning_host,
            self.warning_noise_key,
        )
    }
}

pub struct DefineWatchtowers {
    watchtowers: Vec<Watchtower>,
    view: view::DefineWatchtowers,
}

impl DefineWatchtowers {
    pub fn new() -> Self {
        Self {
            watchtowers: vec![Watchtower::new()],
            view: view::DefineWatchtowers::new(),
        }
    }
}

impl Step for DefineWatchtowers {
    fn is_correct(&self) -> bool {
        !self
            .watchtowers
            .iter()
            .any(|wt| wt.warning_host || wt.warning_noise_key)
    }

    fn check(&mut self) {
        for _watchtower in &mut self.watchtowers {
            // TODO
        }
    }

    fn update(&mut self, message: Message) {
        if let Message::DefineWatchtowers(msg) = message {
            match msg {
                message::DefineWatchtowers::EditWatchtower(
                    i,
                    message::DefineWatchtower::Delete,
                ) => {
                    self.watchtowers.remove(i);
                }
                message::DefineWatchtowers::EditWatchtower(i, msg) => {
                    if let Some(watchtower) = self.watchtowers.get_mut(i) {
                        watchtower.update(msg);
                    }
                }
                message::DefineWatchtowers::AddWatchtower => {
                    self.watchtowers.push(Watchtower::new());
                }
            };
        };
    }

    fn view(&mut self) -> Element<Message> {
        self.view.render(
            self.watchtowers
                .iter_mut()
                .enumerate()
                .map(|(i, xpub)| {
                    xpub.view().map(move |msg| {
                        Message::DefineWatchtowers(message::DefineWatchtowers::EditWatchtower(
                            i, msg,
                        ))
                    })
                })
                .collect(),
        )
    }
}

impl From<DefineWatchtowers> for Box<dyn Step> {
    fn from(s: DefineWatchtowers) -> Box<dyn Step> {
        Box::new(s)
    }
}
