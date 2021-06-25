use std::cmp::Ordering;
use std::str::FromStr;

use bitcoin::util::bip32::ExtendedPubKey;
use iced::Element;
use miniscript::DescriptorPublicKey;
use revault_tx::scripts::{DepositDescriptor, UnvaultDescriptor};

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
    ui::component::form,
};

pub struct DefineStakeholderXpubs {
    other_xpubs: Vec<ParticipantXpub>,
    our_xpub: form::Value<String>,
    warning: Option<String>,

    view: view::DefineStakeholderXpubsAsStakeholder,
}

impl DefineStakeholderXpubs {
    pub fn new() -> Self {
        Self {
            warning: None,
            our_xpub: form::Value::default(),
            other_xpubs: Vec::new(),
            view: view::DefineStakeholderXpubsAsStakeholder::new(),
        }
    }
}

impl Step for DefineStakeholderXpubs {
    fn update(&mut self, message: Message) {
        if let Message::DefineStakeholderXpubs(msg) = message {
            match msg {
                message::DefineStakeholderXpubs::OurXpubEdited(xpub) => {
                    self.our_xpub.value = xpub;
                    self.our_xpub.valid = true;
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
            participant.xpub.valid = ExtendedPubKey::from_str(&participant.xpub.value).is_ok();
        }

        self.our_xpub.valid = ExtendedPubKey::from_str(&self.our_xpub.value).is_ok();

        if !self.our_xpub.valid
            || self
                .other_xpubs
                .iter()
                .any(|participant| !participant.xpub.valid)
        {
            return false;
        }

        config.stakeholder_config = Some(config::StakeholderConfig {
            xpub: ExtendedPubKey::from_str(&self.our_xpub.value).expect("already checked"),
            watchtowers: Vec::new(),
            emergency_address: "".to_string(),
        });

        let mut xpubs: Vec<String> = self
            .other_xpubs
            .iter()
            .map(|participant| participant.xpub.value.clone())
            .collect();
        xpubs.push(self.our_xpub.value.clone());

        xpubs.sort();

        // update ctx for the unvault descriptor next step
        ctx.stakeholders_xpubs = xpubs.clone();
        ctx.number_cosigners = ctx.stakeholders_xpubs.len();

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
        return self.view.render(
            &self.our_xpub,
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
    managers_threshold: usize,
    threshold_warning: bool,
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
            managers_threshold: 1,
            threshold_warning: false,
            spending_delay: 10,
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
        while self.cosigners.len() != ctx.number_cosigners {
            match self.cosigners.len().cmp(&ctx.number_cosigners) {
                Ordering::Greater => {
                    self.cosigners.pop();
                }
                Ordering::Less => self.cosigners.push(CosignerKey::new()),
                Ordering::Equal => (),
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
                message::DefineManagerXpubs::CosignerKey(i, msg) => {
                    if let Some(key) = self.cosigners.get_mut(i) {
                        key.update(msg)
                    };
                }
                message::DefineManagerXpubs::ManagersThreshold(action) => match action {
                    message::Action::Increment => {
                        self.threshold_warning = false;
                        self.managers_threshold += 1;
                    }
                    message::Action::Decrement => {
                        self.threshold_warning = false;
                        if self.managers_threshold > 0 {
                            self.managers_threshold -= 1;
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
            participant.xpub.valid = ExtendedPubKey::from_str(&participant.xpub.value).is_ok();
        }

        for cosigner in &mut self.cosigners {
            cosigner.key.valid = DescriptorPublicKey::from_str(&cosigner.key.value).is_ok();
        }

        self.threshold_warning =
            self.managers_threshold == 0 || self.managers_threshold > self.manager_xpubs.len();
        self.spending_delay_warning = self.spending_delay == 0;

        if self
            .manager_xpubs
            .iter()
            .any(|participant| !participant.xpub.valid)
            || self.cosigners.iter().any(|cosigner| !cosigner.key.valid)
            || self.threshold_warning
            || self.spending_delay_warning
        {
            return false;
        }

        let mut managers_xpubs: Vec<String> = self
            .manager_xpubs
            .iter()
            .map(|participant| format!("{}/*", participant.xpub.value.clone()))
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
            .map(|cosigner| cosigner.key.value.clone())
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
            self.managers_threshold,
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
            self.managers_threshold,
            self.threshold_warning,
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
    address: form::Value<String>,

    view: view::DefineEmergencyAddress,
}

impl DefineEmergencyAddress {
    pub fn new() -> Self {
        Self {
            address: form::Value::default(),
            view: view::DefineEmergencyAddress::new(),
        }
    }
}

impl Step for DefineEmergencyAddress {
    fn update(&mut self, message: Message) {
        if let Message::DefineEmergencyAddress(address) = message {
            self.address.value = address;
            self.address.valid = true;
        };
    }

    fn apply(&mut self, _ctx: &mut Context, config: &mut config::Config) -> bool {
        match bitcoin::Address::from_str(&self.address.value) {
            Ok(_) => {
                if let Some(stakeholder_config) = &mut config.stakeholder_config {
                    stakeholder_config.emergency_address = self.address.value.clone();
                }
                self.address.valid = true;
                true
            }
            Err(_) => {
                self.address.valid = false;
                false
            }
        }
    }

    fn view(&mut self) -> Element<Message> {
        self.view.render(&self.address)
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
    pub host: form::Value<String>,
    pub noise_key: form::Value<String>,

    view: view::Watchtower,
}

impl Watchtower {
    pub fn new() -> Self {
        Self {
            host: form::Value::default(),
            noise_key: form::Value::default(),
            view: view::Watchtower::new(),
        }
    }

    pub fn update(&mut self, msg: message::DefineWatchtower) {
        match msg {
            message::DefineWatchtower::HostEdited(host) => {
                self.host.value = host;
                self.host.valid = true;
            }
            message::DefineWatchtower::NoiseKeyEdited(key) => {
                self.noise_key.value = key;
                self.noise_key.valid = true;
            }
            _ => {}
        }
    }

    pub fn view(&mut self) -> Element<message::DefineWatchtower> {
        self.view.render(&self.host, &self.noise_key)
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
                host: watchtower.host.value.clone(),
                noise_key: watchtower.noise_key.value.clone(),
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
