// SPDX-License-Identifier: GPL-3.0-only

mod applet;
mod components {
    pub mod bar;
    pub mod run;
}
mod color;
mod config;
mod history;
mod localization;

use applet::{Flags, SystemMonitorApplet, ID};
use config::{Config, CONFIG_VERSION};
use cosmic::cosmic_config::{Config as CosmicConfig, CosmicConfigEntry, Error as ConfigError};

fn main() -> cosmic::iced::Result {
    let (config_handler, config) = match CosmicConfig::new(ID, CONFIG_VERSION) {
        Ok(config_handler) => {
            let config = match Config::get_entry(&config_handler) {
                Ok(ok) => ok,
                Err((errs, config)) => {
                    eprintln!("errors loading config: {errs:?}");
                    if errs.iter().any(|err|!err.is_err()){
                        let _ = config.write_entry(&config_handler);
                    }
                    config
                }
            };
            (Some(config_handler), config)
        }
        Err(err) => {
            eprintln!("failed to create config handler: {err}");
            (None, Config::default())
        }
    };

    let flags = Flags {
        config_handler,
        config,
    };

    cosmic::applet::run::<SystemMonitorApplet>(flags)
}
