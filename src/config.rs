// SPDX-License-Identifier: GPL-3.0-only

#![allow(clippy::float_cmp)]
use cosmic::{
    cosmic_config::{
        self, Config as CosmicConfig, ConfigGet, ConfigSet, CosmicConfigEntry, Error as ConfigError,
    },
    iced::Subscription,
};
use serde::{Deserialize, Serialize};

use crate::{
    applet::{Message, ID},
    color::Color,
    components::bar::SortMethod,
};
pub const CONFIG_VERSION: u64 = 2;

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub struct Config {
    // todo radius goes here? should it be different for each view-type?
    pub sampling: SamplingConfig,
    pub components: Box<[ComponentConfig]>,
    pub layout: LayoutConfig,
}

impl CosmicConfigEntry for Config {
    const VERSION: u64 = CONFIG_VERSION;
    fn write_entry(&self, config: &CosmicConfig) -> Result<(), ConfigError> {
        let tx = config.transaction();
        ConfigSet::set(&tx, "sampling", &self.sampling)?;
        ConfigSet::set(&tx, "components", &self.components)?;
        ConfigSet::set(&tx, "layout", &self.layout)?;
        tx.commit()
    }
    fn get_entry(config: &CosmicConfig) -> Result<Self, (Vec<ConfigError>, Self)> {
        let mut default = Self::default();
        let mut errors = Vec::new();

        macro_rules! config_get {
            ($field_name:ident,$field_type:ty) => {
                match ConfigGet::get::<$field_type>(config, stringify!($field_name)) {
                    Ok($field_name) => default.$field_name = $field_name,
                    Err(why) if !why.is_err() => {
                        let tx = config.transaction();
                        if let Ok(_) =
                            ConfigSet::set(&tx, stringify!($field_name), &default.$field_name)
                        {
                            _ = tx.commit();
                        }
                    }
                    Err(e) => errors.push(e),
                }
            };
        }
        config_get!(sampling, SamplingConfig);
        config_get!(components, Box<[ComponentConfig]>);
        config_get!(layout, LayoutConfig);

        if errors.is_empty() {
            Ok(default)
        } else {
            Err((errors, default))
        }
    }
    fn update_keys<T: AsRef<str>>(
        &mut self,
        config: &CosmicConfig,
        changed_keys: &[T],
    ) -> (Vec<ConfigError>, Vec<&'static str>) {
        let mut keys = Vec::with_capacity(changed_keys.len());
        let mut errors = Vec::new();

        macro_rules! config_set {
            ($field_name:ident,$field_type:ty) => {
                match cosmic_config::ConfigGet::get::<$field_type>(config, stringify!($field_name))
                {
                    Ok(value) => {
                        if self.$field_name != value {
                            keys.push(stringify!($field_name));
                        }
                        self.$field_name = value;
                    }
                    Err(e) => {
                        errors.push(e);
                    }
                }
            };
        }

        for key in changed_keys {
            match key.as_ref() {
                "sampling" => config_set!(sampling, SamplingConfig),
                "components" => config_set!(components, Box<[ComponentConfig]>),
                "layout" => config_set!(layout, LayoutConfig),
                _ => {}
            }
        }
        (errors, keys)
    }
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub struct LayoutConfig {
    pub padding: PaddingOption,
    pub spacing: f32,
    pub inner_spacing: f32,
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
    Gpu(Box<[PercentView]>),
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub struct Graphics {
    pub update_interval: u64,
    pub samples: usize,
    pub color_usage: Color,
    pub color_vram: Color,
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
            println!(
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
    #[serde(
        rename = "RunChartBack",
        alias = "RunChartRead",
        alias = "RunChartDownload"
    )]
    RunBack { color: Color, aspect_ratio: f32 },
    /// If IO, B is the system output (e.g. output = disk write rate, net upload rate)
    #[serde(
        rename = "RunChartFront",
        alias = "RunChartWrite",
        alias = "RunChartUpload"
    )]
    RunFront { color: Color, aspect_ratio: f32 },
}

#[derive(Copy, Clone, Debug, Serialize, Deserialize, PartialEq)]
pub enum CpuView {
    #[serde(rename = "RunChart")]
    Run {
        color: Color,
        aspect_ratio: f32,
    },
    BarGlobal {
        color: Color,
        aspect_ratio: f32,
    },
    BarCores {
        color: Color,
        spacing: f32,
        #[serde(alias = "bar_aspect_ratio")]
        aspect_ratio: f32,
        #[serde(default)]
        sorting: SortMethod,
    },
}

#[derive(Copy, Clone, Debug, Serialize, Deserialize, PartialEq)]
pub enum PercentView {
    #[serde(rename = "RunChart")]
    Run {
        #[serde(alias = "color_ram", alias = "color_usage")]
        color_back: Color,
        #[serde(alias = "color_swap", alias = "color_vram")]
        color_front: Color,
        aspect_ratio: f32,
    },
    #[serde(
        rename = "RunChartBack",
        alias = "RunChartRam",
        alias = "RunChartUsage"
    )]
    RunBack { color: Color, aspect_ratio: f32 },
    #[serde(
        rename = "RunChartFront",
        alias = "RunChartSwap",
        alias = "RunChartVram"
    )]
    RunFront { color: Color, aspect_ratio: f32 },

    #[serde(rename = "BarChart")]
    Bar {
        #[serde(alias = "color_ram", alias = "color_usage")]
        color_left: Color,
        #[serde(alias = "color_swap", alias = "color_vram")]
        color_right: Color,
        spacing: f32,
        aspect_ratio: f32,
    },
    #[serde(alias = "BarChartRam", alias = "BarChartUsage")]
    BarLeft { color: Color, aspect_ratio: f32 },
    #[serde(alias = "BarChartSwap", alias = "BarChartVram")]
    BarRight { color: Color, aspect_ratio: f32 },
}

impl Default for Config {
    fn default() -> Self {
        Self {
            layout: LayoutConfig::default(),
            components: [
                ComponentConfig::default_cpu(),
                ComponentConfig::default_mem(),
                ComponentConfig::default_disk(),
                ComponentConfig::default_net(),
                ComponentConfig::default_gpu(),
            ]
            .into(),
            sampling: SamplingConfig::default(),
        }
    }
}

impl Default for LayoutConfig {
    fn default() -> Self {
        Self {
            padding: PaddingOption::Suggested,
            spacing: 5.0,
            inner_spacing: 2.5,
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
                CpuView::Run {
                    aspect_ratio: 1.5,
                    color,
                },
                CpuView::BarGlobal {
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
                    aspect_ratio: 1.5,
                },
                PercentView::Bar {
                    color_left: color_back,
                    color_right: color_front,
                    aspect_ratio: 0.5,
                    spacing: 2.5,
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

    fn default_gpu() -> Self {
        let color_usage = Color::accent_warm_grey;
        let color_vram = Color::accent_indigo;
        ComponentConfig::Gpu(
            [
                PercentView::Run {
                    color_back: color_usage,
                    color_front: color_vram,
                    aspect_ratio: 1.5,
                },
                PercentView::Bar {
                    color_left: color_usage,
                    color_right: color_vram,
                    aspect_ratio: 0.5,
                    spacing: 2.5,
                },
            ]
            .into(),
        )
    }
}
