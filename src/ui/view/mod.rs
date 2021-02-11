pub mod charging;
mod history;
mod home;
pub mod installing;
mod layout;
pub mod manager;
mod network;
mod sidebar;
pub mod vault;

pub use history::HistoryView;
pub use home::{ManagerHomeView, StakeholderHomeView};
pub use manager::ManagerSendView;
pub use network::{ManagerNetworkView, StakeholderNetworkView};

use bitcoin::Network;

use super::menu::Menu;
use crate::revault::Role;

pub struct Context {
    pub network: Network,
    pub network_up: bool,
    pub menu: Menu,
    pub role: Role,
    pub role_edit: bool,
}

impl Context {
    pub fn new(role_edit: bool, role: Role, menu: Menu) -> Self {
        Self {
            role,
            role_edit,
            menu,
            network: bitcoin::Network::Bitcoin,
            network_up: false,
        }
    }
}
