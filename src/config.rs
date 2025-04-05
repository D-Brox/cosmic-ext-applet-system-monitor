// SPDX-License-Identifier: GPL-3.0-only

use crate::{
    applet::{Message, ID},
    color::Color,
};
use cosmic::{
    cosmic_config::{self, cosmic_config_derive::CosmicConfigEntry, CosmicConfigEntry},
    iced::Subscription,
};
use serde::{Deserialize, Serialize};

pub const CONFIG_VERSION: u64 = 2;

#[derive(Clone, CosmicConfigEntry, Debug, Deserialize, PartialEq, Serialize)]
pub struct Config {
    // todo radius goes here? should it be different for each view-type?
    pub charts: Vec<ChartConfig>,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            charts: vec![
                /*
                ChartConfig::CPU(Cpu {
                    update_interval: 1000,
                    color: Color::accent_blue,
                    samples: 60,
                    visualization: vec![
                        CpuView::GlobalUsageRunChart,
                        CpuView::PerCoreUsageHistogram,
                    ],
                }),
                */
                ChartConfig::Ram(Ram::default()),
                ChartConfig::Swap(Swap::default()),
                /*
                ChartConfig::Net(Network {
                    update_interval: 1000,
                    color_up: Color::accent_yellow,
                    color_down: Color::accent_red,
                    aspect_ratio: 1.5,
                    samples: 60,
                    visualization: vec![DoubleChartView::SuperimposedRunChart],
                }),
                ChartConfig::Disk(Disk {
                    update_interval: 2000,
                    color_read: Color::accent_orange,
                    color_write: Color::accent_pink,
                    aspect_ratio: 1.5,
                    samples: 30,
                    visualization: vec![DoubleChartView::SuperimposedRunChart],
                }),
                ChartConfig::VRAM(Generic {
                    update_interval: 2000,
                    color: Color::accent_indigo,
                    aspect_ratio: 1.5,
                    samples: 30,
                    visualization: vec![ChartView::RunChart],
                }),
                */
            ],
        }
    }
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub struct Cpu {
    pub update_interval: u64,
    pub samples: usize,
    pub color: Color,
    pub visualization: Vec<CpuView>,
}

#[allow(clippy::upper_case_acronyms)]
#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub enum ChartConfig {
    Ram(Ram),
    Swap(Swap),
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub struct Ram {
    pub update_interval: u64,
    pub history_size: u8,
    pub vis: Box<[SingleView]>,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub struct Swap {
    pub update_interval: u64,
    pub history_size: u8,
    pub vis: Box<[SingleView]>,
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

    pub visualization: Vec<DoubleView>,
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

    pub visualization: Vec<DoubleView>,
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

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub enum SingleView {
    // todo radius goes inside these?
    Bar { aspect_ratio: f32, color: Color },
    Run { aspect_ratio: f32, color: Color },
}

#[derive(Copy, Clone, Debug, Serialize, Deserialize, PartialEq)]
pub enum DoubleView {
    SuperimposedRunChart { aspect_ratio: f32 },
    // SeperateRunCharts,
}

#[derive(Copy, Clone, Debug, Serialize, Deserialize, PartialEq)]
pub enum CpuView {
    GlobalUsageRunChart,
    PerCoreUsageHistogram,
    GlobalUsageBarChart,
}

impl Default for Ram {
    fn default() -> Self {
        println!("using RAM's default config");
        let color = Color::accent_green;
        Ram {
            update_interval: 2000,
            history_size: 30,
            vis: [
                SingleView::Run {
                    aspect_ratio: 2.0,
                    color,
                },
                SingleView::Bar {
                    aspect_ratio: 0.5,
                    color,
                },
            ]
            .into(),
        }
    }
}

impl Default for Swap {
    fn default() -> Self {
        println!("using SWAP's default config");
        let color = Color::accent_purple;

        Swap {
            update_interval: 5000,
            history_size: 12,
            vis: [
                SingleView::Run {
                    aspect_ratio: 1.5,
                    color,
                },
                SingleView::Bar {
                    aspect_ratio: 0.5,
                    color,
                },
            ]
            .into(),
        }
    }
}
