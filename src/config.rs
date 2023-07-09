use crate::error::Error;

use serde::{Deserialize, Serialize};

const CONFIG_NAME: &'static str = "trss";

#[derive(Debug, Serialize, Deserialize)]
pub struct Config {
    pub subscriptions: Vec<String>,
}

/// `Config` implements `Default`
impl ::std::default::Default for Config {
    fn default() -> Self {
        Self {
            subscriptions: vec![],
        }
    }
}

pub(crate) fn store(config: &Config) -> Result<(), Error> {
    confy::store(env!("CARGO_CRATE_NAME"), CONFIG_NAME, config)?;
    Ok(())
}

pub(crate) fn load() -> Result<Config, Error> {
    let config = confy::load(env!("CARGO_CRATE_NAME"), CONFIG_NAME);

    match config {
        Ok(c) => Ok(c),
        Err(_) => {
            eprintln!("Error: config not loaded using the default config");
            let c = Default::default();
            store(&c)?;
            Ok(c)
        }
    }
}

pub(crate) fn update_or_store(website: String) -> Result<(), Error> {
    let mut config = load()?;
    config.subscriptions.push(website);

    store(&config)
}
