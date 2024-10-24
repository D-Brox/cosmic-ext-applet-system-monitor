// SPDX-License-Identifier: GPL-3.0-only

mod app;
mod chart;
mod color;
mod config;
mod localization;

use app::{Flags, SystemMonitor, ID};
use config::{Config, CONFIG_VERSION};
use cosmic::cosmic_config::{self, CosmicConfigEntry};

fn main() -> cosmic::iced::Result {
    let (config_handler, config) = match cosmic_config::Config::new(ID, CONFIG_VERSION) {
        Ok(config_handler) => {
            let config = match Config::get_entry(&config_handler) {
                Ok(ok) => ok,
                Err((errs, config)) => {
                    eprintln!("errors loading config: {errs:?}");
                    config
                }
            };
            if let Err(err) = config.write_entry(&config_handler) {
                eprintln!("Error writing config: {err:?}");
            }
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

    cosmic::applet::run::<SystemMonitor>(flags)
}
