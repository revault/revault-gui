use std::env::VarError;
use std::path::PathBuf;
use std::str::FromStr;
mod revaultd;

extern crate serde;
extern crate serde_json;

/// logs everything on stdout
/// `./revault-gui > log.txt`
fn setup_logger(level: log::LevelFilter) -> Result<(), fern::InitError> {
    let dispatcher = fern::Dispatch::new()
        .format(|out, message, record| {
            out.finish(format_args!(
                "{}[{}][{}] {}",
                chrono::Local::now().format("[%Y-%m-%d][%H:%M:%S]"),
                record.target(),
                record.level(),
                message
            ))
        })
        .level(level);

    dispatcher.chain(std::io::stdout()).apply()?;

    Ok(())
}

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

    let varloglevel: Option<log::LevelFilter> = match std::env::var("REVAULTGUI_LOGLEVEL") {
        Ok(var) => match FromStr::from_str(&var) {
            Ok(v) => Some(v),
            Err(_) => {
                println!("Error: REVAULTGUI_LOGLEVEL must be 'OFF', 'ERROR', 'WARN', 'INFO', 'DEBUG', 'TRACE'");
                std::process::exit(1);
            }
        },
        Err(VarError::NotUnicode(_)) => {
            println!("Error: REVAULTGUI_LOGLEVEL must be 'OFF', 'ERROR', 'WARN', 'INFO', 'DEBUG', 'TRACE'");
            std::process::exit(1);
        }
        Err(VarError::NotPresent) => None,
    };

    let loglevel = varloglevel.unwrap_or_else(|| {
        if debug {
            log::LevelFilter::Debug
        } else {
            log::LevelFilter::Info
        }
    });

    if let Err(e) = setup_logger(loglevel) {
        println!("Error: failed to setup logger: {}", e.to_string());
        std::process::exit(1);
    };

    if let Some(p) = path {
        revaultd::RevaultD::new(p).unwrap();
    }
}
