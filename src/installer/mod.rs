mod message;
mod step;
mod view;

use iced::{executor, Application, Clipboard, Command, Element, Settings};

use std::path::PathBuf;

use crate::revault::Role;
use message::Message;
use step::{
    manager, stakeholder, DefineCoordinator, DefineCpfpDescriptor, DefineRole, Step, Welcome,
};

pub fn run(config_path: PathBuf) -> Result<(), iced::Error> {
    Installer::run(Settings::with_flags(config_path))
}

pub struct Installer {
    destination_path: PathBuf,
    exit: bool,

    current: usize,
    steps: Vec<Box<dyn Step>>,
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
            Message::Next => {
                self.current_step().check();
                if self.current_step().is_correct() {
                    self.next();
                }
            }
            Message::Previous => {
                self.previous();
            }
            Message::Role(role) => {
                self.update_steps(role);
                self.next();
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
