use std::env::VarError;
use std::path::PathBuf;
use std::str::FromStr;

use tracing_subscriber::filter::EnvFilter;
extern crate serde;
extern crate serde_json;

mod revault;
mod revaultd;
mod ui;

fn main() {
    let path = match std::env::var("REVAULTD_CONF") {
        Ok(p) => Some(PathBuf::from(p)),
        Err(VarError::NotPresent) => None,
        Err(VarError::NotUnicode(_)) => {
            println!("Error: REVAULTD_CONF path has a wrong unicode format");
            std::process::exit(1);
        }
    };

    let debug = match std::env::var("REVAULTGUI_DEBUG") {
        Ok(var) => match FromStr::from_str(&var) {
            Ok(v) => v,
            Err(_) => {
                println!("Error: REVAULTGUI_DEBUG must be `false` or `true`");
                std::process::exit(1);
            }
        },
        Err(VarError::NotUnicode(_)) => {
            println!("Error: REVAULTGUI_DEBUG must be `false` or `true`");
            std::process::exit(1);
        }
        Err(VarError::NotPresent) => false,
    };

    let logfilter: EnvFilter = match std::env::var("REVAULTGUI_LOG") {
        Ok(var) => match EnvFilter::try_new(var) {
            Ok(v) => v,
            Err(_) => {
                println!(
                    "Error: REVAULTGUI_LOG must follow tracing directive like `revaultd=info`"
                );
                std::process::exit(1);
            }
        },
        Err(VarError::NotUnicode(_)) => {
            println!("Error: REVAULTGUI_LOG unicode only");
            std::process::exit(1);
        }
        Err(VarError::NotPresent) => {
            if debug {
                EnvFilter::try_new("revault_gui=debug").unwrap()
            } else {
                EnvFilter::try_new("revault_gui=info").unwrap()
            }
        }
    };

    let subscriber = tracing_subscriber::FmtSubscriber::builder()
        .with_env_filter(logfilter)
        .finish();

    if let Err(e) = tracing::subscriber::set_global_default(subscriber) {
        println!("unexpected: {}", e);
        std::process::exit(1);
    }

    if let Err(e) = ui::app::run(ui::app::Config {
        revaultd_config_path: path,
        debug,
    }) {
        println!("Error: failed to launch UI: {}", e.to_string());
        std::process::exit(1);
    };
}
