use bitcoin::Network;

use super::menu::Menu;
use crate::{conversion::Converter, revault::Role};

pub struct Context {
    pub converter: Converter,
    pub network: Network,
    pub network_up: bool,
    pub menu: Menu,
    pub role: Role,
    pub role_edit: bool,
    pub managers_threshold: usize,
}

impl Context {
    pub fn new(
        converter: Converter,
        network: Network,
        role_edit: bool,
        role: Role,
        menu: Menu,
        managers_threshold: usize,
    ) -> Self {
        Self {
            converter,
            role,
            role_edit,
            menu,
            network,
            network_up: false,
            managers_threshold,
        }
    }
}

impl std::default::Default for Context {
    fn default() -> Self {
        Context {
            converter: Converter::new(Network::Bitcoin),
            network: Network::Bitcoin,
            network_up: false,
            role: Role::Manager,
            menu: Menu::Home,
            role_edit: false,
            managers_threshold: 1,
        }
    }
}
