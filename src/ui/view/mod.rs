pub mod charging;
pub mod installing;
mod layout;
pub mod manager;
pub mod vault;

use bitcoin::Network;

pub struct Context {
    pub network: Network,
}

impl Context {
    pub fn new() -> Self {
        Self {
            network: bitcoin::Network::Bitcoin,
        }
    }
}
