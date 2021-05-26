use std::error::Error;
use std::path::PathBuf;

use tracing_subscriber::filter::EnvFilter;
extern crate serde;
extern crate serde_json;

mod app;
mod conversion;
mod installer;
mod revault;
mod revaultd;
mod ui;

use app::config::{Config, ConfigError, DEFAULT_FILE_NAME};
use revaultd::config::default_datadir;

enum Args {
    ConfigPath(PathBuf),
    DatadirPath(PathBuf),
    None,
}

fn parse_args(args: Vec<String>) -> Result<Args, Box<dyn Error>> {
    if args.len() == 1 {
        return Ok(Args::None);
    }

    if args.len() == 3 {
        if args[1] == "--conf" {
            return Ok(Args::ConfigPath(PathBuf::from(args[2].to_owned())));
        } else if args[1] == "--datadir" {
            return Ok(Args::DatadirPath(PathBuf::from(args[2].to_owned())));
        }
    }

    println!("Usage:\n'--conf <configuration file path>'\n'--datadir <datadir path>'");
    Err(format!("Unknown arguments '{:?}'.", args).into())
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

    let config = match parse_args(args)? {
        Args::ConfigPath(path) => Config::from_file(&path)?,
        Args::None => {
            let path = Config::default_path()
                .map_err(|e| format!("Failed to find revault GUI config: {}", e))?;

            match Config::from_file(&path) {
                Ok(cfg) => cfg,
                Err(ConfigError::NotFound) => {
                    let default_datadir_path =
                        default_datadir().expect("Unexpected filesystem error");
                    if let Err(e) = installer::run(default_datadir_path) {
                        return Err(format!("Failed to install: {}", e).into());
                    };
                    Config::from_file(&path)?
                }
                Err(e) => {
                    return Err(format!("Failed to read configuration file: {}", e).into());
                }
            }
        }
        Args::DatadirPath(datadir_path) => {
            let mut path = datadir_path.clone();
            path.push(DEFAULT_FILE_NAME);
            match Config::from_file(&path) {
                Ok(cfg) => cfg,
                Err(ConfigError::NotFound) => {
                    if let Err(e) = installer::run(datadir_path) {
                        return Err(format!("Failed to install: {}", e).into());
                    };
                    Config::from_file(&path)?
                }
                Err(e) => {
                    return Err(format!("Failed to read configuration file: {}", e).into());
                }
            }
        }
    };

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
