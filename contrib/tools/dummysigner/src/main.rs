mod api;
mod app;
mod server;
mod sign;
mod view;

use std::env;
use std::process;
use std::str::FromStr;

use revault_tx::miniscript::descriptor::DescriptorSecretKey;

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        eprintln!("Usage: '{} <xpriv>  <xpriv>...", args[0]);
        process::exit(1);
    }

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

    if let Err(e) = app::run(app::Config { keys }) {
        println!("{}", e);
    }
}
