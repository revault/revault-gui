use bitcoin::util::bip32::ExtendedPubKey;
use iced::{button::State as Button, scrollable, text_input, Element};
use miniscript::DescriptorPublicKey;
use revault_tx::scripts::{DepositDescriptor, UnvaultDescriptor};
use std::str::FromStr;

use crate::{
    installer::{
        message::{self, Message},
        step::{
            common::{CosignerKey, ParticipantXpub},
            Context, Step,
        },
        view,
    },
    revaultd::config,
};

pub struct DefineStakeholderXpubs {
    other_xpubs: Vec<ParticipantXpub>,
    our_xpub: String,
    our_xpub_warning: bool,
    warning: Option<String>,

    our_xpub_input: text_input::State,
    previous_button: Button,
    save_button: Button,
    add_xpub_button: Button,
    scroll: scrollable::State,
}

impl DefineStakeholderXpubs {
    pub fn new() -> Self {
        Self {
            warning: None,
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

    fn apply(&mut self, ctx: &mut Context, config: &mut config::Config) -> bool {
        for participant in &mut self.other_xpubs {
            if ExtendedPubKey::from_str(&participant.xpub).is_err() {
                participant.warning = true;
            }
        }

        if ExtendedPubKey::from_str(&self.our_xpub).is_err() {
            self.our_xpub_warning = true;
        }

        if self.our_xpub_warning || self.other_xpubs.iter().any(|xpub| xpub.warning) {
            return false;
        }

        config.stakeholder_config = Some(config::StakeholderConfig {
            xpub: ExtendedPubKey::from_str(&self.our_xpub).expect("already checked"),
            watchtowers: Vec::new(),
            emergency_address: "".to_string(),
        });

        let mut xpubs: Vec<String> = self
            .other_xpubs
            .iter()
            .map(|participant| participant.xpub.clone())
            .collect();
        xpubs.push(self.our_xpub.clone());

        xpubs.sort();

        // update ctx for the unvault descriptor next step
        ctx.stakeholders_xpubs = xpubs.clone();

        let keys = xpubs
            .into_iter()
            .map(|xpub| {
                DescriptorPublicKey::from_str(&format!("{}/*", xpub)).expect("already checked")
            })
            .collect();

        match DepositDescriptor::new(keys) {
            Ok(descriptor) => {
                self.warning = None;
                config.scripts_config.deposit_descriptor = descriptor.to_string();
            }
            Err(e) => self.warning = Some(e.to_string()),
        };

        self.warning.is_none()
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
            self.warning.as_ref(),
        );
    }
}

impl Default for DefineStakeholderXpubs {
    fn default() -> Self {
        Self::new()
    }
}

impl From<DefineStakeholderXpubs> for Box<dyn Step> {
    fn from(s: DefineStakeholderXpubs) -> Box<dyn Step> {
        Box::new(s)
    }
}

pub struct DefineManagerXpubs {
    managers_treshold: usize,
    treshold_warning: bool,
    spending_delay: u32,
    spending_delay_warning: bool,
    manager_xpubs: Vec<ParticipantXpub>,
    cosigners: Vec<CosignerKey>,
    warning: Option<String>,
    view: view::DefineManagerXpubsAsStakeholderOnly,

    /// from previous step
    stakeholder_xpubs: Vec<String>,
}

impl DefineManagerXpubs {
    pub fn new() -> Self {
        Self {
            managers_treshold: 0,
            treshold_warning: false,
            spending_delay: 0,
            spending_delay_warning: false,
            manager_xpubs: Vec::new(),
            cosigners: Vec::new(),
            view: view::DefineManagerXpubsAsStakeholderOnly::new(),
            stakeholder_xpubs: Vec::new(),
            warning: None,
        }
    }
}
impl Step for DefineManagerXpubs {
    fn load_context(&mut self, ctx: &Context) {
        self.stakeholder_xpubs = ctx.stakeholders_xpubs.clone();
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
                        self.treshold_warning = false;
                        self.managers_treshold += 1;
                    }
                    message::Action::Decrement => {
                        self.treshold_warning = false;
                        if self.managers_treshold > 0 {
                            self.managers_treshold -= 1;
                        }
                    }
                },
                message::DefineManagerXpubs::SpendingDelay(action) => match action {
                    message::Action::Increment => {
                        self.spending_delay += 1;
                        self.spending_delay_warning = false;
                    }
                    message::Action::Decrement => {
                        self.spending_delay_warning = false;
                        if self.spending_delay > 0 {
                            self.spending_delay -= 1;
                        }
                    }
                },
                _ => {}
            };
        };
    }

    fn apply(&mut self, ctx: &mut Context, config: &mut config::Config) -> bool {
        for participant in &mut self.manager_xpubs {
            if ExtendedPubKey::from_str(&participant.xpub).is_err() {
                participant.warning = true;
            }
        }

        for cosigner in &mut self.cosigners {
            if DescriptorPublicKey::from_str(&cosigner.key).is_err() {
                cosigner.warning = true;
            }
        }

        self.treshold_warning =
            self.managers_treshold == 0 || self.managers_treshold > self.manager_xpubs.len();
        self.spending_delay_warning = self.spending_delay == 0;

        if self.manager_xpubs.iter().any(|xpub| xpub.warning)
            || self.cosigners.iter().any(|key| key.warning)
            || self.treshold_warning
            || self.spending_delay_warning
        {
            return false;
        }

        let mut managers_xpubs: Vec<String> = self
            .manager_xpubs
            .iter()
            .map(|participant| format!("{}/*", participant.xpub.clone()))
            .collect();

        managers_xpubs.sort();

        let managers_keys = managers_xpubs
            .into_iter()
            .map(|xpub| DescriptorPublicKey::from_str(&xpub).expect("already checked"))
            .collect();

        let mut stakeholders_xpubs: Vec<String> = self
            .stakeholder_xpubs
            .iter()
            .map(|xpub| format!("{}/*", xpub.clone()))
            .collect();

        stakeholders_xpubs.sort();

        let stakeholders_keys = stakeholders_xpubs
            .into_iter()
            .map(|xpub| DescriptorPublicKey::from_str(&xpub).expect("already checked"))
            .collect();

        let mut cosigners_keys: Vec<String> = self
            .cosigners
            .iter()
            .map(|cosigner| cosigner.key.clone())
            .collect();

        cosigners_keys.sort();

        let cosigners_keys = cosigners_keys
            .into_iter()
            .map(|key| DescriptorPublicKey::from_str(&key).expect("already checked"))
            .collect();

        ctx.number_cosigners = self.cosigners.len();

        match UnvaultDescriptor::new(
            stakeholders_keys,
            managers_keys,
            self.managers_treshold,
            cosigners_keys,
            self.spending_delay,
        ) {
            Ok(descriptor) => {
                self.warning = None;
                config.scripts_config.unvault_descriptor = descriptor.to_string()
            }
            Err(e) => self.warning = Some(e.to_string()),
        };

        self.warning.is_none()
    }

    fn view(&mut self) -> Element<Message> {
        self.view.render(
            self.managers_treshold,
            self.treshold_warning,
            self.spending_delay,
            self.spending_delay_warning,
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
            self.warning.as_ref(),
        )
    }
}

impl Default for DefineManagerXpubs {
    fn default() -> Self {
        Self::new()
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
    fn update(&mut self, message: Message) {
        if let Message::DefineEmergencyAddress(address) = message {
            self.address = address;
            self.warning = false;
        };
    }

    fn apply(&mut self, _ctx: &mut Context, config: &mut config::Config) -> bool {
        match bitcoin::Address::from_str(&self.address) {
            Ok(_) => {
                if let Some(stakeholder_config) = &mut config.stakeholder_config {
                    stakeholder_config.emergency_address = self.address.clone();
                }
                self.warning = false;
                true
            }
            Err(_) => {
                self.warning = true;
                false
            }
        }
    }

    fn view(&mut self) -> Element<Message> {
        self.view.render(&self.address, self.warning)
    }
}

impl Default for DefineEmergencyAddress {
    fn default() -> Self {
        Self::new()
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

    // TODO: verify
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

impl Default for Watchtower {
    fn default() -> Self {
        Self::new()
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

    fn apply(&mut self, _ctx: &mut Context, config: &mut config::Config) -> bool {
        let mut ws = Vec::new();
        for watchtower in &self.watchtowers {
            ws.push(config::WatchtowerConfig {
                host: watchtower.host.clone(),
                noise_key: watchtower.noise_key.clone(),
            })
        }

        if let Some(stakeholder_config) = &mut config.stakeholder_config {
            stakeholder_config.watchtowers = ws;
        }

        true
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

impl Default for DefineWatchtowers {
    fn default() -> Self {
        Self::new()
    }
}

impl From<DefineWatchtowers> for Box<dyn Step> {
    fn from(s: DefineWatchtowers) -> Box<dyn Step> {
        Box::new(s)
    }
}
