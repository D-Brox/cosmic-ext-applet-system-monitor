// SPDX-License-Identifier: GPL-3.0-only

#![allow(clippy::float_cmp)]
use cosmic::{
    cosmic_config::{self, cosmic_config_derive::CosmicConfigEntry, CosmicConfigEntry},
    iced::Subscription,
};
use serde::{Deserialize, Serialize};

use crate::{
    applet::{Message, ID},
    bar_chart::SortMethod,
    color::Color,
};
pub const CONFIG_VERSION: u64 = 2;

#[derive(Clone, CosmicConfigEntry, Debug, Deserialize, PartialEq, Serialize)]
pub struct Config {
    // todo radius goes here? should it be different for each view-type?
    pub padding: PaddingOption,
    pub sampling: SamplingConfig,
    pub components: Box<[ComponentConfig]>,
    pub component_spacing: f32,
    pub component_inner_spacing: f32,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub struct SamplingConfig {
    pub cpu: Sampling,
    pub mem: Sampling,
    pub net: Sampling,
    pub disk: Sampling,
    pub gpu: Sampling,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub struct Sampling {
    /// amount of time (in milliseconds) between new data
    pub update_interval: u64,
    /// size of the history kept and shown in the run chart
    pub sampling_window: usize,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub enum PaddingOption {
    Suggested,
    #[serde(untagged)]
    Custom(f32),
}

#[allow(clippy::upper_case_acronyms)]
#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub enum ComponentConfig {
    Cpu(Box<[CpuView]>),

    Mem(Box<[PercentView]>),
    Net(Box<[IoView]>),
    Disk(Box<[IoView]>),
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
/// Typically used for input-output pair
pub enum IoView {
    #[serde(rename = "RunChart")]
    Run {
        /// The `cosmic::palette` color to represent the relevant input (e.g. input = disk read rate, net download rate)
        #[serde(alias = "color_read", alias = "color_download")]
        color_back: Color,
        /// The `cosmic::palette` color to represent the relevant output (e.g. output = disk write rate, net upload rate)
        #[serde(alias = "color_write", alias = "color_upload")]
        color_front: Color,
        /// The **ratio** of width to height of the graph.
        aspect_ratio: f32,
    },
    /// If this is a view for some IO, A is for the system input (e.g. input = disk read rate, net download rate)
    #[serde(alias = "ReadRunChart", alias = "DownloadRunChart")]
    RunA { color: Color, aspect_ratio: f32 },
    /// If IO, B is the system output (e.g. output = disk write rate, net upload rate)
    #[serde(alias = "WriteRunChart", alias = "UploadRunChart")]
    RunB { color: Color, aspect_ratio: f32 },
}

#[derive(Copy, Clone, Debug, Serialize, Deserialize, PartialEq)]
pub enum CpuView {
    GlobalRun {
        color: Color,
        aspect_ratio: f32,
    },
    PerCoreBar {
        color: Color,
        spacing: f32,
        bar_aspect_ratio: f32,
        sorting: SortMethod,
    },
    GlobalBar {
        color: Color,
        aspect_ratio: f32,
    },
}

#[derive(Copy, Clone, Debug, Serialize, Deserialize, PartialEq)]
pub enum PercentView {
    #[serde(rename = "RunChart")]
    Run {
        #[serde(alias = "color_ram")]
        color_back: Color,
        #[serde(alias = "color_swap")]
        color_front: Color,
        aspect_ratio: f32,
    },
    #[serde(alias = "RamRunChart")]
    RunA { color: Color, aspect_ratio: f32 },
    #[serde(alias = "SwapRunChart")]
    RunB { color: Color, aspect_ratio: f32 },

    #[serde(rename = "BarChart")]
    Bar {
        #[serde(alias = "color_ram")]
        color_back: Color,
        #[serde(alias = "color_swap")]
        color_front: Color,
        spacing: f32,
        bar_aspect_ratio: f32,
    },
    #[serde(alias = "RamBarChart")]
    BarA { color: Color, aspect_ratio: f32 },
    #[serde(alias = "SwapBarChart")]
    BarB { color: Color, aspect_ratio: f32 },
}

impl Default for Config {
    fn default() -> Self {
        Self {
            padding: PaddingOption::Suggested,
            component_spacing: 10.0,
            component_inner_spacing: 2.5,
            components: [
                ComponentConfig::default_cpu(),
                ComponentConfig::default_mem(),
                ComponentConfig::default_disk(),
                ComponentConfig::default_net(),
                // ComponentConfig::default_gpu(),
            ]
            .into(),
            sampling: SamplingConfig::default(),
        }
    }
}

impl Default for SamplingConfig {
    fn default() -> Self {
        SamplingConfig {
            cpu: Sampling {
                update_interval: 1000,
                sampling_window: 60,
            },
            mem: Sampling {
                update_interval: 2000,
                sampling_window: 30,
            },
            net: Sampling {
                update_interval: 2000,
                sampling_window: 30,
            },
            disk: Sampling {
                update_interval: 1000,
                sampling_window: 60,
            },
            gpu: Sampling {
                update_interval: 2000,
                sampling_window: 30,
            },
        }
    }
}

impl ComponentConfig {
    fn default_cpu() -> Self {
        let color = Color::accent_blue;

        ComponentConfig::Cpu(
            [
                CpuView::GlobalRun {
                    aspect_ratio: 3.0,
                    color,
                },
                CpuView::PerCoreBar {
                    bar_aspect_ratio: 0.25,
                    color,
                    spacing: 3.0,
                    sorting: SortMethod::Unsorted,
                },
                CpuView::GlobalBar {
                    aspect_ratio: 0.5,
                    color,
                },
            ]
            .into(),
        )
    }

    fn default_mem() -> Self {
        let color_back = Color::accent_green;
        let color_front = Color::accent_purple;
        ComponentConfig::Mem(
            [
                PercentView::Run {
                    color_back,
                    color_front,
                    aspect_ratio: 2.0,
                },
                PercentView::Bar {
                    color_back,
                    color_front,
                    bar_aspect_ratio: 0.5,
                    spacing: 3.0,
                },
            ]
            .into(),
        )
    }

    fn default_net() -> Self {
        ComponentConfig::Net(
            [IoView::Run {
                color_front: Color::accent_yellow,
                color_back: Color::accent_red,
                aspect_ratio: 1.5,
            }]
            .into(),
        )
    }

    fn default_disk() -> Self {
        ComponentConfig::Disk(
            [IoView::Run {
                color_front: Color::accent_orange,
                color_back: Color::accent_pink,
                aspect_ratio: 1.5,
            }]
            .into(),
        )
    }

    // fn default_gpu() -> Self {
    //     let color_back = Color::accent_warm_grey;
    //     let color_front = Color::accent_indigo;
    //     ComponentConfig::Mem(
    //         [
    //             PercentView::Run {
    //                 color_back,
    //                 color_front,
    //                 aspect_ratio: 2.0,
    //             },
    //             PercentView::Bar {
    //                 color_back,
    //                 color_front,
    //                 bar_aspect_ratio: 0.5,
    //                 spacing: 3.0,
    //             },
    //         ]
    //         .into(),
    //     )
    // }
}
