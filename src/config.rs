// SPDX-License-Identifier: GPL-3.0-only

use cosmic::{
    cosmic_config,
    cosmic_config::{cosmic_config_derive::CosmicConfigEntry, CosmicConfigEntry},
    iced::Subscription,
};

use serde::{Deserialize, Serialize};

use crate::{
    applet::{Message, ID},
    color::Color,
};

pub const CONFIG_VERSION: u64 = 1;

#[derive(Clone, CosmicConfigEntry, Debug, Deserialize, PartialEq, Serialize)]
pub struct Config {
    pub charts: Vec<ChartConfig>,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            charts: vec![
                ChartConfig::CPU(Generic {
                    update_interval: 1000,
                    color: Color::accent_blue,
                    aspect_ratio: 1.5,
                    samples: 60,
                }),
                ChartConfig::RAM(Generic {
                    update_interval: 2000,
                    color: Color::accent_green,
                    aspect_ratio: 1.5,
                    samples: 30,
                }),
                ChartConfig::Swap(Generic {
                    update_interval: 5000,
                    color: Color::accent_purple,
                    aspect_ratio: 1.5,
                    samples: 12,
                }),
                ChartConfig::Net(Network {
                    update_interval: 1000,
                    color_up: Color::accent_yellow,
                    color_down: Color::accent_red,
                    aspect_ratio: 1.5,
                    samples: 60,
                }),
                ChartConfig::Disk(Disk {
                    update_interval: 2000,
                    color_read: Color::accent_orange,
                    color_write: Color::accent_pink,
                    aspect_ratio: 1.5,
                    samples: 30,
                }),
                ChartConfig::VRAM(Generic {
                    update_interval: 2000,
                    color: Color::accent_indigo,
                    aspect_ratio: 1.5,
                    samples: 30,
                }),
            ],
        }
    }
}

#[allow(clippy::upper_case_acronyms)]
#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub enum ChartConfig {
    CPU(Generic),
    RAM(Generic),
    Swap(Generic),
    Net(Network),
    Disk(Disk),
    VRAM(Generic),
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub struct Generic {
    /// amount of time (in milliseconds) between new data
    pub update_interval: u64,

    /// size of the history kept and shown in the run chart
    pub samples: usize,

    /// The [Color] to use for this color this graph line.
    pub color: Color,

    /// The **ratio** of width to height of the graph.
    pub aspect_ratio: f32,

}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub struct Network {
    /// amount of time (in milliseconds) between new data
    pub update_interval: u64,

    /// size of the history kept and shown in the run chart
    pub samples: usize,

    /// The `cosmic::pallette` color to represent upload rate
    pub color_up: Color,

    /// The `cosmic::pallette` color to represent download rate
    pub color_down: Color,

    /// The **ratio** of width to height of the graph.
    pub aspect_ratio: f32,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub struct Disk {
    /// amount of time (in milliseconds) between new data
    pub update_interval: u64,

    /// size of the history kept and shown in the run chart
    pub samples: usize,

    /// The `cosmic::pallette` color to represent disk read rate
    pub color_read: Color,

    /// The `cosmic::pallette` color to represent disk write rate
    pub color_write: Color,

    /// The **ratio** of width to height of the graph.
    pub aspect_ratio: f32,
}

pub fn config_subscription() -> Subscription<Message> {
    struct ConfigSubscription;
    cosmic_config::config_subscription(
        std::any::TypeId::of::<ConfigSubscription>(),
        ID.into(),
        CONFIG_VERSION,
    )
    .map(|update| {
        if !update.errors.is_empty() {
            eprintln!(
                "errors loading config {:?}: {:?}",
                update.keys, update.errors
            );
        }
        Message::Config(update.config)
    })
}
