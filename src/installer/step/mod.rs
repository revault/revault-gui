mod common;
pub mod manager;
pub mod stakeholder;

use iced::{button::State as Button, scrollable, Element};

use crate::installer::{message::Message, view};

pub trait Step {
    fn check(&mut self) {}
    fn update(&mut self, message: Message);
    fn view(&mut self) -> Element<Message>;
    fn is_correct(&self) -> bool {
        true
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
