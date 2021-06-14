use std::error::Error;
use std::path::PathBuf;

use tracing_subscriber::filter::EnvFilter;
extern crate serde;
extern crate serde_json;

mod app;
mod conversion;
mod revault;
mod revaultd;
mod ui;

use app::config::Config;

fn config_path_from_args(args: Vec<String>) -> Result<Option<PathBuf>, Box<dyn Error>> {
    if args.len() == 1 {
        return Ok(None);
    }

    if args.len() != 3 || args[1] != "--conf" {
        println!("Usage: '--conf <configuration file path>'");
        return Err(format!("Unknown arguments '{:?}'.", args).into());
    }

    Ok(Some(PathBuf::from(args[2].to_owned())))
}

fn log_level_from_config(config: &Config) -> Result<EnvFilter, Box<dyn Error>> {
    if let Some(level) = &config.log_level {
        match level.as_ref() {
            "info" => EnvFilter::try_new("revault_gui=info").map_err(|e| e.into()),
            "debug" => EnvFilter::try_new("revault_gui=debug").map_err(|e| e.into()),
            "trace" => EnvFilter::try_new("revault_gui=trace").map_err(|e| e.into()),
            _ => Err(format!("Unknown loglevel '{:?}'.", level).into()),
        }
    } else if let Some(true) = config.debug {
        EnvFilter::try_new("revault_gui=debug").map_err(|e| e.into())
    } else {
        EnvFilter::try_new("revault_gui=info").map_err(|e| e.into())
    }
}

fn main() -> Result<(), Box<dyn Error>> {
    let args = std::env::args().collect();

    let config_path = if let Some(path) = config_path_from_args(args)? {
        path
    } else {
        Config::default_path().map_err(|e| format!("Failed to find revault GUI config: {}", e))?
    };

    let config = Config::from_file(&config_path)
        .map_err(|e| format!("Failed to read {:?}, {}", &config_path, e))?;

    tracing::subscriber::set_global_default(
        tracing_subscriber::FmtSubscriber::builder()
            .with_env_filter(log_level_from_config(&config)?)
            .finish(),
    )?;

    if let Err(e) = app::run(config) {
        return Err(format!("Failed to launch UI: {}", e).into());
    };
    Ok(())
}
