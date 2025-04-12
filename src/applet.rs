use crate::helpers::{
    base_background, collection, get_sized_aspect_ratio, init_history_with_default,
};
use crate::run_chart::{HistoryChart, SuperimposedHistoryChart};
// use crate::sysmon::bar_chart::VerticalPercentageBar;
// SPDX-License-Identifier: GPL-3.0-only
// use crate::sysmon::SystemMonitor;
use circular_queue::CircularQueue;
use cosmic::app::{Core, Task};
use cosmic::iced::Length::Fixed;
use cosmic::iced_core::padding;
// use cosmic::cosmic_theme::palette::named::WHITE;
// use cosmic::widget::aspect_ratio::aspect_ratio_container;
use cosmic::widget::container;
use plotters_iced::ChartWidget;
use std::time::Duration;
use sysinfo::{Disk, Disks, MemoryRefreshKind, Networks, System};

use crate::bar_chart::VerticalPercentageBar;
use crate::config::{config_subscription, ChartConfig, Config, DoubleView, SingleView};
use cosmic::iced::{Size, Subscription};
use cosmic::{cosmic_config, Application, Apply as _, Element, Theme};

pub type History<T = u64> = CircularQueue<T>;

pub const ID: &str = "dev.DBrox.CosmicSystemMonitor";

pub struct SystemMonitorApplet {
    core: Core,
    config: Config,
    #[allow(dead_code)]
    config_handler: Option<cosmic_config::Config>,
    // chart: SystemMonitor,
    sys: System,
    nets: Networks,
    disks: Disks,
    // gpus: Gpus,
    /// global usage usage, range [0-100]
    // global_cpu: History<f32>,
    ram: History,
    swap: History,
    upload: History,
    download: History,
    disk_read: History,
    disk_write: History,
    // disk_read: History,
    // disk_write: History,
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
        self.core.applet.theme().unwrap_or_default()
    }

    fn swap_percentage(&self) -> f32 {
        self.sys.used_swap() as f32 / self.sys.total_swap() as f32 * 100.0
    }
}

impl Application for SystemMonitorApplet {
    type Executor = cosmic::executor::Default;

    type Flags = Flags;

    type Message = Message;

    const APP_ID: &'static str = ID; // todo inline ID, moving config_subscription to impl SystemMonitorApplet

    fn core(&self) -> &Core {
        &self.core
    }

    fn core_mut(&mut self) -> &mut Core {
        &mut self.core
    }

    fn init(core: Core, flags: Self::Flags) -> (Self, Task<Self::Message>) {
        let (mut ram, mut swap, mut net, mut disk) = Default::default();
        for chart_config in &flags.config.charts {
            match chart_config {
                // ChartConfig::Cpu()
                ChartConfig::Ram(c) => ram = Some(c.history_size),
                ChartConfig::Swap(c) => swap = Some(c.history_size),
                ChartConfig::Net(c) => net = Some(c.history_size),
                ChartConfig::Disk(c) => disk = Some(c.history_size),
            }
        }

        let app = Self {
            core,
            config: flags.config,
            config_handler: flags.config_handler,

            sys: System::new(),
            nets: Networks::new_with_refreshed_list(),
            disks: Disks::new_with_refreshed_list(),

            // global_cpu: init_history_with_default(0),
            ram: init_history_with_default(ram.unwrap_or(0)),
            swap: init_history_with_default(swap.unwrap_or(0)),
            upload: init_history_with_default(net.unwrap_or(0)),
            download: init_history_with_default(net.unwrap_or(0)),
            disk_read: init_history_with_default(disk.unwrap_or(0)),
            disk_write: init_history_with_default(disk.unwrap_or(0)),
        };

        (app, Task::none())
    }

    fn view(&self) -> Element<Message> {
        const INTRA_ITEM_SPACING: u16 = 5;
        let item_iter = self.config.charts.iter().map(|module| {
            match module {
                ChartConfig::Ram(c) => c
                    .vis
                    .iter()
                    .map(|v| match v {
                        SingleView::Bar {
                            aspect_ratio,
                            color,
                        } => {
                            let Size { width, height } =
                                get_sized_aspect_ratio(&self.core.applet, *aspect_ratio);

                            VerticalPercentageBar::new(
                                self.sys.used_memory() as f32 / self.sys.total_memory() as f32
                                    * 100.0,
                                *color,
                            )
                            // .apply(|bar| aspect_ratio_container(bar, *aspect_ratio))
                            .apply(container)
                            .style(base_background)
                            .width(width)
                            .height(height)
                            .apply(Element::new)
                        }
                        SingleView::Run {
                            aspect_ratio,
                            color,
                        } => {
                            let Size { width, height } =
                                get_sized_aspect_ratio(&self.core.applet, *aspect_ratio);
                            let chart = HistoryChart {
                                history: &self.ram,
                                max: self.sys.total_memory(),
                                color: color.as_rgba_color(self.get_theme()),
                            };

                            ChartWidget::new(chart)
                                .width(Fixed(width))
                                .height(Fixed(height))
                                .apply(container)
                                .style(base_background)
                                // .width(width)
                                // .height(height)
                                .apply(Element::new)
                        }
                    })
                    .collect::<Vec<_>>(),
                ChartConfig::Swap(c) => c
                    .vis
                    .iter()
                    .map(|v| match v {
                        SingleView::Bar {
                            aspect_ratio,
                            color,
                        } => {
                            let Size { width, height } =
                                get_sized_aspect_ratio(&self.core.applet, *aspect_ratio);

                            VerticalPercentageBar::new(self.swap_percentage(), *color)
                                // .apply(|bar| aspect_ratio_container(bar, *aspect_ratio))
                                .apply(container)
                                .style(base_background)
                                .width(width)
                                .height(height)
                                .apply(Element::new)
                        }
                        SingleView::Run {
                            aspect_ratio,
                            color,
                        } => {
                            let Size { width, height } =
                                get_sized_aspect_ratio(&self.core.applet, *aspect_ratio);
                            let chart = HistoryChart {
                                history: &self.swap,
                                max: self.sys.total_swap(),
                                color: color.as_rgba_color(self.get_theme()),
                            };
                            ChartWidget::new(chart)
                                .width(Fixed(width))
                                .height(Fixed(height))
                                .apply(container)
                                .style(base_background)
                                .apply(Element::new)
                        }
                    })
                    .collect(),
                ChartConfig::Net(c) => c
                    .vis
                    .iter()
                    .map(|v| {
                        match v {
                            DoubleView::SuperimposedRunChart {
                                aspect_ratio,
                                color_send: color_up,
                                color_receive: color_down,
                            } => {
                                let Size { width, height } =
                                    get_sized_aspect_ratio(&self.core.applet, *aspect_ratio);

                                let upload = HistoryChart::auto_max(
                                    &self.upload,
                                    color_up.as_rgba_color(self.get_theme()),
                                );
                                let download = HistoryChart::auto_max(
                                    &self.download,
                                    color_down.as_rgba_color(self.get_theme()),
                                );

                                ChartWidget::new(SuperimposedHistoryChart {
                                    h1: upload,
                                    h2: download,
                                })
                                // .apply(|bar| aspect_ratio_container(bar, *aspect_ratio))
                                .apply(container)
                                .padding(padding::top(height / 5.0))
                                .style(base_background)
                                .width(width)
                                .height(height)
                                .apply(Element::new)
                            }
                            DoubleView::SingleRunA {
                                color,
                                aspect_ratio,
                            } => {
                                let Size { width, height } =
                                    get_sized_aspect_ratio(&self.core.applet, *aspect_ratio);

                                let down = HistoryChart::auto_max(
                                    &self.download,
                                    color.as_rgba_color(self.get_theme()),
                                );

                                ChartWidget::new(down)
                                    .apply(container)
                                    .padding(padding::top(height / 5.0))
                                    .style(base_background)
                                    .width(width)
                                    .height(height)
                                    .apply(Element::new)
                            }
                            DoubleView::SingleRunB {
                                color,
                                aspect_ratio,
                            } => {
                                let Size { width, height } =
                                    get_sized_aspect_ratio(&self.core.applet, *aspect_ratio);

                                let up = HistoryChart::auto_max(
                                    &self.upload,
                                    color.as_rgba_color(self.get_theme()),
                                );
                                ChartWidget::new(up)
                                    .apply(container)
                                    .padding(padding::top(height / 5.0))
                                    .style(base_background)
                                    .width(width)
                                    .height(height)
                                    .apply(Element::new)
                            }
                        }
                    })
                    .collect(),
                ChartConfig::Disk(c) => c
                    .vis
                    .iter()
                    .map(|v| match v {
                        DoubleView::SuperimposedRunChart {
                            color_send,
                            color_receive,
                            aspect_ratio,
                        } => {
                            let Size { width, height } =
                                get_sized_aspect_ratio(&self.core.applet, *aspect_ratio);

                            let read = HistoryChart::auto_max(
                                &self.disk_read,
                                color_receive.as_rgba_color(self.get_theme()),
                            );
                            let write = HistoryChart::auto_max(
                                &self.disk_write,
                                color_send.as_rgba_color(self.get_theme()),
                            );

                            ChartWidget::new(SuperimposedHistoryChart {
                                h1: read,
                                h2: write,
                            })
                            // .apply(|bar| aspect_ratio_container(bar, *aspect_ratio))
                            .apply(container)
                            .padding(padding::top(height / 5.0))
                            .style(base_background)
                            .width(width)
                            .height(height)
                            .apply(Element::new)
                        }
                        DoubleView::SingleRunA {
                            color,
                            aspect_ratio,
                        } => {
                            let Size { width, height } =
                                get_sized_aspect_ratio(&self.core.applet, *aspect_ratio);

                            let read = HistoryChart::auto_max(
                                &self.disk_read,
                                color.as_rgba_color(self.get_theme()),
                            );
                            ChartWidget::new(read)
                                .apply(container)
                                .padding(padding::top(height / 5.0))
                                .style(base_background)
                                .width(width)
                                .height(height)
                                .apply(Element::new)
                        }
                        DoubleView::SingleRunB {
                            color,
                            aspect_ratio,
                        } => {
                            let Size { width, height } =
                                get_sized_aspect_ratio(&self.core.applet, *aspect_ratio);

                            let write = HistoryChart::auto_max(
                                &self.disk_write,
                                color.as_rgba_color(self.get_theme()),
                            );
                            ChartWidget::new(write)
                                .apply(container)
                                .padding(padding::top(height / 5.0))
                                .style(base_background)
                                .width(width)
                                .height(height)
                                .apply(Element::new)
                        }
                    })
                    .collect(),
            }
            .apply(|elements| collection(&self.core.applet, elements, INTRA_ITEM_SPACING, 0.0))
        });

        let items = collection(&self.core.applet, item_iter, 30, 0.0);

        self.core.applet.autosize_window(items).into()
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
            Message::Config(config) => self.config = config,
            Message::TickCpu => {
                todo!()
                // self.sys.refresh_cpu_all();
                // self.global_cpu.push(self.sys.global_cpu_usage());
            }
            Message::TickRam => {
                self.sys
                    .refresh_memory_specifics(MemoryRefreshKind::nothing().with_ram());
                self.ram.push(self.sys.used_memory());
            }
            Message::TickSwap => {
                self.sys
                    .refresh_memory_specifics(MemoryRefreshKind::nothing().with_swap());
                self.swap.push(self.sys.used_swap());
            }
            Message::TickNet => {
                self.nets.refresh(true);
                let (received, transmitted) =
                    self.nets.iter().fold((0, 0), |(acc_r, acc_t), (_, data)| {
                        (acc_r + data.received(), acc_t + data.transmitted())
                    });
                self.upload.push(transmitted);
                self.download.push(received);
            }
            Message::TickDisk => {
                self.disks.refresh(true);
                let (read, written) = self
                    .disks
                    .iter()
                    .map(Disk::usage)
                    .fold((0, 0), |(acc_r, acc_w), usage| {
                        (acc_r + usage.read_bytes, acc_w + usage.written_bytes)
                    });
                self.disk_read.push(read);
                self.disk_write.push(written);
            } //
              // Message::TickVRAM => self.chart.update_vram(),
        }
        Task::none()
    }

    fn subscription(&self) -> Subscription<Self::Message> {
        let mut subs = Vec::new();
        for chart in &self.config.charts {
            let tick = {
                match chart {
                    /*
                    ChartConfig::CPU(c) => {
                        cosmic::iced::time::every(Duration::from_millis(c.update_interval))
                            .map(|_| Message::TickCpu)
                    }
                    */
                    ChartConfig::Ram(c) => {
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
                    } /*
                      ChartConfig::VRAM(_c) => {
                          // uninplemented
                          continue;
                          // cosmic::iced::time::every(Duration::from_millis(c.update_interval))
                          // .map(|_| Message::TickVRAM)
                      }
                      */
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
