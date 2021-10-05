mod utils;

use serde_json::json;
use std::str::FromStr;
use std::sync::Arc;

use utils::{
    mock::{DaemonClient, MockedRequests},
    sandbox::Sandbox,
};

use revault_gui::{
    app::{context::Context, menu::Menu, state::DepositState},
    conversion::Converter,
    daemon::{
        client::{GetInfoResponse, Request, RevaultD},
        config::Config,
        model::DepositAddress,
    },
    revault::Role,
};

#[tokio::test]
async fn test_deposit_state() {
    let address = bitcoin::Address::from_str(
        "tb1qkldgvljmjpxrjq2ev5qxe8dvhn0dph9q85pwtfkjeanmwdue2akqj4twxj",
    )
    .unwrap();
    let sandbox: Sandbox<DaemonClient, DepositState> = Sandbox::new(DepositState::new());
    let requests: MockedRequests = [
        (
            json!({"method": "getinfo", "params": Option::<Request>::None}),
            Ok(json!(GetInfoResponse {
                blockheight: 0,
                network: "testnet".to_string(),
                sync: 1.0,
                version: "0.1".to_string(),
                managers_threshold: 3
            })),
        ),
        (
            json!({"method": "getdepositaddress", "params": Option::<Request>::None}),
            Ok(json!(DepositAddress {
                address: address.clone()
            })),
        ),
    ]
    .iter()
    .cloned()
    .map(|(k, v)| (k.to_string(), v))
    .collect();

    let cfg = Config::default();
    let client = DaemonClient::new(requests);
    let ctx = Context::new(
        Arc::new(RevaultD::new(&cfg, client).unwrap()),
        Converter::new(bitcoin::Network::Bitcoin),
        bitcoin::Network::Bitcoin,
        false,
        Role::Stakeholder,
        Menu::Vaults,
        3,
    );

    let sandbox = sandbox.load(&ctx).await;

    assert_eq!(&address, sandbox.state().address.as_ref().unwrap());
}
