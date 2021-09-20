mod message;
mod step;
mod view;

use iced::{Clipboard, Command, Element, Subscription};

use bitcoin::hashes::hex::FromHex;
use std::io::Write;
use std::path::PathBuf;

use crate::{app::config as gui_config, daemon::config as revaultd_config, revault::Role};

pub use message::Message;
use step::{
    manager, stakeholder, Context, DefineBitcoind, DefineCoordinator, DefineCpfpDescriptor,
    DefinePrivateNoiseKey, DefineRole, Final, Step, Welcome,
};

pub struct Installer {
    /// Path to the revaultd binary.
    revaultd_path: Option<PathBuf>,
    current: usize,
    steps: Vec<Box<dyn Step>>,

    /// Context is data passed through each step.
    context: Context,
    config: revaultd_config::Config,
}

impl Installer {
    fn next(&mut self) {
        if self.current < self.steps.len() - 1 {
            self.current += 1;
        }
    }

    fn previous(&mut self) {
        if self.current > 0 {
            self.current -= 1;
        }
    }

    fn update_steps(&mut self, role: &[Role]) {
        if role == Role::MANAGER_ONLY {
            self.steps = vec![
                Welcome::new().into(),
                DefineRole::new().into(),
                DefinePrivateNoiseKey::new().into(),
                manager::DefineStakeholderXpubs::new().into(),
                manager::DefineManagerXpubs::new().into(),
                DefineCpfpDescriptor::new().into(),
                DefineCoordinator::new().into(),
                manager::DefineCosigners::new().into(),
                DefineBitcoind::new(self.config.bitcoind_config.network).into(),
                Final::new().into(),
            ];
        } else if role == Role::STAKEHOLDER_ONLY {
            self.steps = vec![
                Welcome::new().into(),
                DefineRole::new().into(),
                DefinePrivateNoiseKey::new().into(),
                stakeholder::DefineStakeholderXpubs::new().into(),
                stakeholder::DefineManagerXpubs::new().into(),
                DefineCpfpDescriptor::new().into(),
                DefineCoordinator::new().into(),
                DefineBitcoind::new(self.config.bitcoind_config.network).into(),
                stakeholder::DefineEmergencyAddress::new().into(),
                Final::new().into(),
            ];
        } else {
            self.steps = vec![
                Welcome::new().into(),
                DefineRole::new().into(),
                DefinePrivateNoiseKey::new().into(),
                stakeholder::DefineStakeholderXpubs::new().into(),
                manager::DefineManagerXpubs::new().into(),
                DefineCpfpDescriptor::new().into(),
                DefineCoordinator::new().into(),
                manager::DefineCosigners::new().into(),
                DefineBitcoind::new(self.config.bitcoind_config.network).into(),
                stakeholder::DefineEmergencyAddress::new().into(),
                Final::new().into(),
            ];
        }
    }

    fn current_step(&mut self) -> &mut Box<dyn Step> {
        self.steps
            .get_mut(self.current)
            .expect("There is always a step")
    }

    pub fn new(
        destination_path: PathBuf,
        revaultd_path: Option<PathBuf>,
        network: bitcoin::Network,
    ) -> (Installer, Command<Message>) {
        let mut config = revaultd_config::Config::new();
        config.bitcoind_config.network = network;
        config.data_dir = Some(destination_path);
        config.daemon = Some(true);
        (
            Installer {
                revaultd_path,
                config,
                current: 0,
                steps: vec![Welcome::new().into(), DefineRole::new().into()],
                context: Context::new(),
            },
            Command::none(),
        )
    }

    pub fn subscription(&self) -> Subscription<Message> {
        Subscription::none()
    }

    pub fn update(&mut self, message: Message, _clipboard: &mut Clipboard) -> Command<Message> {
        match message {
            Message::Next => {
                let current_step = self
                    .steps
                    .get_mut(self.current)
                    .expect("There is always a step");
                if current_step.apply(&mut self.context, &mut self.config) {
                    self.next();
                    // calculate new current_step.
                    let current_step = self
                        .steps
                        .get_mut(self.current)
                        .expect("There is always a step");
                    current_step.load_context(&self.context);
                }
            }
            Message::Previous => {
                self.previous();
            }
            Message::Role(role) => {
                // reset config
                let mut config = revaultd_config::Config::new();
                config.data_dir = self.config.data_dir.clone();
                config.daemon = Some(true);
                self.config = config;

                self.update_steps(role);
                self.next();
            }
            Message::Install => {
                self.current_step().update(message);
                return Command::perform(
                    install(
                        self.context.clone(),
                        self.config.clone(),
                        self.revaultd_path.clone(),
                    ),
                    Message::Installed,
                );
            }
            _ => {
                self.current_step().update(message);
            }
        };
        Command::none()
    }

    pub fn view(&mut self) -> Element<Message> {
        self.current_step().view()
    }
}

fn append_network_suffix(name: &str, network: &bitcoin::Network) -> String {
    if *network == bitcoin::Network::Bitcoin {
        name.to_string()
    } else {
        format!("{}_{}.toml", name.strip_suffix(".toml").unwrap(), network)
    }
}

pub async fn install(
    ctx: Context,
    mut cfg: revaultd_config::Config,
    revaultd_path: Option<PathBuf>,
) -> Result<PathBuf, Error> {
    let datadir_path = cfg.data_dir.clone().unwrap();
    std::fs::create_dir_all(&datadir_path)
        .map_err(|e| Error::CannotCreateDatadir(e.to_string()))?;

    cfg.data_dir =
        Some(datadir_path.canonicalize().map_err(|e| {
            Error::Unexpected(format!("Failed to canonicalize datadir path: {}", e))
        })?);

    // create revaultd configuration file
    let mut revaultd_config_path = datadir_path.clone();
    revaultd_config_path.push(append_network_suffix(
        revaultd_config::DEFAULT_FILE_NAME,
        &cfg.bitcoind_config.network,
    ));
    let mut revaultd_config_file = std::fs::File::create(&revaultd_config_path)
        .map_err(|e| Error::CannotCreateFile(e.to_string()))?;

    // Step needed because of ValueAfterTable error in the toml serialize implementation.
    let revaultd_config =
        toml::Value::try_from(&cfg).expect("daemon::Config has a proper Serialize implementation");

    revaultd_config_file
        .write_all(revaultd_config.to_string().as_bytes())
        .map_err(|e| Error::CannotWriteToFile(e.to_string()))?;

    // create network datadir
    let mut network_datadir = datadir_path.clone();
    network_datadir.push(cfg.bitcoind_config.network.to_string());
    std::fs::create_dir_all(&network_datadir)
        .map_err(|e| Error::CannotCreateDatadir(e.to_string()))?;

    // create noise_secret file
    let mut noise_secret_path = network_datadir;
    noise_secret_path.push("noise_secret");
    let mut noise_secret_file = std::fs::File::create(&noise_secret_path)
        .map_err(|e| Error::CannotCreateFile(e.to_string()))?;

    let private_noise_key: Vec<u8> = FromHex::from_hex(&ctx.private_noise_key)
        .map_err(|e| Error::CannotCreateFile(e.to_string()))?;
    noise_secret_file
        .write_all(&private_noise_key)
        .map_err(|e| Error::CannotWriteToFile(e.to_string()))?;

    // create revault GUI configuration file
    let mut gui_config_path = datadir_path;
    gui_config_path.push(append_network_suffix(
        gui_config::DEFAULT_FILE_NAME,
        &cfg.bitcoind_config.network,
    ));

    let mut gui_config_file = std::fs::File::create(&gui_config_path)
        .map_err(|e| Error::CannotCreateFile(e.to_string()))?;

    gui_config_file
        .write_all(
            toml::to_string(&gui_config::Config::new(
                revaultd_config_path.canonicalize().map_err(|e| {
                    Error::Unexpected(format!(
                        "Failed to canonicalize revaultd config path: {}",
                        e
                    ))
                })?,
                revaultd_path,
            ))
            .unwrap()
            .as_bytes(),
        )
        .map_err(|e| Error::CannotWriteToFile(e.to_string()))?;

    Ok(gui_config_path)
}

#[derive(Debug, Clone)]
pub enum Error {
    CannotCreateDatadir(String),
    CannotCreateFile(String),
    CannotWriteToFile(String),
    Unexpected(String),
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            Self::CannotCreateDatadir(e) => write!(f, "Failed to create datadir: {}", e),
            Self::CannotWriteToFile(e) => write!(f, "Failed to write to file: {}", e),
            Self::CannotCreateFile(e) => write!(f, "Failed to create file: {}", e),
            Self::Unexpected(e) => write!(f, "Unexpected: {}", e),
        }
    }
}
