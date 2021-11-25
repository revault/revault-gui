mod api;
mod app;
mod config;
mod server;
mod sign;
mod view;

use std::env;
use std::path::PathBuf;
use std::process;
use std::str::FromStr;

use revault_tx::miniscript::descriptor::DescriptorSecretKey;

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        eprintln!(
            "Usage:\n{} <xpriv>  <xpriv>...\n{} --conf <config path>",
            args[0], args[0]
        );
        process::exit(1);
    }

    let cfg = if args[1] == "--conf" || args[1] == "-c" {
        let path = &args[2];
        match config::Config::from_file(&PathBuf::from(path)) {
            Ok(cfg) => cfg,
            Err(e) => {
                eprintln!("{}", e);
                process::exit(1);
            }
        }
    } else {
        let mut keys = Vec::new();
        for arg in &args[1..] {
            let key = match DescriptorSecretKey::from_str(arg) {
                Ok(DescriptorSecretKey::XPrv(xpriv)) => xpriv.xkey,
                _ => {
                    eprintln!("{} is not a xpriv", arg);
                    process::exit(1);
                }
            };
            keys.push(key);
        }
        config::Config::new(keys)
    };

    if let Err(e) = app::run(cfg) {
        println!("{}", e);
    }
}
