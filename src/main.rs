use std::env::VarError;
use std::path::PathBuf;
mod revaultd;

extern crate serde;
extern crate serde_json;

fn main() {
    let path = match std::env::var("REVAULTD_CONF") {
        Ok(p) => Some(PathBuf::from(p)),
        Err(VarError::NotPresent) => None,
        Err(VarError::NotUnicode(_)) => {
            println!("Error: REVAULTD_CONF path has a wrong unicode format");
            std::process::exit(1);
        }
    };

    if let Some(p) = path {
        revaultd::RevaultD::new(p).unwrap();
    }
}
