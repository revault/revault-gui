use std::cmp::Ordering;
use std::net::SocketAddr;
use std::str::FromStr;

use bitcoin::hashes::hex::FromHex;
use bitcoin::util::bip32::ExtendedPubKey;
use iced::{button::State as Button, scrollable, Element};
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
    stakeholder_xpubs: Vec<ParticipantXpub>,
    add_xpub_button: Button,
    scroll: scrollable::State,
    previous_button: Button,
    save_button: Button,
    warning: Option<String>,
}

impl DefineStakeholderXpubs {
    pub fn new() -> Self {
        Self {
            add_xpub_button: Button::new(),
            stakeholder_xpubs: Vec::new(),
            scroll: scrollable::State::new(),
            previous_button: Button::new(),
            save_button: Button::new(),
            warning: None,
        }
    }
}

impl Default for DefineStakeholderXpubs {
    fn default() -> Self {
        Self::new()
    }
}

impl Step for DefineStakeholderXpubs {
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

    fn apply(&mut self, ctx: &mut Context, config: &mut config::Config) -> bool {
        for participant in &mut self.stakeholder_xpubs {
            participant.xpub.valid = ExtendedPubKey::from_str(&participant.xpub.value).is_ok()
        }

        if self
            .stakeholder_xpubs
            .iter()
            .any(|participant| !participant.xpub.valid)
        {
            return false;
        }

        let mut xpubs: Vec<String> = self
            .stakeholder_xpubs
            .iter()
            .map(|participant| format!("{}/*", participant.xpub.value.clone()))
            .collect();

        xpubs.sort();

        let keys = xpubs
            .into_iter()
            .map(|xpub| DescriptorPublicKey::from_str(&xpub).expect("already checked"))
            .collect();

        match DepositDescriptor::new(keys) {
            Ok(descriptor) => {
                self.warning = None;
                config.scripts_config.deposit_descriptor = descriptor.to_string()
            }
            Err(e) => self.warning = Some(e.to_string()),
        }

        if self.warning.is_some() {
            return false;
        }

        ctx.stakeholders_xpubs = self
            .stakeholder_xpubs
            .iter()
            .map(|participant| participant.xpub.value.clone())
            .collect();
        ctx.number_cosigners = ctx.stakeholders_xpubs.len();

        true
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
            self.warning.as_ref(),
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
    our_xpub: form::Value<String>,
    managers_threshold: usize,
    threshold_warning: bool,
    spending_delay: u32,
    spending_delay_warning: bool,
    warning: Option<String>,

    view: view::DefineManagerXpubsAsManager,

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
            our_xpub: form::Value::default(),
            other_xpubs: Vec::new(),
            cosigners: Vec::new(),
            view: view::DefineManagerXpubsAsManager::new(),
            stakeholder_xpubs: Vec::new(),
            warning: None,
        }
    }
}

impl Default for DefineManagerXpubs {
    fn default() -> Self {
        Self::new()
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
                message::DefineManagerXpubs::OurXpubEdited(xpub) => {
                    self.our_xpub.value = xpub;
                    self.our_xpub.valid = true;
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
                        self.spending_delay_warning = false;
                        self.spending_delay += 1;
                    }
                    message::Action::Decrement => {
                        self.spending_delay_warning = false;
                        if self.spending_delay > 0 {
                            self.spending_delay -= 1;
                        }
                    }
                },
            };
        };
    }

    fn apply(&mut self, ctx: &mut Context, config: &mut config::Config) -> bool {
        for participant in &mut self.other_xpubs {
            participant.xpub.valid = DescriptorPublicKey::from_str(&participant.xpub.value).is_ok();
        }

        self.our_xpub.valid = DescriptorPublicKey::from_str(&self.our_xpub.value).is_ok();

        for cosigner in &mut self.cosigners {
            cosigner.key.valid = DescriptorPublicKey::from_str(&cosigner.key.value).is_ok();
        }

        // If user is manager, other_xpubs can be equal to zero and threshold equal to 1.
        self.threshold_warning =
            self.managers_threshold == 0 || self.managers_threshold > self.other_xpubs.len() + 1;
        self.spending_delay_warning = self.spending_delay == 0;

        if !self.our_xpub.valid
            || self
                .other_xpubs
                .iter()
                .any(|participant| !participant.xpub.valid)
            || self.cosigners.iter().any(|cosigner| !cosigner.key.valid)
            || self.threshold_warning
            || self.spending_delay_warning
        {
            return false;
        }

        let mut managers_xpubs: Vec<String> = self
            .other_xpubs
            .iter()
            .map(|participant| format!("{}/*", participant.xpub.value.clone()))
            .collect();
        managers_xpubs.push(format!("{}/*", self.our_xpub.value.clone()));

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
        ctx.number_managers = managers_keys.len();

        config.manager_config = Some(config::ManagerConfig {
            xpub: ExtendedPubKey::from_str(&self.our_xpub.value).expect("already checked"),
            cosigners: Vec::new(),
        });

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
        return self.view.render(
            self.managers_threshold,
            self.threshold_warning,
            self.spending_delay,
            self.spending_delay_warning,
            &self.our_xpub,
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
        );
    }
}

impl From<DefineManagerXpubs> for Box<dyn Step> {
    fn from(s: DefineManagerXpubs) -> Box<dyn Step> {
        Box::new(s)
    }
}

pub struct Cosigner {
    pub host: form::Value<String>,
    pub noise_key: form::Value<String>,

    view: view::Cosigner,
}

impl Cosigner {
    pub fn new() -> Self {
        Self {
            host: form::Value::default(),
            noise_key: form::Value::default(),
            view: view::Cosigner::new(),
        }
    }

    pub fn update(&mut self, msg: message::DefineCosigner) {
        match msg {
            message::DefineCosigner::HostEdited(host) => {
                self.host.value = host;
                self.host.valid = true;
            }
            message::DefineCosigner::NoiseKeyEdited(key) => {
                self.noise_key.value = key;
                self.noise_key.valid = true;
            }
        }
    }

    pub fn view(&mut self) -> Element<message::DefineCosigner> {
        self.view.render(&self.host, &self.noise_key)
    }
}

impl Default for Cosigner {
    fn default() -> Self {
        Self::new()
    }
}

pub struct DefineCosigners {
    cosigners: Vec<Cosigner>,
    view: view::DefineCosigners,
}

impl DefineCosigners {
    pub fn new() -> Self {
        Self {
            cosigners: Vec::new(),
            view: view::DefineCosigners::new(),
        }
    }
}

impl Step for DefineCosigners {
    fn load_context(&mut self, ctx: &Context) {
        while self.cosigners.len() != ctx.number_cosigners {
            match self.cosigners.len().cmp(&ctx.number_cosigners) {
                Ordering::Greater => {
                    self.cosigners.pop();
                }
                Ordering::Less => self.cosigners.push(Cosigner::new()),
                Ordering::Equal => (),
            }
        }
    }

    fn update(&mut self, message: Message) {
        if let Message::DefineCosigners(i, msg) = message {
            if let Some(cosigner) = self.cosigners.get_mut(i) {
                cosigner.update(msg);
            }
        };
    }

    fn apply(&mut self, _ctx: &mut Context, config: &mut config::Config) -> bool {
        for cosigner in &mut self.cosigners {
            if let Ok(bytes) = Vec::from_hex(&cosigner.noise_key.value) {
                if bytes.len() != 32 {
                    cosigner.noise_key.valid = false;
                }
            } else {
                cosigner.noise_key.valid = false;
            }

            cosigner.host.valid = SocketAddr::from_str(&cosigner.host.value).is_ok();
        }

        if self
            .cosigners
            .iter()
            .any(|cosigner| !cosigner.noise_key.valid || !cosigner.host.valid)
        {
            return false;
        }

        if let Some(manager_config) = &mut config.manager_config {
            manager_config.cosigners = self
                .cosigners
                .iter()
                .map(|cosigner| config::CosignerConfig {
                    host: cosigner.host.value.clone(),
                    noise_key: cosigner.noise_key.value.clone(),
                })
                .collect();
        }

        true
    }

    fn view(&mut self) -> Element<Message> {
        self.view.render(
            self.cosigners
                .iter_mut()
                .enumerate()
                .map(|(i, xpub)| xpub.view().map(move |msg| Message::DefineCosigners(i, msg)))
                .collect(),
        )
    }
}

impl Default for DefineCosigners {
    fn default() -> Self {
        Self::new()
    }
}

impl From<DefineCosigners> for Box<dyn Step> {
    fn from(s: DefineCosigners) -> Box<dyn Step> {
        Box::new(s)
    }
}
