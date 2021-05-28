mod common;
pub mod manager;
pub mod stakeholder;

use std::path::PathBuf;
use std::str::FromStr;

use bitcoin::{util::bip32::ExtendedPubKey, Network};
use iced::{button::State as Button, scrollable, Element};
use miniscript::DescriptorPublicKey;
use revault_tx::scripts::CpfpDescriptor;

use crate::{
    installer::{
        message::{self, Message},
        step::common::ParticipantXpub,
        view,
    },
    revaultd::config,
};

pub trait Step {
    fn check(&mut self) {}
    fn update(&mut self, message: Message);
    fn view(&mut self) -> Element<Message>;
    fn update_context(&self, _ctx: &mut Context) {}
    fn load_context(&mut self, _ctx: &Context) {}
    fn is_correct(&self) -> bool {
        true
    }
    fn edit_config(&self, _config: &mut config::Config) {}
}

pub struct Context {
    pub number_cosigners: usize,
    pub stakeholders_xpubs: Vec<String>,
}

impl Context {
    pub fn new() -> Self {
        Self {
            number_cosigners: 0,
            stakeholders_xpubs: Vec::new(),
        }
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

    fn edit_config(&self, config: &mut config::Config) {
        let mut xpubs: Vec<String> = self
            .manager_xpubs
            .iter()
            .map(|participant| format!("{}/*", participant.xpub))
            .collect();

        xpubs.sort();

        let keys = xpubs
            .into_iter()
            .map(|xpub| DescriptorPublicKey::from_str(&xpub).expect("already checked"))
            .collect();

        config.scripts_config.cpfp_descriptor = CpfpDescriptor::new(keys).unwrap().to_string();
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

pub struct DefineCoordinator {
    host: String,
    noise_key: String,
    warning: bool,

    view: view::DefineCoordinator,
}

impl DefineCoordinator {
    pub fn new() -> Self {
        Self {
            host: "".to_string(),
            noise_key: "".to_string(),
            warning: false,
            view: view::DefineCoordinator::new(),
        }
    }
}

impl Step for DefineCoordinator {
    fn is_correct(&self) -> bool {
        !self.warning
    }

    fn check(&mut self) {
        // TODO
    }

    fn update(&mut self, message: Message) {
        if let Message::DefineCoordinator(msg) = message {
            match msg {
                message::DefineCoordinator::HostEdited(host) => self.host = host,
                message::DefineCoordinator::NoiseKeyEdited(key) => self.noise_key = key,
            };
        };
    }

    fn edit_config(&self, config: &mut config::Config) {
        config.coordinator_host = self.host.clone();
        config.coordinator_noise_key = self.noise_key.clone();
    }

    fn view(&mut self) -> Element<Message> {
        self.view.render(&self.host, &self.noise_key, self.warning)
    }
}

impl From<DefineCoordinator> for Box<dyn Step> {
    fn from(s: DefineCoordinator) -> Box<dyn Step> {
        Box::new(s)
    }
}

pub struct DefineBitcoind {
    cookie_path: String,
    address: String,

    warning_cookie: bool,
    warning_address: bool,

    view: view::DefineBitcoind,
}

impl DefineBitcoind {
    pub fn new() -> Self {
        Self {
            cookie_path: "".to_string(),
            address: "".to_string(),
            warning_cookie: false,
            warning_address: false,
            view: view::DefineBitcoind::new(),
        }
    }
}

impl Step for DefineBitcoind {
    fn is_correct(&self) -> bool {
        !self.warning_address && !self.warning_cookie
    }

    fn check(&mut self) {
        self.warning_cookie = PathBuf::from_str(&self.cookie_path).is_err();
        self.warning_address = std::net::SocketAddr::from_str(&self.address).is_err();
        // TODO
    }

    fn update(&mut self, message: Message) {
        if let Message::DefineBitcoind(msg) = message {
            match msg {
                message::DefineBitcoind::AddressEdited(address) => {
                    self.address = address;
                    self.warning_address = false;
                }
                message::DefineBitcoind::CookiePathEdited(path) => {
                    self.cookie_path = path;
                    self.warning_cookie = false;
                }
            };
        };
    }

    fn edit_config(&self, config: &mut config::Config) {
        config.bitcoind_config = config::BitcoindConfig {
            network: Network::from_str("bitcoin").expect("already checked"),
            cookie_path: PathBuf::from_str(&self.cookie_path).expect("already checked"),
            poll_interval_secs: None,
            addr: std::net::SocketAddr::from_str(&self.address).expect("already checked"),
        }
    }

    fn view(&mut self) -> Element<Message> {
        self.view.render(
            &self.address,
            &self.cookie_path,
            self.warning_address,
            self.warning_cookie,
        )
    }
}

impl From<DefineBitcoind> for Box<dyn Step> {
    fn from(s: DefineBitcoind) -> Box<dyn Step> {
        Box::new(s)
    }
}

pub struct Final {
    generating: bool,
    success: bool,
    warning: Option<String>,
    view: view::Final,
}

impl Final {
    pub fn new() -> Self {
        Self {
            generating: false,
            success: false,
            warning: None,
            view: view::Final::new(),
        }
    }
}

impl Step for Final {
    fn update(&mut self, message: Message) {
        match message {
            Message::Installed(res) => {
                self.generating = false;
                if let Err(e) = res {
                    self.success = false;
                    self.warning = Some(e.to_string());
                } else {
                    self.success = true;
                }
            }
            Message::Install => {
                self.generating = true;
                self.success = false;
                self.warning = None;
            }
            _ => {}
        };
    }

    fn view(&mut self) -> Element<Message> {
        self.view
            .render(self.generating, self.success, self.warning.as_ref())
    }
}

impl From<Final> for Box<dyn Step> {
    fn from(s: Final) -> Box<dyn Step> {
        Box::new(s)
    }
}
