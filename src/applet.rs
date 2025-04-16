// SPDX-License-Identifier: GPL-3.0-only

use circular_queue::CircularQueue;
use cosmic::{
    app::{Core, Task},
    cosmic_config,
    iced::{Padding, Size, Subscription},
    iced_core::padding,
    widget::{container, Container},
    Application, Apply as _, Element, Renderer, Theme,
};
use plotters_iced::ChartWidget;
use std::time::Duration;
use sysinfo::{Cpu, Disk, Disks, MemoryRefreshKind, Networks, System};

use crate::{
    bar_chart::{SortMethod, VerticalPercentageBar},
    config::{config_subscription, ComponentConfig, Config, CpuView, DoubleView, SingleView},
    helpers::{base_background, init_history_with_default, panel_collection},
    run_chart::{HistoryChart, SuperimposedHistoryChart},
};

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
    /// percentage global cpu used between refreshes
    global_cpu: History<f32>,
    ram: History,
    swap: History,
    /// amount uploaded between refresh of sysinfo::Nets. (DEOS NOT STORE RATE)
    upload: History,
    /// amount downloaded between refresh of sysinfo::Nets. (DEOS NOT STORE RATE)
    download: History,
    /// amount read between refresh of sysinfo::Disks. (DEOS NOT STORE RATE)
    disk_read: History,
    /// amount written between refresh of sysinfo::Disks. (DEOS NOT STORE RATE)
    disk_write: History,
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

    fn size_aspect_ratio(&self, aspect_ratio: f32) -> Size {
        let (bounds_width, bounds_height) = self.core.applet.suggested_window_size();
        let padding = self.padding();

        if self.core.applet.is_horizontal() {
            let height = bounds_height.get() as f32 - padding.vertical();
            Size {
                width: height * aspect_ratio,
                height,
            }
        } else {
            let width = bounds_width.get() as f32 - padding.horizontal();
            Size {
                width,
                height: width * aspect_ratio,
            }
        }
    }

    fn padding(&self) -> Padding {
        match self.config.padding {
            crate::config::PaddingOption::Suggested => {
                self.core.applet.suggested_padding(false).into()
            }
            crate::config::PaddingOption::Custom(p) => p,
        }
        .into()
    }

    fn aspect_ratio_container<'a>(
        &self,
        content: impl Into<Element<'a, Message>>,
        aspect_ratio: f32,
    ) -> Container<'a, Message, Theme, Renderer> {
        let size = self.size_aspect_ratio(aspect_ratio);

        sized_container(content, size)
    }

    fn aspect_ratio_container_with_padding<'a>(
        &self,
        content: impl Into<Element<'a, Message>>,
        aspect_ratio: f32,
    ) -> Container<'a, Message, Theme, Renderer> {
        let size = self.size_aspect_ratio(aspect_ratio);

        sized_container(content, size).padding(padding::top(size.height / 5.0))
    }
}

fn sized_container<'a>(
    content: impl Into<Element<'a, Message>>,
    size: Size,
) -> Container<'a, Message, Theme> {
    container(content.into())
        .width(size.width)
        .height(size.height)
        .style(base_background)
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
        let (mut cpu, mut ram, mut swap, mut net, mut disk) = Default::default();
        for chart_config in &flags.config.components {
            match chart_config {
                ComponentConfig::Cpu(c) => cpu = Some(c.sampling_window),
                ComponentConfig::Ram(c) => ram = Some(c.sampling_window),
                ComponentConfig::Swap(c) => swap = Some(c.sampling_window),
                ComponentConfig::Net(c) => net = Some(c.sampling_window),
                ComponentConfig::Disk(c) => disk = Some(c.sampling_window),
            }
        }

        let app = Self {
            core,
            config: flags.config,
            config_handler: flags.config_handler,

            sys: System::new_all(),
            nets: Networks::new_with_refreshed_list(),
            disks: Disks::new_with_refreshed_list(),

            global_cpu: init_history_with_default(cpu.unwrap_or(0)),
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
        let item_iter = self.config.components.iter().map(|module| {
            match module {
                ComponentConfig::Cpu(c) => c
                    .vis
                    .iter()
                    .map(|v| match v {
                        CpuView::GlobalUsageBarChart {
                            aspect_ratio,
                            color,
                        } => {
                            let Size { width, height } = self.size_aspect_ratio(*aspect_ratio);

                            VerticalPercentageBar::new(self.sys.global_cpu_usage(), *color)
                                .apply(container)
                                .style(base_background)
                                .width(width)
                                .height(height)
                                .apply(Element::new)
                        }
                        CpuView::PerCoreUsageHistogram {
                            per_core_aspect_ratio: aspect_ratio,
                            color,
                            spacing,
                            sorting,
                        } => {
                            let Size { width, height } = self.size_aspect_ratio(*aspect_ratio);

                            let mut cpus: Vec<_> =
                                self.sys.cpus().iter().map(Cpu::cpu_usage).collect();

                            if let Some(method) = sorting {
                                match method {
                                    SortMethod::Descending => {
                                        cpus.sort_by(|a, b| b.partial_cmp(&a).unwrap());
                                    }
                                }
                                cpus.sort_by(method.method())
                            }

                            let bars: Vec<_> = cpus
                                .into_iter()
                                .map(|usage| {
                                    VerticalPercentageBar::new(usage, *color)
                                        .apply(container)
                                        .width(width)
                                        .height(height)
                                        .apply(Element::new)
                                })
                                .collect();

                            panel_collection(&self.core.applet, bars, *spacing, 0.0)
                                .apply(container)
                                .style(base_background)
                                .into()
                        }
                        CpuView::GlobalUsageRunChart {
                            aspect_ratio,
                            color,
                        } => {
                            let Size { width, height } = self.size_aspect_ratio(*aspect_ratio);
                            let chart = HistoryChart {
                                history: &self.global_cpu,
                                max: 100.0,
                                color: color.as_rgba_color(self.get_theme()),
                            };

                            ChartWidget::new(chart)
                                .apply(container)
                                .style(base_background)
                                .width(width)
                                .height(height)
                                .apply(Element::new)
                        }
                    })
                    .collect::<Vec<_>>(),
                ComponentConfig::Ram(c) => c
                    .vis
                    .iter()
                    .map(|v| {
                        match v {
                            SingleView::Bar {
                                aspect_ratio,
                                color,
                            } => {
                                let content = VerticalPercentageBar::from_pair(
                                    self.sys.used_memory(),
                                    self.sys.total_memory(),
                                    *color,
                                );
                                self.aspect_ratio_container(content, *aspect_ratio)
                            }
                            SingleView::Run {
                                aspect_ratio,
                                color,
                            } => {
                                let chart = HistoryChart {
                                    history: &self.ram,
                                    max: self.sys.total_memory(),
                                    color: color.as_rgba_color(self.get_theme()),
                                };

                                let content = ChartWidget::new(chart);
                                self.aspect_ratio_container(content, *aspect_ratio)
                            }
                        }
                        .into()
                    })
                    .collect(),
                ComponentConfig::Swap(c) => c
                    .vis
                    .iter()
                    .map(|v| {
                        match v {
                            SingleView::Bar {
                                aspect_ratio,
                                color,
                            } => {
                                let content =
                                    VerticalPercentageBar::new(self.swap_percentage(), *color);
                                self.aspect_ratio_container(content, *aspect_ratio)
                            }
                            SingleView::Run {
                                aspect_ratio,
                                color,
                            } => {
                                let chart = HistoryChart {
                                    history: &self.swap,
                                    max: self.sys.total_swap(),
                                    color: color.as_rgba_color(self.get_theme()),
                                };
                                let content = ChartWidget::new(chart);
                                self.aspect_ratio_container(content, *aspect_ratio)
                            }
                        }
                        .into()
                    })
                    .collect(),
                ComponentConfig::Net(c) => c
                    .vis
                    .iter()
                    .map(|v| {
                        match v {
                            DoubleView::SuperimposedRunChart {
                                aspect_ratio,
                                color_out: color_up,
                                color_in: color_down,
                            } => {
                                let upload = HistoryChart::auto_max(
                                    &self.upload,
                                    color_up.as_rgba_color(self.get_theme()),
                                );
                                let download = HistoryChart::auto_max(
                                    &self.download,
                                    color_down.as_rgba_color(self.get_theme()),
                                );

                                let content = ChartWidget::new(SuperimposedHistoryChart {
                                    h1: upload,
                                    h2: download,
                                });

                                self.aspect_ratio_container_with_padding(content, *aspect_ratio)
                            }
                            DoubleView::SingleRunA {
                                color,
                                aspect_ratio,
                            } => {
                                let down = HistoryChart::auto_max(
                                    &self.download,
                                    color.as_rgba_color(self.get_theme()),
                                );

                                let content = ChartWidget::new(down);
                                self.aspect_ratio_container_with_padding(content, *aspect_ratio)
                            }
                            DoubleView::SingleRunB {
                                color,
                                aspect_ratio,
                            } => {
                                let up = HistoryChart::auto_max(
                                    &self.upload,
                                    color.as_rgba_color(self.get_theme()),
                                );
                                let content = ChartWidget::new(up);
                                self.aspect_ratio_container_with_padding(content, *aspect_ratio)
                            }
                        }
                        .into()
                    })
                    .collect(),
                ComponentConfig::Disk(c) => c
                    .vis
                    .iter()
                    .map(|v| {
                        match v {
                            DoubleView::SuperimposedRunChart {
                                color_out: color_send,
                                color_in: color_receive,
                                aspect_ratio,
                            } => {
                                let read = HistoryChart::auto_max(
                                    &self.disk_read,
                                    color_receive.as_rgba_color(self.get_theme()),
                                );
                                let write = HistoryChart::auto_max(
                                    &self.disk_write,
                                    color_send.as_rgba_color(self.get_theme()),
                                );

                                let content = ChartWidget::new(SuperimposedHistoryChart {
                                    h1: read,
                                    h2: write,
                                });
                                self.aspect_ratio_container_with_padding(content, *aspect_ratio)
                            }
                            DoubleView::SingleRunA {
                                color,
                                aspect_ratio,
                            } => {
                                let read = HistoryChart::auto_max(
                                    &self.disk_read,
                                    color.as_rgba_color(self.get_theme()),
                                );
                                let content = ChartWidget::new(read);
                                self.aspect_ratio_container_with_padding(content, *aspect_ratio)
                            }
                            DoubleView::SingleRunB {
                                color,
                                aspect_ratio,
                            } => {
                                let write = HistoryChart::auto_max(
                                    &self.disk_write,
                                    color.as_rgba_color(self.get_theme()),
                                );
                                let content = ChartWidget::new(write);
                                self.aspect_ratio_container_with_padding(content, *aspect_ratio)
                            }
                        }
                        .into()
                    })
                    .collect(),
            }
            .apply(|elements| {
                panel_collection(
                    &self.core.applet,
                    elements,
                    self.config.spacing_within_component,
                    0.0,
                )
            })
        });

        let items = panel_collection(
            &self.core.applet,
            item_iter,
            self.config.spacing_between_components,
            self.padding(),
        );

        self.core.applet.autosize_window(items).into()
    }

    fn update(&mut self, message: Message) -> Task<Message> {
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
                self.sys.refresh_cpu_all();
                self.global_cpu.push(self.sys.global_cpu_usage());
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
        for chart in &self.config.components {
            let tick = {
                match chart {
                    ComponentConfig::Cpu(c) => {
                        cosmic::iced::time::every(Duration::from_millis(c.update_interval))
                            .map(|_| Message::TickCpu)
                    }
                    ComponentConfig::Ram(c) => {
                        cosmic::iced::time::every(Duration::from_millis(c.update_interval))
                            .map(|_| Message::TickRam)
                    }
                    ComponentConfig::Swap(c) => {
                        cosmic::iced::time::every(Duration::from_millis(c.update_interval))
                            .map(|_| Message::TickSwap)
                    }
                    ComponentConfig::Net(c) => {
                        cosmic::iced::time::every(Duration::from_millis(c.update_interval))
                            .map(|_| Message::TickNet)
                    }
                    ComponentConfig::Disk(c) => {
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
