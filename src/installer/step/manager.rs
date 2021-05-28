use bitcoin::util::bip32::ExtendedPubKey;
use iced::{button::State as Button, scrollable, Element};
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
    fn update_context(&self, ctx: &mut Context) {
        ctx.stakeholders_xpubs = self
            .stakeholder_xpubs
            .iter()
            .map(|participant| participant.xpub.clone())
            .collect();
    }

    fn is_correct(&self) -> bool {
        !self.stakeholder_xpubs.iter().any(|xpub| xpub.warning)
    }

    fn check(&mut self) {
        for participant in &mut self.stakeholder_xpubs {
            if DescriptorPublicKey::from_str(&participant.xpub).is_err() {
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

    fn edit_config(&self, config: &mut config::Config) {
        let mut xpubs: Vec<String> = self
            .stakeholder_xpubs
            .iter()
            .map(|participant| format!("{}/*", participant.xpub.clone()))
            .collect();

        xpubs.sort();

        let keys = xpubs
            .into_iter()
            .map(|xpub| DescriptorPublicKey::from_str(&xpub).expect("already checked"))
            .collect();

        config.scripts_config.deposit_descriptor =
            DepositDescriptor::new(keys).unwrap().to_string();
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
    managers_treshold: usize,
    spending_delay: u32,

    view: view::DefineManagerXpubsAsManager,

    /// from previous step
    stakeholder_xpubs: Vec<String>,
}

impl DefineManagerXpubs {
    pub fn new() -> Self {
        Self {
            managers_treshold: 0,
            spending_delay: 0,
            our_xpub: "".to_string(),
            our_xpub_warning: false,
            other_xpubs: Vec::new(),
            cosigners: Vec::new(),
            view: view::DefineManagerXpubsAsManager::new(),
            stakeholder_xpubs: Vec::new(),
        }
    }
}

impl Step for DefineManagerXpubs {
    fn update_context(&self, ctx: &mut Context) {
        ctx.number_cosigners = self.cosigners.len();
    }

    fn load_context(&mut self, ctx: &Context) {
        self.stakeholder_xpubs = ctx.stakeholders_xpubs.clone();
    }

    fn check(&mut self) {
        for participant in &mut self.other_xpubs {
            if DescriptorPublicKey::from_str(&participant.xpub).is_err() {
                participant.warning = true;
            }
        }
        if DescriptorPublicKey::from_str(&self.our_xpub).is_err() {
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
            };
        };
    }

    fn edit_config(&self, config: &mut config::Config) {
        let mut managers_xpubs: Vec<String> = self
            .other_xpubs
            .iter()
            .map(|participant| format!("{}/*", participant.xpub.clone()))
            .collect();
        managers_xpubs.push(format!("{}/*", self.our_xpub.clone()));

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

        config.scripts_config.unvault_descriptor = UnvaultDescriptor::new(
            stakeholders_keys,
            managers_keys,
            self.managers_treshold,
            cosigners_keys,
            self.spending_delay,
        )
        .unwrap()
        .to_string();

        config.manager_config = Some(config::ManagerConfig {
            xpub: ExtendedPubKey::from_str(&self.our_xpub).expect("already checked"),
            cosigners: Vec::new(),
        });
    }

    fn view(&mut self) -> Element<Message> {
        return self.view.render(
            self.managers_treshold,
            self.spending_delay,
            &self.our_xpub,
            self.our_xpub_warning,
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
        );
    }
}

impl From<DefineManagerXpubs> for Box<dyn Step> {
    fn from(s: DefineManagerXpubs) -> Box<dyn Step> {
        Box::new(s)
    }
}

pub struct Cosigner {
    pub host: String,
    pub noise_key: String,
    warning_host: bool,
    warning_noise_key: bool,

    view: view::Cosigner,
}

impl Cosigner {
    pub fn new() -> Self {
        Self {
            host: "".to_string(),
            noise_key: "".to_string(),
            warning_host: false,
            warning_noise_key: false,
            view: view::Cosigner::new(),
        }
    }

    pub fn update(&mut self, msg: message::DefineCosigner) {
        match msg {
            message::DefineCosigner::HostEdited(host) => {
                self.host = host;
                self.warning_host = false;
            }
            message::DefineCosigner::NoiseKeyEdited(key) => {
                self.noise_key = key;
                self.warning_noise_key = false;
            }
        }
    }

    pub fn view(&mut self) -> Element<message::DefineCosigner> {
        self.view.render(
            &self.host,
            &self.noise_key,
            self.warning_host,
            self.warning_noise_key,
        )
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
            if self.cosigners.len() > ctx.number_cosigners {
                self.cosigners.pop();
            } else if self.cosigners.len() < ctx.number_cosigners {
                self.cosigners.push(Cosigner::new());
            }
        }
    }

    fn is_correct(&self) -> bool {
        !self
            .cosigners
            .iter()
            .any(|wt| wt.warning_host || wt.warning_noise_key)
    }

    fn check(&mut self) {
        for _cosigner in &mut self.cosigners {
            // TODO
        }
    }

    fn update(&mut self, message: Message) {
        if let Message::DefineCosigners(i, msg) = message {
            if let Some(cosigner) = self.cosigners.get_mut(i) {
                cosigner.update(msg);
            }
        };
    }

    fn edit_config(&self, config: &mut config::Config) {
        let mut vec = Vec::new();
        for cosigner in &self.cosigners {
            vec.push(config::CosignerConfig {
                host: cosigner.host.clone(),
                noise_key: cosigner.noise_key.clone(),
            })
        }

        if let Some(manager_config) = &mut config.manager_config {
            manager_config.cosigners = vec;
        }
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

impl From<DefineCosigners> for Box<dyn Step> {
    fn from(s: DefineCosigners) -> Box<dyn Step> {
        Box::new(s)
    }
}
