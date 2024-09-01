// SPDX-License-Identifier: GPL-3.0-only

use std::time::Duration;

use crate::chart::SystemMonitorChart;
use cosmic::app::{Command, Core};

use cosmic::iced::Subscription;
use cosmic::iced_style::application;
use cosmic::widget;
use cosmic::{cosmic_config, Application, Element, Theme};

use crate::config::{config_subscription, ChartConfig, Config};

// pub const CONFIG_VERSION: u64 = 1;
pub const ID: &str = "dev.DBrox.CosmicSystemMonitor";

pub struct SystemMonitor {
    core: Core,
    config: Config,
    #[allow(dead_code)]
    config_handler: Option<cosmic_config::Config>,
    chart: SystemMonitorChart,
}

#[derive(Debug, Clone)]
pub enum Message {
    Config(Config),
    TickCpu,
    TickRam,
    TickSwap,
    TickNet,
    // TickDisk,
    // TickVRAM,
}

#[derive(Clone, Debug)]
pub struct Flags {
    pub config_handler: Option<cosmic_config::Config>,
    pub config: Config,
}

impl SystemMonitor {
    fn get_theme(&self) -> Theme {
        self.core
            .applet
            .theme()
            .expect("Error: applet theme not found")
    }
}

impl Application for SystemMonitor {
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

    fn init(core: Core, flags: Self::Flags) -> (Self, Command<Self::Message>) {
        let theme = core.applet.theme().expect("Error: applet theme not found");
        let app = SystemMonitor {
            core,
            chart: SystemMonitorChart::new(&flags.config, &theme),
            config: flags.config,
            config_handler: flags.config_handler,
        };

        (app, Command::none())
    }

    fn view(&self) -> Element<Self::Message> {
        let (_, height) = self.core.applet.suggested_size(false);
        widget::layer_container(self.chart.view(height.into())).into()
    }

    fn update(&mut self, message: Self::Message) -> Command<Self::Message> {
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
            Message::TickCpu => self.chart.update_cpu(&self.get_theme()),
            Message::TickRam => self.chart.update_ram(&self.get_theme()),
            Message::TickSwap => self.chart.update_swap(&self.get_theme()),
            Message::TickNet => self.chart.update_net(&self.get_theme()),
            // Message::TickDisk => self.chart.update_disk(&self.get_theme()),
            // Message::TickVRAM => self.chart.update_vram(&self.get_theme()),
        }
        Command::none()
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
                    ChartConfig::Disk(_c) => {
                        // uninplemented
                        continue;
                        // cosmic::iced::time::every(Duration::from_millis(c.update_interval))
                        // .map(|_| Message::TickDisk)
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

    fn style(&self) -> Option<<Theme as application::StyleSheet>::Style> {
        Some(cosmic::applet::style())
    }
}
