mod utils;

use serde_json::json;
use std::str::FromStr;
use std::sync::Arc;

use utils::{
    mock::{Daemon, DaemonClient},
    sandbox::Sandbox,
};

use revault_gui::{
    app::{
        context::Context,
        menu::Menu,
        message::Message,
        state::{DepositState, EmergencyState},
    },
    conversion::Converter,
    daemon::{
        client::{GetInfoResponse, ListVaultsResponse, Request, RevaultD},
        config::Config,
        model::{DepositAddress, Vault, VaultStatus},
    },
    revault::Role,
};

#[tokio::test]
async fn test_deposit_state() {
    let addr = bitcoin::Address::from_str(
        "tb1qkldgvljmjpxrjq2ev5qxe8dvhn0dph9q85pwtfkjeanmwdue2akqj4twxj",
    )
    .unwrap();
    let daemon = Daemon::new(vec![
        (
            Some(json!({"method": "getinfo", "params": Option::<Request>::None})),
            Ok(json!(GetInfoResponse {
                blockheight: 0,
                network: "testnet".to_string(),
                sync: 1.0,
                version: "0.1".to_string(),
                managers_threshold: 3
            })),
        ),
        (
            Some(json!({"method": "getdepositaddress", "params": Option::<Request>::None})),
            Ok(json!(DepositAddress {
                address: addr.clone()
            })),
        ),
    ]);

    let sandbox: Sandbox<DaemonClient, DepositState> = Sandbox::new(DepositState::new());

    let cfg = Config::default();
    let client = daemon.run();
    let ctx = Context::new(
        Arc::new(RevaultD::new(&cfg, client).unwrap()),
        Converter::new(bitcoin::Network::Bitcoin),
        bitcoin::Network::Bitcoin,
        false,
        Role::Stakeholder,
        Menu::Vaults,
        3,
        false,
    );

    let sandbox = sandbox.load(&ctx).await;

    assert!(
        if let DepositState::Loaded { address, .. } = sandbox.state() {
            addr == *address
        } else {
            false
        }
    )
}

#[tokio::test]
async fn test_emergency_state() {
    let daemon = Daemon::new(vec![
        (
            Some(json!({"method": "getinfo", "params": Option::<Request>::None})),
            Ok(json!(GetInfoResponse {
                blockheight: 0,
                network: "testnet".to_string(),
                sync: 1.0,
                version: "0.1".to_string(),
                managers_threshold: 3
            })),
        ),
        (
            Some(json!({"method": "listvaults", "params": Some(&[[
                VaultStatus::Secured,
                VaultStatus::Active,
                VaultStatus::Activating,
                VaultStatus::Unvaulting,
                VaultStatus::Unvaulted,
                VaultStatus::EmergencyVaulting,
                VaultStatus::EmergencyVaulted,
                VaultStatus::UnvaultEmergencyVaulting,
                VaultStatus::UnvaultEmergencyVaulted,
            ]])})),
            Ok(json!(ListVaultsResponse {
                vaults: vec![
                    Vault {
                        address: "".to_string(),
                        amount: 500,
                        derivation_index: 0,
                        status: VaultStatus::Secured,
                        txid: bitcoin::Txid::from_str(
                            "a1075db55d416d3ca199f55b6084e2115b9345e16c5cf302fc80e9d5fbf5d48d"
                        )
                        .unwrap(),
                        vout: 1,
                    },
                    Vault {
                        address: "".to_string(),
                        amount: 700,
                        derivation_index: 0,
                        status: VaultStatus::Secured,
                        txid: bitcoin::Txid::from_str(
                            "a1075db55d416d3ca199f55b6084e2115b9345e16c5cf302fc80e9d5fbf5d48d"
                        )
                        .unwrap(),
                        vout: 1,
                    }
                ]
            })),
        ),
        (
            Some(json!({"method": "emergency", "params": Option::<Request>::None})),
            Ok(json!({})),
        ),
    ]);

    let sandbox: Sandbox<DaemonClient, EmergencyState> = Sandbox::new(EmergencyState::new());

    let cfg = Config::default();
    let client = daemon.run();
    let ctx = Context::new(
        Arc::new(RevaultD::new(&cfg, client).unwrap()),
        Converter::new(bitcoin::Network::Bitcoin),
        bitcoin::Network::Bitcoin,
        false,
        Role::Stakeholder,
        Menu::Vaults,
        3,
        false,
    );

    let sandbox = sandbox.load(&ctx).await;
    assert!(matches!(sandbox.state(), EmergencyState::Loaded { .. }));

    if let EmergencyState::Loaded {
        vaults_number,
        funds_amount,
        ..
    } = sandbox.state()
    {
        assert_eq!(*vaults_number, 2);
        assert_eq!(*funds_amount, 1200);
    }

    let sandbox = sandbox.update(&ctx, Message::Emergency).await;
    assert!(matches!(sandbox.state(), EmergencyState::Triggered { .. }));
}
