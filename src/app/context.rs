use super::menu::Menu;
use crate::{
    app::config, conversion::Converter, daemon::client::Client, daemon::client::RevaultD,
    revault::Role,
};
use revaultd::common::config::Config as DaemonConfig;
use revaultd::revault_tx::miniscript::DescriptorPublicKey;

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

    pub fn stakeholders_xpubs(&self) -> Vec<DescriptorPublicKey> {
        self.config.daemon.scripts_config.deposit_descriptor.xpubs()
    }

    pub fn managers_xpubs(&self) -> Vec<DescriptorPublicKey> {
        // The managers' xpubs are all the xpubs from the Unvault descriptor except the
        // Stakehodlers' ones and the Cosigning Servers' ones.
        let stk_xpubs = self.stakeholders_xpubs();
        self.config
            .daemon
            .scripts_config
            .unvault_descriptor
            .xpubs()
            .into_iter()
            .filter_map(|xpub| {
                match xpub {
                    DescriptorPublicKey::SinglePub(_) => None, // Cosig
                    DescriptorPublicKey::XPub(_) => {
                        if stk_xpubs.contains(&xpub) {
                            None // Stakeholder
                        } else {
                            Some(xpub) // Manager
                        }
                    }
                }
            })
            .collect()
    }
}

pub struct ConfigContext {
    pub daemon: DaemonConfig,
    pub gui: config::Config,
}
