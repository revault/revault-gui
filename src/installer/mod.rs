mod message;
mod step;
mod view;

use iced::{executor, Application, Clipboard, Command, Element, Settings};

use std::io::Write;
use std::path::PathBuf;

use crate::{app::config as gui_config, revault::Role, revaultd::config as revaultd_config};

use message::Message;
use step::{
    manager, stakeholder, Context, DefineBitcoind, DefineCoordinator, DefineCpfpDescriptor,
    DefineRole, Final, Step, Welcome,
};

pub fn run(config_path: PathBuf) -> Result<(), iced::Error> {
    Installer::run(Settings::with_flags(config_path))
}

pub struct Installer {
    destination_path: PathBuf,
    exit: bool,

    current: usize,
    steps: Vec<Box<dyn Step>>,

    /// Context is data passed through each step.
    context: Context,
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
                manager::DefineStakeholderXpubs::new().into(),
                manager::DefineManagerXpubs::new().into(),
                DefineCpfpDescriptor::new().into(),
                DefineCoordinator::new().into(),
                manager::DefineCosigners::new().into(),
                DefineBitcoind::new().into(),
                Final::new().into(),
            ];
        } else if role == Role::STAKEHOLDER_ONLY {
            self.steps = vec![
                Welcome::new().into(),
                DefineRole::new().into(),
                stakeholder::DefineStakeholderXpubs::new().into(),
                stakeholder::DefineManagerXpubs::new().into(),
                DefineCpfpDescriptor::new().into(),
                DefineCoordinator::new().into(),
                stakeholder::DefineEmergencyAddress::new().into(),
                DefineBitcoind::new().into(),
                Final::new().into(),
            ];
        } else {
            self.steps = vec![
                Welcome::new().into(),
                DefineRole::new().into(),
                stakeholder::DefineStakeholderXpubs::new().into(),
                manager::DefineManagerXpubs::new().into(),
                DefineCpfpDescriptor::new().into(),
                DefineCoordinator::new().into(),
                stakeholder::DefineEmergencyAddress::new().into(),
                stakeholder::DefineWatchtowers::new().into(),
                manager::DefineCosigners::new().into(),
                DefineBitcoind::new().into(),
                Final::new().into(),
            ];
        }
    }

    fn current_step(&mut self) -> &mut Box<dyn Step> {
        self.steps
            .get_mut(self.current)
            .expect("There is always a step")
    }
}

impl Application for Installer {
    type Executor = executor::Default;
    type Message = Message;
    type Flags = PathBuf;

    fn new(destination_path: PathBuf) -> (Installer, Command<Self::Message>) {
        (
            Installer {
                destination_path,
                exit: false,
                current: 0,
                steps: vec![Welcome::new().into(), DefineRole::new().into()],
                context: Context::new(),
            },
            Command::none(),
        )
    }

    fn title(&self) -> String {
        String::from("Revault Installer")
    }

    fn should_exit(&self) -> bool {
        self.exit
    }

    fn update(
        &mut self,
        message: Self::Message,
        _clipboard: &mut Clipboard,
    ) -> Command<Self::Message> {
        match message {
            Message::Exit => {
                self.exit = true;
            }
            Message::Next => {
                let current_step = self
                    .steps
                    .get_mut(self.current)
                    .expect("There is always a step");
                current_step.check();
                if current_step.is_correct() {
                    current_step.update_context(&mut self.context);
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
                self.update_steps(role);
                self.next();
            }
            Message::Install => {
                self.current_step().update(message);
                let mut cfg = revaultd_config::Config::new();
                for step in &self.steps {
                    step.edit_config(&mut cfg);
                }
                return Command::perform(
                    install(cfg, self.destination_path.clone()),
                    Message::Installed,
                );
            }
            _ => {
                self.current_step().update(message);
            }
        };
        Command::none()
    }

    fn view(&mut self) -> Element<Self::Message> {
        self.current_step().view()
    }
}

pub async fn install(cfg: revaultd_config::Config, datadir_path: PathBuf) -> Result<(), Error> {
    std::fs::create_dir_all(&datadir_path)
        .map_err(|e| Error::CannotCreateDatadir(e.to_string()))?;

    // create revaultd configuration file
    let mut revaultd_config_path = datadir_path.clone();
    revaultd_config_path.push(revaultd_config::DEFAULT_FILE_NAME);
    let mut revaultd_config_file = std::fs::File::create(&revaultd_config_path)
        .map_err(|e| Error::CannotCreateFile(e.to_string()))?;

    // Step needed because of ValueAfterTable error in the toml serialize implementation.
    let value = toml::Value::try_from(&cfg)
        .expect("revaultd::Config has a proper Serialize implementation");

    revaultd_config_file
        .write_all(value.to_string().as_bytes())
        .map_err(|e| Error::CannotWriteToFile(e.to_string()))?;

    // create revault GUI configuration file
    let cfg = gui_config::Config::new(revaultd_config_path);
    let mut gui_config_path = datadir_path.clone();
    gui_config_path.push(gui_config::DEFAULT_FILE_NAME);

    let mut gui_config_file = std::fs::File::create(gui_config_path)
        .map_err(|e| Error::CannotCreateFile(e.to_string()))?;

    gui_config_file
        .write_all(toml::to_string(&cfg).unwrap().as_bytes())
        .map_err(|e| Error::CannotWriteToFile(e.to_string()))?;

    Ok(())
}

#[derive(Debug, Clone)]
pub enum Error {
    CannotCreateDatadir(String),
    CannotCreateFile(String),
    CannotWriteToFile(String),
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            Self::CannotCreateDatadir(e) => write!(f, "Failed to create datadir: {}", e),
            Self::CannotWriteToFile(e) => write!(f, "Failed to write to file: {}", e),
            Self::CannotCreateFile(e) => write!(f, "Failed to create file: {}", e),
        }
    }
}