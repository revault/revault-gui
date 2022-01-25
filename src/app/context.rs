use super::menu::Menu;
use crate::{
    app::config,
    conversion::Converter,
    daemon::client::RevaultD,
    daemon::{self, client::Client},
    revault::Role,
};

use revault_hwi::{app::revault::RevaultHWI, HWIError};
use std::future::Future;
use std::pin::Pin;
use std::sync::Arc;

pub type HardwareWallet =
    Box<dyn Future<Output = Result<Box<dyn RevaultHWI + Send>, HWIError>> + Send + Sync>;

/// Context is an object passing general information
/// and service clients through the application components.
pub struct Context<C: Client> {
    pub config: ConfigContext,
    pub blockheight: u64,
    pub revaultd: Arc<RevaultD<C>>,
    pub converter: Converter,
    pub menu: Menu,
    pub role: Role,
    pub managers_threshold: usize,
    pub internal_daemon: bool,
    pub hardware_wallet: Box<dyn Fn() -> Pin<HardwareWallet> + Send + Sync>,
}

impl<C: Client> Context<C> {
    pub fn new(
        config: ConfigContext,
        revaultd: Arc<RevaultD<C>>,
        converter: Converter,
        role: Role,
        menu: Menu,
        internal_daemon: bool,
        hardware_wallet: Box<dyn Fn() -> Pin<HardwareWallet> + Send + Sync>,
    ) -> Self {
        Self {
            config,
            blockheight: 0,
            revaultd,
            converter,
            role,
            menu,
            managers_threshold: 0,
            internal_daemon,
            hardware_wallet,
        }
    }

    pub fn role_editable(&self) -> bool {
        self.config.daemon.stakeholder_config.is_some()
            && self.config.daemon.manager_config.is_some()
    }

    pub fn network(&self) -> bitcoin::Network {
        self.config.daemon.bitcoind_config.network
    }
}

pub struct ConfigContext {
    pub daemon: daemon::config::Config,
    pub gui: config::Config,
}
