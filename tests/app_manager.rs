mod utils;

use serde_json::json;
use std::path::PathBuf;
use std::str::FromStr;
use std::sync::Arc;

use utils::{fixtures::random_daemon_config, mock::Daemon, no_hardware_wallet, sandbox::Sandbox};

use bitcoin::{base64, util::bip32, Address, Amount, OutPoint};

use revaultd::revault_tx::transactions::{CancelTransaction, UnvaultTransaction};

use revault_gui::{
    app::{
        config::Config as GUIConfig,
        context::{ConfigContext, Context},
        menu::Menu,
        message::{InputMessage, Message, SpendTxMessage},
        state::manager::{ManagerCreateSendTransactionState, ManagerImportSendTransactionState},
    },
    conversion::Converter,
    daemon::{
        client::{ListPresignedTransactionsResponse, ListVaultsResponse, RevaultD},
        model::{Vault, VaultPresignedTransactions, VaultStatus},
    },
    revault::Role,
};
use revaultd::revault_tx::transactions::SpendTransaction;

#[tokio::test]
async fn test_manager_create_spend() {
    let unvault = UnvaultTransaction::from_raw_psbt(&base64::decode("cHNidP8BAIkCAAAAAUeuD/NEqc88sk3DoBrKoVKjXbN2xW8Jr/4GO5q87JqJAQAAAAD9////AriGJgcAAAAAIgAgSOjPZes2prPdrcgiv+IG1sjXyTCc4KDr9+C9F+xk6LwwdQAAAAAAACIAIAjkMa8elv7dHUmYpDATWBtmMmpv9yyKFawMunvGQ1AMAAAAAAABASsADicHAAAAACIAIHXyaRd0yBZ3gxhGsCgiAOKIssWXELWPdDGD1JJVB9vFAQMEAQAAAAEFR1IhAlgt7b9E9GVk5djNsGdTbWDr40zR0YAc/1G7+desKJtDIQNHBN7LVbWqiP/R710GNmJIwTFOGWVRE2/xTquLukpJDlKuIgYCWC3tv0T0ZWTl2M2wZ1NtYOvjTNHRgBz/Ubv516wom0MI1n1/6QAAAAAiBgNHBN7LVbWqiP/R710GNmJIwTFOGWVRE2/xTquLukpJDghyqV8iAAAAAAAiAgICkzqxA36tCqSnhYxtSdZwXh+zvF9msAkYr3ufAOzVJgglHWAJAAAAACICAlgt7b9E9GVk5djNsGdTbWDr40zR0YAc/1G7+desKJtDCNZ9f+kAAAAAIgIDRwTey1W1qoj/0e9dBjZiSMExThllURNv8U6ri7pKSQ4IcqlfIgAAAAAAIgICUHL04HZXilyJ1B118e1Smr+S8c1qtja46Le7DzMCaUMI+93szQAAAAAA").unwrap()).unwrap();
    let cancel = CancelTransaction::from_raw_psbt(&base64::decode("cHNidP8BAF4CAAAAATdzv51EXeeNc1fv6E852OhRxc67KNaWd+BrA3qN1a/1AAAAAAD9////ARRLJgcAAAAAIgAgdfJpF3TIFneDGEawKCIA4oiyxZcQtY90MYPUklUH28UAAAAAAAEBK7iGJgcAAAAAIgAgSOjPZes2prPdrcgiv+IG1sjXyTCc4KDr9+C9F+xk6LwBAwSBAAAAAQVhIQICkzqxA36tCqSnhYxtSdZwXh+zvF9msAkYr3ufAOzVJqxRh2R2qRRyqV8ir5obrrhS+alScvjCHZjyZIisa3apFLbJrbicjJNybIPiobXZR4nXe5VhiKxsk1KHZ1iyaCIGAgKTOrEDfq0KpKeFjG1J1nBeH7O8X2awCRive58A7NUmCCUdYAkAAAAAIgYCWC3tv0T0ZWTl2M2wZ1NtYOvjTNHRgBz/Ubv516wom0MI1n1/6QAAAAAiBgNHBN7LVbWqiP/R710GNmJIwTFOGWVRE2/xTquLukpJDghyqV8iAAAAAAAiAgJYLe2/RPRlZOXYzbBnU21g6+NM0dGAHP9Ru/nXrCibQwjWfX/pAAAAACICA0cE3stVtaqI/9HvXQY2YkjBMU4ZZVETb/FOq4u6SkkOCHKpXyIAAAAAAA==").unwrap()).unwrap();
    let cancels = [
        cancel.clone(),
        cancel.clone(),
        cancel.clone(),
        cancel.clone(),
        cancel,
    ];
    let daemon = Daemon::new(vec![
        (
            Some(json!({"method": "listvaults", "params": Some(&[[
                VaultStatus::Active.to_string(),
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
                        status: VaultStatus::Active,
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
                        status: VaultStatus::Active,
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
                ]
            })),
        ),
        (
            Some(
                json!({"method": "listpresignedtransactions", "params": Some(&[[
                        OutPoint::from_str("a1075db55d416d3ca199f55b6084e2115b9345e16c5cf302fc80e9d5fbf5d48d:0").unwrap(),
                        OutPoint::from_str("a1075db55d416d3ca199f55b6084e2115b9345e16c5cf302fc80e9d5fbf5d48d:1").unwrap(),
                ]])}),
            ),
            Ok(json!(ListPresignedTransactionsResponse {
                presigned_transactions: vec![
                    VaultPresignedTransactions {
                        vault_outpoint: OutPoint::from_str(
                            "a1075db55d416d3ca199f55b6084e2115b9345e16c5cf302fc80e9d5fbf5d48d:0"
                        )
                        .unwrap(),
                        unvault: unvault.clone(),
                        cancel: cancels.clone(),
                        emergency: None,
                        unvault_emergency: None
                    },
                    VaultPresignedTransactions {
                        vault_outpoint: OutPoint::from_str(
                            "a1075db55d416d3ca199f55b6084e2115b9345e16c5cf302fc80e9d5fbf5d48d:1"
                        )
                        .unwrap(),
                        unvault,
                        cancel: cancels,
                        emergency: None,
                        unvault_emergency: None
                    }
                ]
            })),
        ),
    ]);

    let sandbox: Sandbox<ManagerCreateSendTransactionState> =
        Sandbox::new(ManagerCreateSendTransactionState::new());

    let client = daemon.run();
    let ctx = Context::new(
        ConfigContext {
            daemon: random_daemon_config(),
            gui: GUIConfig::new(PathBuf::from_str("revault_gui.toml").unwrap()),
        },
        Arc::new(RevaultD::new(client)),
        Converter::new(bitcoin::Network::Bitcoin),
        Role::Stakeholder,
        Menu::DelegateFunds,
        Box::new(|| Box::pin(no_hardware_wallet())),
    );

    let sandbox = sandbox.load(&ctx).await;
    let state = sandbox.state();
    assert_eq!(state.input_amount(), 0);

    let sandbox = sandbox
        .update(&ctx, Message::Input(1, InputMessage::Select))
        .await;
    assert_eq!(sandbox.state().input_amount(), 119965368);
}

#[tokio::test]
async fn test_manager_import_spend() {
    let psbt_string = "cHNidP8BAIkCAAAAAUeuD/NEqc88sk3DoBrKoVKjXbN2xW8Jr/4GO5q87JqJAQAAAAD9////AriGJgcAAAAAIgAgSOjPZes2prPdrcgiv+IG1sjXyTCc4KDr9+C9F+xk6LwwdQAAAAAAACIAIAjkMa8elv7dHUmYpDATWBtmMmpv9yyKFawMunvGQ1AMAAAAAAABASsADicHAAAAACIAIHXyaRd0yBZ3gxhGsCgiAOKIssWXELWPdDGD1JJVB9vFAQMEAQAAAAEFR1IhAlgt7b9E9GVk5djNsGdTbWDr40zR0YAc/1G7+desKJtDIQNHBN7LVbWqiP/R710GNmJIwTFOGWVRE2/xTquLukpJDlKuIgYCWC3tv0T0ZWTl2M2wZ1NtYOvjTNHRgBz/Ubv516wom0MI1n1/6QAAAAAiBgNHBN7LVbWqiP/R710GNmJIwTFOGWVRE2/xTquLukpJDghyqV8iAAAAAAAiAgICkzqxA36tCqSnhYxtSdZwXh+zvF9msAkYr3ufAOzVJgglHWAJAAAAACICAlgt7b9E9GVk5djNsGdTbWDr40zR0YAc/1G7+desKJtDCNZ9f+kAAAAAIgIDRwTey1W1qoj/0e9dBjZiSMExThllURNv8U6ri7pKSQ4IcqlfIgAAAAAAIgICUHL04HZXilyJ1B118e1Smr+S8c1qtja46Le7DzMCaUMI+93szQAAAAAA";
    let spend = SpendTransaction::from_raw_psbt(&base64::decode(psbt_string).unwrap()).unwrap();

    let daemon = Daemon::new(vec![(
        Some(json!({"method": "updatespendtx", "params":
            Some(vec![
                psbt_string
            ])
        })),
        Ok(json!({})),
    )]);

    let sandbox: Sandbox<ManagerImportSendTransactionState> =
        Sandbox::new(ManagerImportSendTransactionState::new());

    let client = daemon.run();
    let ctx = Context::new(
        ConfigContext {
            daemon: random_daemon_config(),
            gui: GUIConfig::new(PathBuf::from_str("revault_gui.toml").unwrap()),
        },
        Arc::new(RevaultD::new(client)),
        Converter::new(bitcoin::Network::Bitcoin),
        Role::Stakeholder,
        Menu::DelegateFunds,
        Box::new(|| Box::pin(no_hardware_wallet())),
    );
    let sandbox = sandbox
        .update(
            &ctx,
            Message::SpendTx(SpendTxMessage::PsbtEdited(psbt_string.to_string())),
        )
        .await;

    let sandbox = sandbox
        .update(&ctx, Message::SpendTx(SpendTxMessage::Import))
        .await;

    let state: &ManagerImportSendTransactionState = sandbox.state();
    assert_eq!(state.imported_state().as_ref().unwrap(), &spend);
}
