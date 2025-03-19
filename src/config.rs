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
                ChartConfig::CPU(Cpu {
                    update_interval: 1000,
                    color: Color::accent_blue,
                    size: 1.5,
                    samples: 60,
                    visualization: CpuView::GlobalUsageRunChart,
                }),
                ChartConfig::RAM(Generic {
                    update_interval: 2000,
                    color: Color::accent_green,
                    size: 1.5,
                    samples: 30,
                    visualization: ChartView::RunChart,
                }),
                ChartConfig::Swap(Generic {
                    update_interval: 5000,
                    color: Color::accent_purple,
                    size: 1.5,
                    samples: 12,
                    visualization: ChartView::RunChart,
                }),
                ChartConfig::Net(Network {
                    update_interval: 1000,
                    color_up: Color::accent_yellow,
                    color_down: Color::accent_red,
                    size: 1.5,
                    samples: 60,
                    visualization: ChartView::RunChart,
                }),
                ChartConfig::Disk(Disk {
                    update_interval: 2000,
                    color_read: Color::accent_orange,
                    color_write: Color::accent_pink,
                    size: 1.5,
                    samples: 30,
                    visualization: ChartView::RunChart,
                }),
                ChartConfig::VRAM(Generic {
                    update_interval: 2000,
                    color: Color::accent_indigo,
                    size: 1.5,
                    samples: 30,
                    visualization: ChartView::RunChart,
                }),
            ],
        }
    }
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub struct Cpu {
    pub update_interval: u64,
    pub samples: usize,
    pub color: Color,
    pub size: f32, // todo: `size` is never used?
    pub visualization: CpuView,
}

#[allow(clippy::upper_case_acronyms)]
#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub enum ChartConfig {
    CPU(Cpu),
    RAM(Generic),
    Swap(Generic),
    Net(Network),
    Disk(Disk),
    VRAM(Generic),
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub struct Generic {
    pub update_interval: u64,
    pub samples: usize,
    pub color: Color,
    pub size: f32, // todo: `size` is never used?
    pub visualization: ChartView,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub struct Network {
    pub update_interval: u64,
    pub samples: usize,
    pub color_up: Color,
    pub color_down: Color,
    pub size: f32,
    pub visualization: ChartView,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub struct Disk {
    pub update_interval: u64,
    pub samples: usize,
    pub color_read: Color,
    pub color_write: Color,
    pub size: f32,
    pub visualization: ChartView,
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

#[derive(Copy, Clone, Debug, Serialize, Deserialize, PartialEq)]
pub enum ChartView {
    RunChart,
    BarChart,
}

#[derive(Copy, Clone, Debug, Serialize, Deserialize, PartialEq)]
pub enum CpuView {
    GlobalUsageRunChart,
    PerCoreUsageHistogram,
    GlobalUsageBarChart,
}
