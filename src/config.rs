// SPDX-License-Identifier: GPL-3.0-only

use crate::{
    applet::{Message, ID},
    bar_chart::SortMethod,
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
    pub padding: PaddingOption,
    pub components: Box<[ComponentConfig]>,
    pub spacing_between_components: f32,
    pub spacing_within_component: f32,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub enum PaddingOption {
    Suggested,
    Custom(f32),
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub struct Cpu {
    /// amount of time (in milliseconds) between new data
    pub update_interval: u64,

    /// size of the history kept and shown in the run chart
    pub history_size: usize,
    pub vis: Box<[CpuView]>,
}

#[allow(clippy::upper_case_acronyms)]
#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub enum ComponentConfig {
    Cpu(Cpu),
    Ram(Ram),
    Swap(Swap),
    Net(Network),
    Disk(Disk),
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub struct Ram {
    /// amount of time (in milliseconds) between new data
    pub update_interval: u64,

    /// size of the history kept and shown in the run chart
    pub history_size: usize,
    pub vis: Box<[SingleView]>,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub struct Swap {
    /// amount of time (in milliseconds) between new data
    pub update_interval: u64,

    /// size of the history kept and shown in the run chart
    pub history_size: usize,
    pub vis: Box<[SingleView]>,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub struct Network {
    /// amount of time (in milliseconds) between new data
    pub update_interval: u64,

    /// size of the history kept and shown in the run chart
    pub history_size: usize,
    pub vis: Box<[DoubleView]>,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub struct Disk {
    /// amount of time (in milliseconds) between new data
    pub update_interval: u64,

    /// size of the history kept and shown in the run chart
    pub history_size: usize,
    pub vis: Vec<DoubleView>,
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
/// Typically used for input-output pair
pub enum DoubleView {
    SuperimposedRunChart {
        /// The `cosmic::pallette` color to represent the relevant input (e.g. input = disk read rate, net download rate)
        color_in: Color,
        /// The `cosmic::pallette` color to represent the relevant output (e.g. output = disk write rate, net upload rate)
        color_out: Color,
        /// The **ratio** of width to height of the graph.
        aspect_ratio: f32,
    },
    /// If this is a view for some IO, A is for the system input (e.g. input = disk read rate, net download rate)
    SingleRunA { color: Color, aspect_ratio: f32 },
    /// If IO, B is the system output (e.g. output = disk write rate, net upload rate)
    SingleRunB { color: Color, aspect_ratio: f32 },
    // SeperateRunCharts,
}

#[derive(Copy, Clone, Debug, Serialize, Deserialize, PartialEq)]
pub enum CpuView {
    GlobalUsageRunChart {
        aspect_ratio: f32,
        color: Color,
    },
    PerCoreUsageHistogram {
        per_core_aspect_ratio: f32,
        color: Color,
        spacing: f32,
        sorting: Option<SortMethod>,
    },
    GlobalUsageBarChart {
        aspect_ratio: f32,
        color: Color,
    },
}

impl Default for Config {
    fn default() -> Self {
        Self {
            padding: PaddingOption::Suggested,
            spacing_between_components: 15.0,
            spacing_within_component: 5.0,
            components: [
                ComponentConfig::Cpu(Cpu::default()),
                ComponentConfig::Ram(Ram::default()),
                ComponentConfig::Swap(Swap::default()),
                ComponentConfig::Net(Network::default()),
                ComponentConfig::Disk(Disk::default()),
                // ChartConfig::VRAM(VRAM::default()),
            ]
            .into(),
        }
    }
}

impl Default for Cpu {
    fn default() -> Self {
        let color = Color::accent_blue;

        Cpu {
            update_interval: 1000,
            history_size: 60,
            vis: [
                CpuView::GlobalUsageRunChart {
                    aspect_ratio: 3.0,
                    color,
                },
                CpuView::PerCoreUsageHistogram {
                    per_core_aspect_ratio: 0.25,
                    color,
                    spacing: 3.0,
                    sorting: Some(SortMethod::Descending),
                },
                CpuView::GlobalUsageBarChart {
                    aspect_ratio: 0.5,
                    color,
                },
            ]
            .into(),
        }
    }
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

impl Default for Network {
    fn default() -> Self {
        Network {
            update_interval: 1000,
            history_size: 60,
            vis: [DoubleView::SuperimposedRunChart {
                color_out: Color::accent_yellow,
                color_in: Color::accent_red,
                aspect_ratio: 1.5,
            }]
            .into(),
        }
    }
}

impl Default for Disk {
    fn default() -> Self {
        Disk {
            update_interval: 2000,
            history_size: 30,
            vis: [DoubleView::SuperimposedRunChart {
                color_out: Color::accent_orange,
                color_in: Color::accent_pink,
                aspect_ratio: 1.5,
            }]
            .into(),
        }
    }
}
