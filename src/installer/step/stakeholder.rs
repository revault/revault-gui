use std::cmp::Ordering;
use std::str::FromStr;

use bitcoin::util::bip32::ExtendedPubKey;
use iced::Element;
use revaultd::revault_tx::{
    miniscript::DescriptorPublicKey,
    scripts::{DepositDescriptor, EmergencyAddress, UnvaultDescriptor},
};

use revault_ui::component::form;

use crate::installer::{
    config,
    message::{self, Message},
    step::{
        common::{CosignerKey, ParticipantXpub, RequiredXpub},
        Context, Step,
    },
    view,
};

pub struct DefineStakeholderXpubs {
    other_xpubs: Vec<ParticipantXpub>,
    our_xpub: RequiredXpub,
    warning: Option<String>,

    view: view::DefineStakeholderXpubsAsStakeholder,
}

impl DefineStakeholderXpubs {
    pub fn new() -> Self {
        Self {
            warning: None,
            our_xpub: RequiredXpub::new(),
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
                    self.our_xpub.update(xpub);
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
            participant.check_validity(&ctx.network);
        }

        self.our_xpub.check_validity(&ctx.network);

        if !self.our_xpub.xpub.valid
            || self
                .other_xpubs
                .iter()
                .any(|participant| !participant.xpub.valid)
        {
            return false;
        }

        config.stakeholder_config = Some(config::StakeholderConfig {
            xpub: ExtendedPubKey::from_str(&self.our_xpub.xpub.value).expect("already checked"),
            watchtowers: Vec::new(),
            emergency_address: "".to_string(),
        });

        let mut xpubs: Vec<String> = self
            .other_xpubs
            .iter()
            .map(|participant| participant.xpub.value.clone())
            .collect();
        xpubs.push(self.our_xpub.xpub.value.clone());

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
            &self.our_xpub.xpub,
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
    managers_threshold: form::Value<usize>,
    spending_delay: form::Value<u32>,
    manager_xpubs: Vec<ParticipantXpub>,
    cosigners_enabled: bool,
    cosigners: Vec<CosignerKey>,
    warning: Option<String>,
    view: view::DefineManagerXpubsAsStakeholderOnly,

    /// from previous step
    stakeholder_xpubs: Vec<String>,
}

impl DefineManagerXpubs {
    pub fn new() -> Self {
        Self {
            managers_threshold: form::Value {
                value: 1,
                valid: true,
            },
            spending_delay: form::Value {
                value: 10,
                valid: true,
            },
            manager_xpubs: Vec::new(),
            cosigners_enabled: false,
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
        self.cosigners_enabled = ctx.cosigners_enabled;
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
                message::DefineManagerXpubs::CosignersEnabled(enable) => {
                    self.cosigners_enabled = enable
                }
                message::DefineManagerXpubs::ManagersThreshold(action) => match action {
                    message::Action::Increment => {
                        self.managers_threshold.valid = true;
                        self.managers_threshold.value += 1;
                    }
                    message::Action::Decrement => {
                        self.managers_threshold.valid = true;
                        if self.managers_threshold.value > 0 {
                            self.managers_threshold.value -= 1;
                        }
                    }
                },
                message::DefineManagerXpubs::SpendingDelay(action) => match action {
                    message::Action::Increment => {
                        self.spending_delay.value += 1;
                        self.spending_delay.valid = true;
                    }
                    message::Action::Decrement => {
                        self.spending_delay.valid = true;
                        if self.spending_delay.value > 0 {
                            self.spending_delay.value -= 1;
                        }
                    }
                },
                _ => {}
            };
        };
    }

    fn apply(&mut self, ctx: &mut Context, config: &mut config::Config) -> bool {
        for participant in &mut self.manager_xpubs {
            participant.check_validity(&ctx.network);
        }

        for cosigner in &mut self.cosigners {
            cosigner.key.valid = DescriptorPublicKey::from_str(&cosigner.key.value).is_ok();
        }

        self.managers_threshold.valid = self.managers_threshold.value != 0
            && self.managers_threshold.value <= self.manager_xpubs.len();
        self.spending_delay.valid = self.spending_delay.value != 0;

        if self
            .manager_xpubs
            .iter()
            .any(|participant| !participant.xpub.valid)
            || (self.cosigners_enabled && self.cosigners.iter().any(|cosigner| !cosigner.key.valid))
            || !self.managers_threshold.valid
            || !self.spending_delay.valid
        {
            return false;
        }

        let mut managers_xpubs: Vec<String> = self
            .manager_xpubs
            .iter()
            .map(|participant| format!("{}/*", participant.xpub.value.clone()))
            .collect();

        managers_xpubs.sort();

        let managers_keys: Vec<DescriptorPublicKey> = managers_xpubs
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

        let cosigners_keys = if self.cosigners_enabled {
            let mut cosigners_keys: Vec<String> = self
                .cosigners
                .iter()
                .map(|cosigner| cosigner.key.value.clone())
                .collect();
            cosigners_keys.sort();
            cosigners_keys
                .into_iter()
                .map(|key| DescriptorPublicKey::from_str(&key).expect("already checked"))
                .collect()
        } else {
            Vec::new()
        };

        ctx.number_cosigners = self.cosigners.len();
        ctx.number_managers = managers_keys.len();

        match UnvaultDescriptor::new(
            stakeholders_keys,
            managers_keys,
            self.managers_threshold.value,
            cosigners_keys,
            self.spending_delay.value,
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
        let cosigners = if self.cosigners_enabled {
            self.cosigners
                .iter_mut()
                .enumerate()
                .map(|(i, key)| {
                    key.view().map(move |msg| {
                        Message::DefineManagerXpubs(message::DefineManagerXpubs::CosignerKey(
                            i, msg,
                        ))
                    })
                })
                .collect()
        } else {
            Vec::new()
        };
        self.view.render(
            &self.managers_threshold,
            &self.spending_delay,
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
            cosigners,
            self.cosigners_enabled,
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
    warning: Option<String>,

    view: view::DefineEmergencyAddress,
}

impl DefineEmergencyAddress {
    pub fn new() -> Self {
        Self {
            address: form::Value::default(),
            view: view::DefineEmergencyAddress::new(),
            warning: None,
        }
    }
}

impl Step for DefineEmergencyAddress {
    fn update(&mut self, message: Message) {
        if let Message::DefineEmergencyAddress(address) = message {
            self.address.value = address;
            self.address.valid = true;
            self.warning = None;
        };
    }

    fn apply(&mut self, _ctx: &mut Context, config: &mut config::Config) -> bool {
        match bitcoin::Address::from_str(&self.address.value) {
            Ok(address) => {
                if EmergencyAddress::from(address.clone()).is_err() {
                    self.warning = Some("address is not a v0 P2WSH".to_string());
                    return false;
                }

                // All good, signet addresses have the testnet type
                if address.network == bitcoin::Network::Testnet
                    && config.bitcoind_config.network == bitcoin::Network::Signet
                {
                } else if address.network != config.bitcoind_config.network {
                    self.warning = Some(format!(
                        "address is not usable with the specified bitcoind network: {}",
                        config.bitcoind_config.network
                    ));
                    return false;
                }
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
        self.view.render(&self.address, self.warning.as_ref())
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
