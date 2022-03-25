mod utils;

use serde_json::json;
use std::path::PathBuf;
use std::str::FromStr;
use std::sync::Arc;

use utils::{fixtures::random_daemon_config, mock::Daemon, no_hardware_wallet, sandbox::Sandbox};

use bitcoin::{
    base64,
    consensus::encode,
    util::{bip32, psbt::PartiallySignedTransaction as Psbt},
    Address, Amount, OutPoint,
};

use revault_gui::{
    app::{
        config::Config as GUIConfig,
        context::{ConfigContext, Context},
        menu::Menu,
        message::{InputMessage, Message},
        state::manager::ManagerCreateSendTransactionState,
    },
    conversion::Converter,
    daemon::{
        client::{ListVaultsResponse, RevaultD, UnvaultTransaction},
        model::{Vault, VaultStatus},
    },
    revault::Role,
};

#[tokio::test]
async fn test_manager_create_spend() {
    let unvault: Psbt = encode::deserialize(&base64::decode("cHNidP8BAIkCAAAAAUeuD/NEqc88sk3DoBrKoVKjXbN2xW8Jr/4GO5q87JqJAQAAAAD9////AriGJgcAAAAAIgAgSOjPZes2prPdrcgiv+IG1sjXyTCc4KDr9+C9F+xk6LwwdQAAAAAAACIAIAjkMa8elv7dHUmYpDATWBtmMmpv9yyKFawMunvGQ1AMAAAAAAABASsADicHAAAAACIAIHXyaRd0yBZ3gxhGsCgiAOKIssWXELWPdDGD1JJVB9vFAQMEAQAAAAEFR1IhAlgt7b9E9GVk5djNsGdTbWDr40zR0YAc/1G7+desKJtDIQNHBN7LVbWqiP/R710GNmJIwTFOGWVRE2/xTquLukpJDlKuIgYCWC3tv0T0ZWTl2M2wZ1NtYOvjTNHRgBz/Ubv516wom0MI1n1/6QAAAAAiBgNHBN7LVbWqiP/R710GNmJIwTFOGWVRE2/xTquLukpJDghyqV8iAAAAAAAiAgICkzqxA36tCqSnhYxtSdZwXh+zvF9msAkYr3ufAOzVJgglHWAJAAAAACICAlgt7b9E9GVk5djNsGdTbWDr40zR0YAc/1G7+desKJtDCNZ9f+kAAAAAIgIDRwTey1W1qoj/0e9dBjZiSMExThllURNv8U6ri7pKSQ4IcqlfIgAAAAAAIgICUHL04HZXilyJ1B118e1Smr+S8c1qtja46Le7DzMCaUMI+93szQAAAAAA").unwrap()).unwrap();
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
            Some(json!({"method": "getunvaulttx", "params": Some(&[
                    OutPoint::from_str("a1075db55d416d3ca199f55b6084e2115b9345e16c5cf302fc80e9d5fbf5d48d:0").unwrap(),
            ])})),
            Ok(json!(UnvaultTransaction {
                unvault_tx: unvault.clone(),
            })),
        ),
        (
            Some(json!({"method": "getunvaulttx", "params": Some(&[
                    OutPoint::from_str("a1075db55d416d3ca199f55b6084e2115b9345e16c5cf302fc80e9d5fbf5d48d:1").unwrap(),
            ])})),
            Ok(json!(UnvaultTransaction {
                unvault_tx: unvault
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
