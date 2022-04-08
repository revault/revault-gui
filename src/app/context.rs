use std::fs::OpenOptions;
use std::future::Future;
use std::io::Write;
use std::pin::Pin;
use std::sync::Arc;

use bitcoin::util::psbt::PartiallySignedTransaction as Psbt;

use revaultd::config::Config as DaemonConfig;
use revaultd::revault_tx::miniscript::DescriptorPublicKey;

use revault_hwi::{app::revault::RevaultHWI, HWIError};

use crate::{
    app::{config, error::Error, menu::Menu},
    conversion::Converter,
    daemon::Daemon,
    revault::Role,
};

pub type HardwareWallet =
    Box<dyn Future<Output = Result<Box<dyn RevaultHWI + Send>, HWIError>> + Send + Sync>;

/// Context is an object passing general information
/// and service clients through the application components.
pub struct Context {
    pub config: ConfigContext,
    pub blockheight: i32,
    pub revaultd: Arc<dyn Daemon + Sync + Send>,
    pub converter: Converter,
    pub menu: Menu,
    pub role: Role,
    pub managers_threshold: usize,
    pub hardware_wallet: Box<dyn Fn() -> Pin<HardwareWallet> + Send + Sync>,
}

impl Context {
    pub fn new(
        config: ConfigContext,
        revaultd: Arc<dyn Daemon + Sync + Send>,
        converter: Converter,
        role: Role,
        menu: Menu,
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

    pub fn user_signed(&self, psbt: &Psbt) -> bool {
        let man_fp = &self
            .config
            .daemon
            .manager_config
            .as_ref()
            .map(|key| key.xpub.fingerprint());
        let stk_fp = &self
            .config
            .daemon
            .stakeholder_config
            .as_ref()
            .map(|key| key.xpub.fingerprint());
        if let Some(input) = psbt.inputs.first() {
            input.partial_sigs.keys().any(|key| {
                input
                    .bip32_derivation
                    .get(key)
                    .map(|(fingerprint, _)| {
                        Some(*fingerprint) == *man_fp || Some(*fingerprint) == *stk_fp
                    })
                    .unwrap_or(false)
            })
        } else {
            false
        }
    }

    pub fn load_daemon_config(&mut self, cfg: DaemonConfig) -> Result<(), Error> {
        loop {
            if let Some(daemon) = Arc::get_mut(&mut self.revaultd) {
                daemon.load_config(cfg.clone())?;
                break;
            }
        }

        let mut daemon_config_file = OpenOptions::new()
            .write(true)
            .open(&self.config.gui.revaultd_config_path)
            .map_err(|e| Error::Config(e.to_string()))?;

        let content =
            toml::to_string(&self.config.daemon).map_err(|e| Error::Config(e.to_string()))?;

        daemon_config_file
            .write_all(content.as_bytes())
            .map_err(|e| {
                log::warn!("failed to write to file: {:?}", e);
                Error::Config(e.to_string())
            })?;

        Ok(())
    }
}

pub struct ConfigContext {
    pub daemon: DaemonConfig,
    pub gui: config::Config,
}
