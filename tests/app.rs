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
        state::{
            history::HISTORY_EVENT_PAGE_SIZE, DepositState, EmergencyState, HistoryState,
            VaultsState,
        },
    },
    conversion::Converter,
    daemon::{
        client::{Request, RevaultD},
        model::{
            DepositAddress, GetHistoryResponse, HistoryEvent, HistoryEventKind,
            ListOnchainTransactionsResponse, ListVaultsResponse, Vault, VaultStatus,
            VaultTransactions, WalletTransaction, ALL_HISTORY_EVENTS,
        },
    },
    revault::Role,
};

#[tokio::test]
async fn test_deposit_state() {
    let addr = Address::from_str("tb1qkldgvljmjpxrjq2ev5qxe8dvhn0dph9q85pwtfkjeanmwdue2akqj4twxj")
        .unwrap();
    let daemon = Daemon::new(vec![(
        Some(json!({"method": "getdepositaddress", "params": Option::<Request>::None})),
        Ok(json!(DepositAddress {
            address: addr.clone()
        })),
    )]);

    let sandbox: Sandbox<DepositState> = Sandbox::new(DepositState::new());

    let client = daemon.run();
    let ctx = Context::new(
        ConfigContext {
            daemon: random_daemon_config(),
            gui: GUIConfig::new(PathBuf::from_str("revault_gui.toml").unwrap()),
        },
        Arc::new(RevaultD::new(client)),
        Converter::new(bitcoin::Network::Bitcoin),
        Role::Stakeholder,
        Menu::Vaults,
        false,
        Box::new(|| Box::pin(no_hardware_wallet())),
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
            Some(json!({"method": "listvaults", "params": Some(&[[
                VaultStatus::Secured.to_string(),
                VaultStatus::Active.to_string(),
                VaultStatus::Activating.to_string(),
                VaultStatus::Unvaulting.to_string(),
                VaultStatus::Unvaulted.to_string(),
                VaultStatus::EmergencyVaulting.to_string(),
                VaultStatus::EmergencyVaulted.to_string(),
                VaultStatus::UnvaultEmergencyVaulting.to_string(),
                VaultStatus::UnvaultEmergencyVaulted.to_string(),
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
                        status: VaultStatus::Secured,
                        txid: bitcoin::Txid::from_str(
                            "a1075db55d416d3ca199f55b6084e2115b9345e16c5cf302fc80e9d5fbf5d48d"
                        )
                        .unwrap(),
                        vout: 1,
                        blockheight: 1,
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
                        status: VaultStatus::Secured,
                        txid: bitcoin::Txid::from_str(
                            "a1075db55d416d3ca199f55b6084e2115b9345e16c5cf302fc80e9d5fbf5d48d"
                        )
                        .unwrap(),
                        vout: 1,
                        blockheight: 1,
                        delegated_at: None,
                        secured_at: Some(1),
                        funded_at: Some(1),
                        moved_at: None
                    }
                ]
            })),
        ),
        (
            Some(json!({"method": "emergency", "params": Option::<Request>::None})),
            Ok(json!({})),
        ),
    ]);

    let sandbox: Sandbox<EmergencyState> = Sandbox::new(EmergencyState::new());

    let client = daemon.run();
    let ctx = Context::new(
        ConfigContext {
            daemon: random_daemon_config(),
            gui: GUIConfig::new(PathBuf::from_str("revault_gui.toml").unwrap()),
        },
        Arc::new(RevaultD::new(client)),
        Converter::new(bitcoin::Network::Bitcoin),
        Role::Stakeholder,
        Menu::Vaults,
        false,
        Box::new(|| Box::pin(no_hardware_wallet())),
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
            Some(json!({"method": "listvaults", "params": Some(&[[
                VaultStatus::Securing.to_string(),
                VaultStatus::Secured.to_string(),
                VaultStatus::Activating.to_string(),
                VaultStatus::Active.to_string(),
                VaultStatus::Unvaulting.to_string(),
                VaultStatus::Unvaulted.to_string(),
                VaultStatus::Canceling.to_string(),
                VaultStatus::EmergencyVaulting.to_string(),
                VaultStatus::UnvaultEmergencyVaulting.to_string(),
                VaultStatus::Spending.to_string(),
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
                        status: VaultStatus::Secured,
                        txid: bitcoin::Txid::from_str(
                            "a1075db55d416d3ca199f55b6084e2115b9345e16c5cf302fc80e9d5fbf5d48d"
                        )
                        .unwrap(),
                        vout: 0,
                        blockheight: 1,
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
                        status: VaultStatus::Secured,
                        txid: bitcoin::Txid::from_str(
                            "a1075db55d416d3ca199f55b6084e2115b9345e16c5cf302fc80e9d5fbf5d48d"
                        )
                        .unwrap(),
                        vout: 1,
                        blockheight: 1,
                        delegated_at: None,
                        secured_at: Some(1),
                        funded_at: Some(1),
                        moved_at: None
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
                    vault_outpoint: OutPoint::from_str(
                        "a1075db55d416d3ca199f55b6084e2115b9345e16c5cf302fc80e9d5fbf5d48d:1"
                            ).unwrap(),
                    deposit: WalletTransaction {
                        blockheight: Some(1),
                        hex: "0200000001b4243a48b54cc360e754e0175a985a49b67cf4615d8523ec5aa46d42421cdf7d0000000000504200000280b2010000000000220020b9be8f8574f8da64bb1cb6668f6134bc4706df7936eeab8411f9d82de20a895b08280954020000000000000000".to_string(),
                        received_time: 1,
                        blocktime: Some(1)
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

    let sandbox: Sandbox<VaultsState> = Sandbox::new(VaultsState::new());

    let client = daemon.run();
    let ctx = Context::new(
        ConfigContext {
            daemon: random_daemon_config(),
            gui: GUIConfig::new(PathBuf::from_str("revault_gui.toml").unwrap()),
        },
        Arc::new(RevaultD::new(client)),
        Converter::new(bitcoin::Network::Bitcoin),
        Role::Stakeholder,
        Menu::Vaults,
        false,
        Box::new(|| Box::pin(no_hardware_wallet())),
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
            Message::SelectVault(bitcoin::OutPoint {
                txid: bitcoin::Txid::from_str(
                    "a1075db55d416d3ca199f55b6084e2115b9345e16c5cf302fc80e9d5fbf5d48d",
                )
                .unwrap(),
                vout: 1,
            }),
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
async fn test_history_state_filter() {
    let daemon = Daemon::new(vec![
        (
            None,
            Ok(json!(GetHistoryResponse {
                events: vec![
                    HistoryEvent {
                        blockheight: 1,
                        date: 1,
                        txid: bitcoin::Txid::from_str(
                            "a1075db55d416d3ca199f55b6084e2115b9345e16c5cf302fc80e9d5fbf5d48d"
                        )
                        .unwrap(),
                        kind: HistoryEventKind::Spend,
                        amount: Some(1_000_000),
                        fee: Some(2000),
                        vaults: Vec::new()
                    },
                    HistoryEvent {
                        blockheight: 0,
                        date: 0,
                        txid: bitcoin::Txid::from_str(
                            "a1075db55d416d3ca199f55b6084e2115b9345e16c5cf302fc80e9d5fbf5d48d"
                        )
                        .unwrap(),
                        kind: HistoryEventKind::Spend,
                        amount: Some(2_000_000),
                        fee: None,
                        vaults: Vec::new()
                    },
                ]
            })),
        ),
        (
            None,
            Ok(json!(GetHistoryResponse {
                events: vec![HistoryEvent {
                    blockheight: 0,
                    date: 0,
                    txid: bitcoin::Txid::from_str(
                        "a1075db55d416d3ca199f55b6084e2115b9345e16c5cf302fc80e9d5fbf5d48d"
                    )
                    .unwrap(),
                    kind: HistoryEventKind::Spend,
                    amount: Some(2_000_000),
                    fee: None,
                    vaults: Vec::new()
                },]
            })),
        ),
    ]);

    let sandbox: Sandbox<HistoryState> = Sandbox::new(HistoryState::new());

    let client = daemon.run();
    let ctx = Context::new(
        ConfigContext {
            daemon: random_daemon_config(),
            gui: GUIConfig::new(PathBuf::from_str("revault_gui.toml").unwrap()),
        },
        Arc::new(RevaultD::new(client)),
        Converter::new(bitcoin::Network::Bitcoin),
        Role::Stakeholder,
        Menu::Vaults,
        false,
        Box::new(|| Box::pin(no_hardware_wallet())),
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

#[tokio::test]
async fn test_history_state_pagination() {
    let mut events: Vec<HistoryEvent> = Vec::new();
    for i in 0..25 {
        events.push(HistoryEvent {
            blockheight: i,
            date: i as u32,
            txid: bitcoin::Txid::from_str(
                "a1075db55d416d3ca199f55b6084e2115b9345e16c5cf302fc80e9d5fbf5d48d",
            )
            .unwrap(),
            kind: HistoryEventKind::Deposit,
            amount: Some(1_000_000),
            fee: None,
            vaults: vec![bitcoin::OutPoint {
                txid: bitcoin::Txid::from_str(
                    "a1075db55d416d3ca199f55b6084e2115b9345e16c5cf302fc80e9d5fbf5d48d",
                )
                .unwrap(),
                vout: i,
            }],
        });
    }
    let daemon = Daemon::new(vec![
        (
            // SystemTime::now() is used, so we cannot check the request correctness for the
            // moment.
            None,
            Ok(json!(GetHistoryResponse {
                events: events[0..20].to_vec()
            })),
        ),
        (
            Some(
                json!({"method": "gethistory", "params": Some(&[json!(&ALL_HISTORY_EVENTS), json!(0 as u32), json!(19 as u32), json!(HISTORY_EVENT_PAGE_SIZE)])}),
            ),
            Ok(json!(GetHistoryResponse {
                events: events[20..25].to_vec()
            })),
        ),
        (
            // SystemTime::now() is used, so we cannot check the request correctness for the
            // moment.
            None,
            Ok(json!(GetHistoryResponse {
                events: events[0..20].to_vec()
            })),
        ),
        (
            Some(
                json!({"method": "gethistory", "params": Some(&[json!(&[HistoryEventKind::Deposit]), json!(0 as u32), json!(19 as u32), json!(HISTORY_EVENT_PAGE_SIZE)])}),
            ),
            Ok(json!(GetHistoryResponse {
                events: events[20..25].to_vec()
            })),
        ),
    ]);

    let sandbox: Sandbox<HistoryState> = Sandbox::new(HistoryState::new());

    let client = daemon.run();
    let ctx = Context::new(
        ConfigContext {
            daemon: random_daemon_config(),
            gui: GUIConfig::new(PathBuf::from_str("revault_gui.toml").unwrap()),
        },
        Arc::new(RevaultD::new(client)),
        Converter::new(bitcoin::Network::Bitcoin),
        Role::Stakeholder,
        Menu::Vaults,
        false,
        Box::new(|| Box::pin(no_hardware_wallet())),
    );

    let sandbox = sandbox.load(&ctx).await;
    assert!(matches!(sandbox.state(), HistoryState::Loaded { .. }));

    if let HistoryState::Loaded {
        events, has_next, ..
    } = sandbox.state()
    {
        assert_eq!(events.len() as u64, HISTORY_EVENT_PAGE_SIZE);
        assert!(has_next);
    }

    let sandbox = sandbox.update(&ctx, Message::Next).await;
    assert!(matches!(sandbox.state(), HistoryState::Loaded { .. }));

    if let HistoryState::Loaded {
        events, has_next, ..
    } = sandbox.state()
    {
        assert_eq!(events.len() as u64, 25);
        assert!(!has_next);
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
        has_next,
        ..
    } = sandbox.state()
    {
        assert_eq!(*event_kind_filter, Some(HistoryEventKind::Deposit));
        assert_eq!(events.len(), 20);
        assert!(has_next);
    }

    let sandbox = sandbox.update(&ctx, Message::Next).await;
    assert!(matches!(sandbox.state(), HistoryState::Loaded { .. }));

    if let HistoryState::Loaded {
        events,
        event_kind_filter,
        has_next,
        ..
    } = sandbox.state()
    {
        assert_eq!(*event_kind_filter, Some(HistoryEventKind::Deposit));
        assert_eq!(events.len() as u64, 25);
        assert!(!has_next);
    }
}

/// Test the case in which a big batch of history events with the size superior
/// to the HISTORY_EVENT_PAGE_SIZE happened in the same block (with the same blocktime).
#[tokio::test]
async fn test_history_state_pagination_batching() {
    let mut events: Vec<HistoryEvent> = Vec::new();
    for i in 0..65 {
        events.push(HistoryEvent {
            blockheight: 1,
            date: 1,
            txid: bitcoin::Txid::from_str(
                "a1075db55d416d3ca199f55b6084e2115b9345e16c5cf302fc80e9d5fbf5d48d",
            )
            .unwrap(),
            kind: HistoryEventKind::Deposit,
            amount: Some(1_000_000),
            fee: None,
            vaults: vec![bitcoin::OutPoint {
                txid: bitcoin::Txid::from_str(
                    "a1075db55d416d3ca199f55b6084e2115b9345e16c5cf302fc80e9d5fbf5d48d",
                )
                .unwrap(),
                vout: i,
            }],
        });
    }
    let daemon = Daemon::new(vec![
        (
            None,
            Ok(json!(GetHistoryResponse {
                events: events[0..20].to_vec()
            })),
        ),
        (
            Some(
                json!({"method": "gethistory", "params": Some(&[json!(&ALL_HISTORY_EVENTS), json!(0 as u32), json!(1 as u32), json!(HISTORY_EVENT_PAGE_SIZE)])}),
            ),
            Ok(json!(GetHistoryResponse {
                events: events[20..40].to_vec()
            })),
        ),
        (
            Some(
                json!({"method": "gethistory", "params": Some(&[json!(&ALL_HISTORY_EVENTS), json!(0 as u32), json!(1 as u32), json!(HISTORY_EVENT_PAGE_SIZE*2)])}),
            ),
            Ok(json!(GetHistoryResponse {
                events: events[20..60].to_vec()
            })),
        ),
        (
            Some(
                json!({"method": "gethistory", "params": Some(&[json!(&ALL_HISTORY_EVENTS), json!(0 as u32), json!(1 as u32), json!(HISTORY_EVENT_PAGE_SIZE*3)])}),
            ),
            Ok(json!(GetHistoryResponse {
                events: events[20..65].to_vec()
            })),
        ),
    ]);

    let sandbox: Sandbox<HistoryState> = Sandbox::new(HistoryState::new());

    let client = daemon.run();
    let ctx = Context::new(
        ConfigContext {
            daemon: random_daemon_config(),
            gui: GUIConfig::new(PathBuf::from_str("revault_gui.toml").unwrap()),
        },
        Arc::new(RevaultD::new(client)),
        Converter::new(bitcoin::Network::Bitcoin),
        Role::Stakeholder,
        Menu::Vaults,
        false,
        Box::new(|| Box::pin(no_hardware_wallet())),
    );

    let sandbox = sandbox.load(&ctx).await;
    assert!(matches!(sandbox.state(), HistoryState::Loaded { .. }));

    if let HistoryState::Loaded {
        events, has_next, ..
    } = sandbox.state()
    {
        assert_eq!(events.len() as u64, HISTORY_EVENT_PAGE_SIZE);
        assert!(has_next);
    }

    let sandbox = sandbox.update(&ctx, Message::Next).await;
    assert!(matches!(sandbox.state(), HistoryState::Loaded { .. }));

    if let HistoryState::Loaded {
        events, has_next, ..
    } = sandbox.state()
    {
        assert_eq!(events.len() as u64, 65);
        assert!(!has_next);
    }
}

#[tokio::test]
async fn test_history_state_select_event() {
    let oupoint = bitcoin::OutPoint {
        txid: bitcoin::Txid::from_str(
            "a1075db55d416d3ca199f55b6084e2115b9345e16c5cf302fc80e9d5fbf5d48d",
        )
        .unwrap(),
        vout: 1,
    };
    let daemon = Daemon::new(vec![
        (
            None,
            Ok(json!(GetHistoryResponse {
                events: vec![
                    HistoryEvent {
                        blockheight: 1,
                        date: 1,
                        txid: bitcoin::Txid::from_str(
                            "a1075db55d416d3ca199f55b6084e2115b9345e16c5cf302fc80e9d5fbf5d48d"
                        )
                        .unwrap(),
                        kind: HistoryEventKind::Spend,
                        amount: Some(1_000_000),
                        fee: Some(2000),
                        vaults: vec!(bitcoin::OutPoint {
                            txid: bitcoin::Txid::from_str(
                                "a1075db55d416d3ca199f55b6084e2115b9345e16c5cf302fc80e9d5fbf5d48d",
                            )
                            .unwrap(),
                            vout: 0,
                        })
                    },
                    HistoryEvent {
                        blockheight: 0,
                        date: 0,
                        txid: bitcoin::Txid::from_str(
                            "a1075db55d416d3ca199f55b6084e2115b9345e16c5cf302fc80e9d5fbf5d48d"
                        )
                        .unwrap(),
                        kind: HistoryEventKind::Spend,
                        amount: Some(2_000_000),
                        fee: None,
                        vaults: vec!(oupoint.clone())
                    },
                ]
            })),
        ),
        (
            Some(json!({"method": "listonchaintransactions", "params": [[oupoint.clone()]]})),
            Ok(json!(ListOnchainTransactionsResponse {
                onchain_transactions: vec![VaultTransactions {
                    vault_outpoint: oupoint,
                    deposit: WalletTransaction {
                        blockheight: Some(1),
                        hex: "0200000001b4243a48b54cc360e754e0175a985a49b67cf4615d8523ec5aa46d42421cdf7d0000000000504200000280b2010000000000220020b9be8f8574f8da64bb1cb6668f6134bc4706df7936eeab8411f9d82de20a895b08280954020000000000000000".to_string(),
                        received_time: 1,
                        blocktime: Some(1),
                    },
                    unvault: None,
                    spend: None,
                    cancel: None,
                    emergency: None,
                    unvault_emergency: None,
                }],
            })),
        )
    ]);

    let sandbox: Sandbox<HistoryState> = Sandbox::new(HistoryState::new());

    let client = daemon.run();
    let ctx = Context::new(
        ConfigContext {
            daemon: random_daemon_config(),
            gui: GUIConfig::new(PathBuf::from_str("revault_gui.toml").unwrap()),
        },
        Arc::new(RevaultD::new(client)),
        Converter::new(bitcoin::Network::Bitcoin),
        Role::Stakeholder,
        Menu::Vaults,
        false,
        Box::new(|| Box::pin(no_hardware_wallet())),
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

    let sandbox = sandbox.update(&ctx, Message::SelectHistoryEvent(1)).await;
    assert!(matches!(sandbox.state(), HistoryState::Loaded { .. }));

    if let HistoryState::Loaded { selected_event, .. } = sandbox.state() {
        assert!(selected_event.is_some());
    }

    let sandbox = sandbox.update(&ctx, Message::Close).await;
    assert!(matches!(sandbox.state(), HistoryState::Loaded { .. }));

    if let HistoryState::Loaded { selected_event, .. } = sandbox.state() {
        assert!(selected_event.is_none());
    }
}
