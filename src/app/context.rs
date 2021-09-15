use bitcoin::Network;

use super::menu::Menu;
use crate::{conversion::Converter, daemon::client::RevaultD, revault::Role};
use std::sync::Arc;

/// Context is an object passing general information
/// and service clients through the application components.
pub struct Context {
    pub revaultd: Arc<RevaultD>,
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
        revaultd: Arc<RevaultD>,
        converter: Converter,
        network: Network,
        role_edit: bool,
        role: Role,
        menu: Menu,
        managers_threshold: usize,
    ) -> Self {
        Self {
            revaultd,
            converter,
            role,
            role_edit,
            menu,
            network,
            network_up: true,
            managers_threshold,
        }
    }
}
