mod utils;

use serde_json::json;
use std::str::FromStr;
use std::sync::Arc;

use utils::{
    mock::{Daemon, DaemonClient},
    sandbox::Sandbox,
};

use bitcoin::hashes::hex::FromHex;

use revault_gui::{
    app::{
        context::Context,
        menu::Menu,
        message::Message,
        state::{DepositState, EmergencyState, HistoryState, VaultsState},
    },
    conversion::Converter,
    daemon::{
        client::{
            GetHistoryResponse, GetInfoResponse, ListOnchainTransactionsResponse,
            ListVaultsResponse, Request, RevaultD,
        },
        config::Config,
        model::{
            BroadcastedTransaction, DepositAddress, HistoryEvent, HistoryEventKind, Vault,
            VaultStatus, VaultTransactions,
        },
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

#[tokio::test]
async fn test_vaults_state() {
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
                VaultStatus::Securing,
                VaultStatus::Secured,
                VaultStatus::Activating,
                VaultStatus::Active,
                VaultStatus::Unvaulting,
                VaultStatus::Unvaulted,
                VaultStatus::Canceling,
                VaultStatus::EmergencyVaulting,
                VaultStatus::UnvaultEmergencyVaulting,
                VaultStatus::Spending,
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
                        vout: 0,
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
            Some(
                json!({"method": "listonchaintransactions", "params": Some(&[[
                        "a1075db55d416d3ca199f55b6084e2115b9345e16c5cf302fc80e9d5fbf5d48d:1"
                ]])}),
            ),
            Ok(json!(ListOnchainTransactionsResponse {
                onchain_transactions: vec![VaultTransactions {
                    vault_outpoint:
                        "a1075db55d416d3ca199f55b6084e2115b9345e16c5cf302fc80e9d5fbf5d48d:1"
                            .to_string(),
                    deposit: BroadcastedTransaction {
                        blockheight: Some(1),
                        tx: bitcoin::consensus::encode::deserialize(&Vec::from_hex("0200000001b4243a48b54cc360e754e0175a985a49b67cf4615d8523ec5aa46d42421cdf7d0000000000504200000280b2010000000000220020b9be8f8574f8da64bb1cb6668f6134bc4706df7936eeab8411f9d82de20a895b08280954020000000000000000").unwrap()).unwrap(),
                        received_at: 1,
                    },
                    unvault: None,
                    spend: None,
                    cancel: None,
                    emergency: None,
                    unvault_emergency: None,
                }]
            })),
        ),
    ]);

    let sandbox: Sandbox<DaemonClient, VaultsState> = Sandbox::new(VaultsState::new());

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
    assert!(matches!(sandbox.state(), VaultsState::Loaded { .. }));

    if let VaultsState::Loaded {
        vaults,
        selected_vault,
        ..
    } = sandbox.state()
    {
        assert!(selected_vault.is_none());
        assert_eq!(vaults.len(), 2);
    }

    let sandbox = sandbox
        .update(
            &ctx,
            Message::SelectVault(
                bitcoin::OutPoint {
                    txid: bitcoin::Txid::from_str(
                        "a1075db55d416d3ca199f55b6084e2115b9345e16c5cf302fc80e9d5fbf5d48d",
                    )
                    .unwrap(),
                    vout: 1,
                }
                .to_string(),
            ),
        )
        .await;
    assert!(matches!(sandbox.state(), VaultsState::Loaded { .. }));

    if let VaultsState::Loaded { selected_vault, .. } = sandbox.state() {
        assert!(selected_vault.is_some());
        if let Some(vault_state) = selected_vault {
            assert_eq!(
                vault_state.vault.txid,
                bitcoin::Txid::from_str(
                    "a1075db55d416d3ca199f55b6084e2115b9345e16c5cf302fc80e9d5fbf5d48d",
                )
                .unwrap()
            )
        }
    }
}

#[tokio::test]
async fn test_history_state() {
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
            None,
            Ok(json!(GetHistoryResponse {
                events: vec![
                    HistoryEvent {
                        date: 1,
                        txid: bitcoin::Txid::from_str(
                            "a1075db55d416d3ca199f55b6084e2115b9345e16c5cf302fc80e9d5fbf5d48d"
                        )
                        .unwrap(),
                        kind: HistoryEventKind::Spend,
                        amount: Some(1_000_000),
                        fee: Some(2000),
                    },
                    HistoryEvent {
                        date: 0,
                        txid: bitcoin::Txid::from_str(
                            "a1075db55d416d3ca199f55b6084e2115b9345e16c5cf302fc80e9d5fbf5d48d"
                        )
                        .unwrap(),
                        kind: HistoryEventKind::Spend,
                        amount: Some(2_000_000),
                        fee: None,
                    },
                ]
            })),
        ),
        (
            None,
            Ok(json!(GetHistoryResponse {
                events: vec![HistoryEvent {
                    date: 0,
                    txid: bitcoin::Txid::from_str(
                        "a1075db55d416d3ca199f55b6084e2115b9345e16c5cf302fc80e9d5fbf5d48d"
                    )
                    .unwrap(),
                    kind: HistoryEventKind::Spend,
                    amount: Some(2_000_000),
                    fee: None,
                },]
            })),
        ),
    ]);

    let sandbox: Sandbox<DaemonClient, HistoryState> = Sandbox::new(HistoryState::new());

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
    assert!(matches!(sandbox.state(), HistoryState::Loaded { .. }));

    if let HistoryState::Loaded {
        events,
        event_kind_filter,
        ..
    } = sandbox.state()
    {
        assert!(event_kind_filter.is_none());
        assert_eq!(events.len(), 2);
    }

    let sandbox = sandbox
        .update(
            &ctx,
            Message::FilterHistoryEvents(Some(HistoryEventKind::Deposit)),
        )
        .await;
    assert!(matches!(sandbox.state(), HistoryState::Loaded { .. }));

    if let HistoryState::Loaded {
        events,
        event_kind_filter,
        ..
    } = sandbox.state()
    {
        assert_eq!(*event_kind_filter, Some(HistoryEventKind::Deposit));
        assert_eq!(events.len(), 1);
    }
}
