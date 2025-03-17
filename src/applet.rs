// SPDX-License-Identifier: GPL-3.0-only
use std::time::Duration;
use circular_queue::CircularQueue;
use crate::bar_chart::{self, SortMethod};
use crate::bar_chart::{BarConfig, Orientation};
use crate::sysmon::SystemMonitor;
use crate::cpu_widget::{self, CpuVisualization, CpuWidget};
use cosmic::app::{Core, Task};

use cosmic::iced::alignment::{Horizontal, Vertical};
use cosmic::iced::Subscription;
use cosmic::iced_widget::{column, row};
use cosmic::{iced, Also};
use cosmic::{cosmic_config, Application, Element, Theme};
use sysinfo::System;

use crate::config::{config_subscription, ChartConfig, Config, Generic};

// pub const CONFIG_VERSION: u64 = 1;
pub const ID: &str = "dev.DBrox.CosmicSystemMonitor";

pub struct SystemMonitorApplet {
    core: Core,
    config: Config,
    #[allow(dead_code)]
    config_handler: Option<cosmic_config::Config>,
    chart: SystemMonitor,
    sys: System,
    cpu_history: CircularQueue<i64>
}

#[derive(Debug, Clone)]
pub enum Message {
    Config(Config),
    TickCpu,
    TickRam,
    TickSwap,
    TickNet,
    TickDisk,
    // TickVRAM,
}

#[derive(Clone, Debug)]
pub struct Flags {
    pub config_handler: Option<cosmic_config::Config>,
    pub config: Config,
}

impl SystemMonitorApplet {
    fn get_theme(&self) -> Theme {
        self.core
            .applet
            .theme()
            .expect("Error: applet theme not found")
    }
}

impl Application for SystemMonitorApplet {
    type Executor = cosmic::executor::Default;

    type Flags = Flags;

    type Message = Message;

    const APP_ID: &'static str = ID;

    fn core(&self) -> &Core {
        &self.core
    }

    fn core_mut(&mut self) -> &mut Core {
        &mut self.core
    }

    fn init(core: Core, flags: Self::Flags) -> (Self, Task<Self::Message>) {
        let theme = core.applet.theme().expect("Error: applet theme not found");

        let mut sys = System::new();
        sys.refresh_cpu_usage(); // otherwise, sys.cpus().len == 0, meaning no bars will be drawn until the first refresh





        let app = Self {
            core,
            chart: SystemMonitor::new(&flags.config, &theme),
            config: flags.config,
            config_handler: flags.config_handler,
            sys,
            cpu_history: CircularQueue::with_capacity(40)

        };

        (app, Task::none())
    }

    fn view(&self) -> Element<Self::Message> {
        let (_, size) = self.core.applet.suggested_size(false);
        let pad = self.core.applet.suggested_padding(false);
        let is_horizontal = self.core.applet.is_horizontal();

        let config = BarConfig {
            full_length: size.into(),
            orientation: if is_horizontal {
                Orientation::PointingUp
            } else {
                Orientation::PointingRight
            },
            spacing: 5.0.into(),
            ..Default::default()
        };

        let generic_config = Generic {
            update_interval: 1000,
            samples: 60,
            color: crate::color::Color::bright_green,
            size: 1.5,
        };
        let children: Vec<Element<Message>> = Vec::from_iter([
            Element::from(CpuWidget::run_chart(generic_config.clone(), &self.sys, &self.core.applet, self.cpu_history.clone())).explain(iced::Color::WHITE),

            // self.chart.view(size.into(), pad.into(), is_horizontal),
            bar_chart::per_core_cpu_container(&self.sys.cpus(), config.clone(), generic_config.color.as_srgba(&self.get_theme())).into(),
            CpuWidget::global_bar(
                generic_config.clone(),
                &self.sys,
                &self.core.applet,
                size as f32,
                size as f32,
            )
            .into(),
            CpuWidget::multi_bar(generic_config.clone(), &self.sys, config, &self.core.applet).into(),
        ]);

        self.core
            .applet
            .autosize_window(if is_horizontal {
                Element::from(
                    row(children)
                        .align_y(Vertical::Center)
                        .padding([0, pad])
                        .spacing(pad),
                )
            } else {
                Element::from(
                    column(children)
                        .align_x(Horizontal::Center)
                        .padding([pad, 0])
                        .spacing(pad),
                )
            })
            .into()
    }

    fn update(&mut self, message: Self::Message) -> Task<Self::Message> {
        #[allow(unused_macros)]
        macro_rules! config_set {
            ($name: ident, $value: expr) => {
                match &self.config_handler {
                    Some(config_handler) => {
                        match paste::paste! { self.config.[<set_ $name>](config_handler, $value) } {
                            Ok(_) => {}
                            Err(err) => {
                                eprintln!("failed to save config {:?}: {}", stringify!($name), err);
                            }
                        }
                    }
                    None => {
                        self.config.$name = $value;
                        eprintln!(
                            "failed to save config {:?}: no config handler",
                            stringify!($name),
                        );
                    }
                }
            };
        }

        match message {
            Message::Config(config) => {
                if config != self.config {
                    self.config = config;
                    self.chart.update_config(&self.config, &self.get_theme());
                }
            }
            Message::TickCpu => {
                self.chart.update_cpu(&self.get_theme());
                self.sys.refresh_cpu_usage();
                self.cpu_history.push(self.sys.global_cpu_usage() as i64);
            }
            Message::TickRam => self.chart.update_ram(&self.get_theme()),
            Message::TickSwap => self.chart.update_swap(&self.get_theme()),
            Message::TickNet => self.chart.update_net(&self.get_theme()),
            Message::TickDisk => self.chart.update_disk(&self.get_theme()),
            // Message::TickVRAM => self.chart.update_vram(&self.get_theme()),
        }
        Task::none()
    }

    fn subscription(&self) -> Subscription<Self::Message> {
        let mut subs = Vec::new();
        for chart in &self.config.charts {
            let tick = {
                match chart {
                    ChartConfig::CPU(c) => {
                        cosmic::iced::time::every(Duration::from_millis(c.update_interval))
                            .map(|_| Message::TickCpu)
                    }
                    ChartConfig::RAM(c) => {
                        cosmic::iced::time::every(Duration::from_millis(c.update_interval))
                            .map(|_| Message::TickRam)
                    }
                    ChartConfig::Swap(c) => {
                        cosmic::iced::time::every(Duration::from_millis(c.update_interval))
                            .map(|_| Message::TickSwap)
                    }
                    ChartConfig::Net(c) => {
                        cosmic::iced::time::every(Duration::from_millis(c.update_interval))
                            .map(|_| Message::TickNet)
                    }
                    ChartConfig::Disk(c) => {
                        cosmic::iced::time::every(Duration::from_millis(c.update_interval))
                            .map(|_| Message::TickDisk)
                    }
                    ChartConfig::VRAM(_c) => {
                        // uninplemented
                        continue;
                        // cosmic::iced::time::every(Duration::from_millis(c.update_interval))
                        // .map(|_| Message::TickVRAM)
                    }
                }
            };
            subs.push(tick);
        }

        subs.push(config_subscription());

        Subscription::batch(subs)
    }

    fn style(&self) -> Option<cosmic::iced_runtime::Appearance> {
        Some(cosmic::applet::style())
    }
}
