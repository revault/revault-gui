mod utils;

use serde_json::json;
use std::path::PathBuf;
use std::str::FromStr;
use std::sync::Arc;

use utils::{fixtures::random_daemon_config, mock::Daemon, no_hardware_wallet, sandbox::Sandbox};

use bitcoin::{util::bip32, Address, Amount, OutPoint};

use revault_gui::{
    app::{
        config::Config as GUIConfig,
        context::{ConfigContext, Context},
        menu::Menu,
        message::Message,
        state::RevaultVaultsState,
    },
    conversion::Converter,
    daemon::{
        client::{ListVaultsResponse, RevaultD},
        model::{Vault, VaultStatus},
    },
    revault::Role,
};

#[tokio::test]
async fn test_revault_state() {
    let daemon = Daemon::new(vec![
        (
            Some(json!({"method": "listvaults", "params": Some(&[[
                VaultStatus::Unvaulting.to_string(),
                VaultStatus::Unvaulted.to_string(),
            ]])})),
            Ok(json!(ListVaultsResponse {
                vaults: vec![
                    Vault {
                        address: Address::from_str(
                            "tb1qkldgvljmjpxrjq2ev5qxe8dvhn0dph9q85pwtfkjeanmwdue2akqj4twxj"
                        )
                        .unwrap(),
                        amount: Amount::from_sat(500),
                        derivation_index: bip32::ChildNumber::from_normal_idx(0).unwrap(),
                        status: VaultStatus::Unvaulting,
                        txid: bitcoin::Txid::from_str(
                            "a1075db55d416d3ca199f55b6084e2115b9345e16c5cf302fc80e9d5fbf5d48d"
                        )
                        .unwrap(),
                        vout: 0,
                        blockheight: Some(1),
                        delegated_at: None,
                        secured_at: Some(1),
                        funded_at: Some(1),
                        moved_at: None
                    },
                    Vault {
                        address: Address::from_str(
                            "tb1qkldgvljmjpxrjq2ev5qxe8dvhn0dph9q85pwtfkjeanmwdue2akqj4twxj"
                        )
                        .unwrap(),
                        amount: Amount::from_sat(700),
                        derivation_index: bip32::ChildNumber::from_normal_idx(0).unwrap(),
                        status: VaultStatus::Unvaulted,
                        txid: bitcoin::Txid::from_str(
                            "a1075db55d416d3ca199f55b6084e2115b9345e16c5cf302fc80e9d5fbf5d48d"
                        )
                        .unwrap(),
                        vout: 1,
                        blockheight: Some(1),
                        delegated_at: None,
                        secured_at: Some(1),
                        funded_at: Some(1),
                        moved_at: None
                    },
                    Vault {
                        address: Address::from_str(
                            "tb1qkldgvljmjpxrjq2ev5qxe8dvhn0dph9q85pwtfkjeanmwdue2akqj4twxj"
                        )
                        .unwrap(),
                        amount: Amount::from_sat(700),
                        derivation_index: bip32::ChildNumber::from_normal_idx(0).unwrap(),
                        status: VaultStatus::Unvaulted,
                        txid: bitcoin::Txid::from_str(
                            "a1075db55d416d3ca199f55b6084e2115b9345e16c5cf302fc80e9d5fbf5d48d"
                        )
                        .unwrap(),
                        vout: 2,
                        blockheight: Some(1),
                        delegated_at: None,
                        secured_at: Some(1),
                        funded_at: Some(1),
                        moved_at: None
                    }
                ]
            })),
        ),
        (
            Some(json!({"method": "revault", "params": Some(&[
                OutPoint::from_str(
                    "a1075db55d416d3ca199f55b6084e2115b9345e16c5cf302fc80e9d5fbf5d48d:1",
                )
                .unwrap()
            ])})),
            Ok(json!({})),
        ),
        (
            Some(json!({"method": "revault", "params": Some(&[
                OutPoint::from_str(
                    "a1075db55d416d3ca199f55b6084e2115b9345e16c5cf302fc80e9d5fbf5d48d:2",
                )
                .unwrap()
            ])})),
            Ok(json!({})),
        ),
    ]);

    let sandbox: Sandbox<RevaultVaultsState> = Sandbox::new(RevaultVaultsState::default());

    let client = daemon.run();
    let ctx = Context::new(
        ConfigContext {
            daemon: random_daemon_config(),
            gui: GUIConfig::new(PathBuf::from_str("revault_gui.toml").unwrap()),
        },
        Arc::new(RevaultD::new(client)),
        Converter::new(bitcoin::Network::Bitcoin),
        Role::Stakeholder,
        Menu::RevaultVaults,
        Box::new(|| Box::pin(no_hardware_wallet())),
    );

    let sandbox = sandbox.load(&ctx).await;
    assert!(matches!(
        sandbox.state(),
        RevaultVaultsState::SelectVaults { .. }
    ));

    if let RevaultVaultsState::SelectVaults { vaults, .. } = sandbox.state() {
        assert_eq!(vaults.len(), 3);
    }

    // select vault 2
    let sandbox = sandbox
        .update(
            &ctx,
            Message::SelectVault(
                OutPoint::from_str(
                    "a1075db55d416d3ca199f55b6084e2115b9345e16c5cf302fc80e9d5fbf5d48d:1",
                )
                .unwrap(),
            ),
        )
        .await;

    // select vault 3
    let sandbox = sandbox
        .update(
            &ctx,
            Message::SelectVault(
                OutPoint::from_str(
                    "a1075db55d416d3ca199f55b6084e2115b9345e16c5cf302fc80e9d5fbf5d48d:2",
                )
                .unwrap(),
            ),
        )
        .await;

    assert!(matches!(
        sandbox.state(),
        RevaultVaultsState::SelectVaults { .. }
    ));

    if let RevaultVaultsState::SelectVaults { vaults, .. } = sandbox.state() {
        assert_eq!(vaults.iter().filter(|v| v.is_selected()).count(), 2);
    }

    let sandbox = sandbox.update(&ctx, Message::Revault).await;
    assert!(matches!(
        sandbox.state(),
        RevaultVaultsState::Success { .. }
    ));

    if let RevaultVaultsState::Success { vaults, .. } = sandbox.state() {
        assert_eq!(vaults.len(), 2);
    }
}
