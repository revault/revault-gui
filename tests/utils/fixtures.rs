use revaultd::config::Config as DaemonConfig;

pub fn random_daemon_config() -> DaemonConfig {
    toml::from_str(
r#"
coordinator_host = "127.0.0.1:8383"
coordinator_noise_key = "fa4aa4fd8bd5bc2746efff75a9e012305531f41f29557e88cef68e678dffab3a"
daemon = true
data_dir = "/home/edouard/code/revault/demo/demo-noel"

[bitcoind_config]
addr = "127.0.0.1:9002"
cookie_path = "/home/edouard/code/revault/demo/demo-noel/regtest/bcdir2/regtest/.cookie"
network = "regtest"

[manager_config]
cosigners = []
xpub = "tpubD6NzVbkrYhZ4XkehE7ghxNboGmT4Pd1SZ9RWLN5dG5vgRKXQgSxYtsmUgAYsqzdbK9petorBFceU36PNAfkVmrMhfNsJRSoiyWpu6NJA1BQ"

[scripts_config]
cpfp_descriptor = "wsh(multi(1,tpubD6NzVbkrYhZ4XkehE7ghxNboGmT4Pd1SZ9RWLN5dG5vgRKXQgSxYtsmUgAYsqzdbK9petorBFceU36PNAfkVmrMhfNsJRSoiyWpu6NJA1BQ/*,tpubD6NzVbkrYhZ4XyJXPpnkwCpTazWgerTFgXLtVehbPyoNKVFfPgXRcoxLGupEES1tSteVGsJon85AxEzGyWVSxm8LX8bdZsz87GWt585X2wf/*))#8h972ae2"
deposit_descriptor = "wsh(multi(2,tpubD6NzVbkrYhZ4WmzFjvQrp7sDa4ECUxTi9oby8K4FZkd3XCBtEdKwUiQyYJaxiJo5y42gyDWEczrFpozEjeLxMPxjf2WtkfcbpUdfvNnozWF/*,tpubD6NzVbkrYhZ4XyJXPpnkwCpTazWgerTFgXLtVehbPyoNKVFfPgXRcoxLGupEES1tSteVGsJon85AxEzGyWVSxm8LX8bdZsz87GWt585X2wf/*))#36w5x8qy"
unvault_descriptor = "wsh(andor(multi(1,tpubD6NzVbkrYhZ4XcB3kRJVob8bmjMvA2zBuagidVzh7ASY5FyAEtq4nTzx9wHYu5XDQAg7vdFNiF6yX38kTCK8zjVVmFTiQR2YKAqZBTGjnoD/*,tpubD6NzVbkrYhZ4XkehE7ghxNboGmT4Pd1SZ9RWLN5dG5vgRKXQgSxYtsmUgAYsqzdbK9petorBFceU36PNAfkVmrMhfNsJRSoiyWpu6NJA1BQ/*),older(10),thresh(2,pkh(tpubD6NzVbkrYhZ4WmzFjvQrp7sDa4ECUxTi9oby8K4FZkd3XCBtEdKwUiQyYJaxiJo5y42gyDWEczrFpozEjeLxMPxjf2WtkfcbpUdfvNnozWF/*),a:pkh(tpubD6NzVbkrYhZ4XyJXPpnkwCpTazWgerTFgXLtVehbPyoNKVFfPgXRcoxLGupEES1tSteVGsJon85AxEzGyWVSxm8LX8bdZsz87GWt585X2wf/*))))#lej6yrsc"

[stakeholder_config]
emergency_address = "bcrt1qqyds0grsuaxpx2dxg4ueugn4p6qyfg6lszmzert77yqh0m8ku3dqxragug"
watchtowers = []
xpub = "tpubD6NzVbkrYhZ4WmzFjvQrp7sDa4ECUxTi9oby8K4FZkd3XCBtEdKwUiQyYJaxiJo5y42gyDWEczrFpozEjeLxMPxjf2WtkfcbpUdfvNnozWF"
"#
    ).unwrap()
}
