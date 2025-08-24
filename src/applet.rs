// SPDX-License-Identifier: GPL-3.0-only

use cosmic::{
    app::{Core, Task},
    cosmic_config,
    iced::{Alignment, Padding, Pixels, Size, Subscription},
    iced_core::padding,
    surface,
    widget::{container, Column, Container, Row},
    Application, Apply as _, Element, Renderer, Theme,
};
use std::time::Duration;
use sysinfo::{
    Cpu, CpuRefreshKind, Disk, DiskRefreshKind, Disks, MemoryRefreshKind, Networks, RefreshKind,
    System,
};

use crate::{
    components::{
        bar::PercentageBar,
        gpu::{GpuData, Gpus},
        run::{HistoryChart, SimpleHistoryChart, SuperimposedHistoryChart},
    },
    config::{
        config_subscription, ComponentConfig, Config, CpuView, IoView, PaddingOption, PercentView,
    },
    history::History,
};

pub const ID: &str = "dev.DBrox.CosmicSystemMonitor";

pub struct SystemMonitorApplet {
    core: Core,
    config: Config,
    #[allow(dead_code)]
    config_handler: Option<cosmic_config::Config>,

    sys: System,
    nets: Networks,
    disks: Disks,
    gpus: Gpus,
    /// percentage global cpu used between refreshes
    global_cpu: History<f32>,
    ram: History,
    swap: History,
    /// amount uploaded between refresh of `sysinfo::Nets`. (DOES NOT STORE RATE)
    upload: History,
    /// amount downloaded between refresh of `sysinfo::Nets`. (DOES NOT STORE RATE)
    download: History,
    /// amount read between refresh of `sysinfo::Disks`. (DOES NOT STORE RATE)
    disk_read: History,
    /// amount written between refresh of `sysinfo::Disks`. (DOES NOT STORE RATE)
    disk_write: History,
    /// amount read between refresh of `sysinfo::Disks`. (DOES NOT STORE RATE)
    gpu_usage: Vec<History>,
    /// amount written between refresh of `sysinfo::Disks`. (DOES NOT STORE RATE)
    vram: Vec<History>,
}

#[derive(Debug, Clone)]
pub enum Message {
    Config(Config),
    TickCpu,
    TickMem,
    TickNet,
    TickDisk,
    TickGpu,
    Surface(surface::Action),
}

#[derive(Clone, Debug)]
pub struct Flags {
    pub config_handler: Option<cosmic_config::Config>,
    pub config: Config,
}

impl SystemMonitorApplet {
    fn size_aspect_ratio(&self, aspect_ratio: f32) -> Size {
        let (bounds_width, bounds_height) = self.core.applet.suggested_window_size();
        let padding = self.padding();

        #[allow(clippy::cast_precision_loss)]
        if self.is_horizontal() {
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
        match self.config.layout.padding {
            PaddingOption::Suggested => self.core.applet.suggested_padding(false).into(),
            PaddingOption::Custom(p) => p,
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

        sized_container(content, size).padding(padding::top(size.height / 5.0).bottom(0.0))
    }

    fn is_horizontal(&self) -> bool {
        self.core.applet.is_horizontal()
    }

    pub fn panel_collection<'a>(
        &self,
        elements: impl IntoIterator<Item = impl Into<Element<'a, Message>>>,
        spacing: impl Into<Pixels>,
        padding: impl Into<Padding>,
    ) -> Element<'a, Message> {
        if self.is_horizontal() {
            Row::with_children(elements.into_iter().map(Into::into))
                .spacing(spacing)
                .align_y(Alignment::Center)
                .padding(padding)
                .into()
        } else {
            Column::with_children(elements.into_iter().map(Into::into))
                .spacing(spacing)
                .align_x(Alignment::Center)
                .padding(padding)
                .into()
        }
    }

    // Helper functions for formatting data for tooltips
    fn format_bytes(&self, bytes: u64) -> String {
        const UNITS: &[&str] = &["B", "KB", "MB", "GB", "TB"];
        let mut size = bytes as f64;
        let mut unit_index = 0;

        while size >= 1024.0 && unit_index < UNITS.len() - 1 {
            size /= 1024.0;
            unit_index += 1;
        }

        if unit_index == 0 {
            format!("{}{}", bytes, UNITS[unit_index])
        } else {
            format!("{:.1}{}", size, UNITS[unit_index])
        }
    }

    fn format_percentage(&self, current: u64, total: u64) -> String {
        if total == 0 {
            "0.0%".to_string()
        } else {
            let percentage = (current as f64 / total as f64) * 100.0;
            format!("{:.1}%", percentage)
        }
    }

    fn format_cpu_tooltip(&self, usage: f32) -> String {
        format!("CPU: {:.1}%", usage)
    }

    fn format_mem_tooltip(&self) -> String {
        format!(
            "{}\n{}",
            self.format_ram_tooltip(),
            self.format_swap_tooltip()
        )
    }

    fn format_ram_tooltip(&self) -> String {
        let used = self.sys.used_memory();
        let total = self.sys.total_memory();
        let percentage = self.format_percentage(used, total);
        format!(
            "RAM: {} / {} ({})",
            self.format_bytes(used),
            self.format_bytes(total),
            percentage
        )
    }

    fn format_swap_tooltip(&self) -> String {
        let used = self.sys.used_swap();
        let total = self.sys.total_swap();
        if total == 0 {
            "Swap: Not available".to_string()
        } else {
            let percentage = self.format_percentage(used, total);
            format!(
                "Swap: {} / {} ({})",
                self.format_bytes(used),
                self.format_bytes(total),
                percentage
            )
        }
    }

    fn format_network_tooltip(&self) -> String {
        format!(
            "{}\n{}",
            self.format_network_tooltip_inner(false),
            self.format_network_tooltip_inner(true)
        )
    }

    fn format_network_tooltip_inner(&self, is_upload: bool) -> String {
        let history = if is_upload {
            &self.upload
        } else {
            &self.download
        };
        let current_rate = history.iter().last().copied().unwrap_or(0);
        let direction = if is_upload { "Upload" } else { "Download" };
        format!("{}: {}/s", direction, self.format_bytes(current_rate))
    }

    fn format_disk_tooltip(&self, is_write: bool) -> String {
        let history = if is_write {
            &self.disk_write
        } else {
            &self.disk_read
        };
        let current_rate = history.iter().last().copied().unwrap_or(0);
        let operation = if is_write { "Write" } else { "Read" };
        format!("Disk {}: {}/s", operation, self.format_bytes(current_rate))
    }

    fn format_gpu_tooltip(&self, gpu_index: usize, gpu_data: &GpuData) -> String {
        format!(
            "{}\n{}",
            self.format_gpu_usage_tooltip(gpu_index, gpu_data),
            self.format_gpu_vram_tooltip(gpu_index, gpu_data)
        )
    }

    fn format_gpu_usage_tooltip(&self, gpu_index: usize, gpu_data: &GpuData) -> String {
        format!("GPU{} Usage: {}%", gpu_index, gpu_data.usage)
    }

    fn format_gpu_vram_tooltip(&self, gpu_index: usize, gpu_data: &GpuData) -> String {
        let vram_percentage = self.format_percentage(gpu_data.used_vram, gpu_data.total_vram);
        format!(
            "GPU{} VRAM: {}/{} ({})",
            gpu_index,
            self.format_bytes(gpu_data.used_vram),
            self.format_bytes(gpu_data.total_vram),
            vram_percentage
        )
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
        let (mut cpu, mut mem, mut net, mut disk, mut gpu) = Default::default();
        let sampling = &flags.config.sampling;
        for chart_config in &flags.config.components {
            match chart_config {
                ComponentConfig::Cpu(_) => cpu = sampling.cpu.sampling_window,
                ComponentConfig::Mem(_) => mem = sampling.mem.sampling_window,
                ComponentConfig::Net(_) => net = sampling.net.sampling_window,
                ComponentConfig::Disk(_) => disk = sampling.disk.sampling_window,
                ComponentConfig::Gpu(_) => gpu = sampling.gpu.sampling_window,
            }
        }
        let gpus = Gpus::new();
        let app = Self {
            core,
            config: flags.config,
            config_handler: flags.config_handler,

            global_cpu: History::with_capacity(cpu),
            ram: History::with_capacity(mem),
            swap: History::with_capacity(mem),
            upload: History::with_capacity(net),
            download: History::with_capacity(net),
            disk_read: History::with_capacity(disk),
            disk_write: History::with_capacity(disk),
            gpu_usage: vec![History::with_capacity(gpu); gpus.num_gpus()],
            vram: vec![History::with_capacity(gpu); gpus.num_gpus()],

            sys: System::new_with_specifics(
                RefreshKind::nothing()
                    .with_cpu(CpuRefreshKind::nothing().with_cpu_usage())
                    .with_memory(MemoryRefreshKind::everything()),
            ),
            nets: Networks::new_with_refreshed_list(),
            disks: Disks::new_with_refreshed_list_specifics(
                DiskRefreshKind::nothing().with_io_usage(),
            ),
            gpus,
        };

        (app, Task::none())
    }

    #[allow(clippy::too_many_lines)]
    fn view(&'_ self) -> Element<'_, Message> {
        let item_iter = self.config.components.iter().map(|module| {
            match module {
                ComponentConfig::Cpu(vis) => vis
                    .iter()
                    .map(|v| match v {
                        CpuView::BarGlobal {
                            aspect_ratio,
                            color,
                        } => {
                            let content = PercentageBar::new(
                                self.is_horizontal(),
                                self.sys.global_cpu_usage(),
                                *color,
                            );
                            let container = self.aspect_ratio_container(content, *aspect_ratio);
                            let tooltip_text = self.format_cpu_tooltip(self.sys.global_cpu_usage());
                            if self.config.tooltip_enabled {
                                self.core
                                    .applet
                                    .applet_tooltip(
                                        container,
                                        tooltip_text,
                                        false,
                                        Message::Surface,
                                        None,
                                    )
                                    .into()
                            } else {
                                container.into()
                            }
                        }
                        CpuView::BarCores {
                            aspect_ratio,
                            color,
                            spacing,
                            sorting,
                        } => {
                            let mut cpus: Vec<_> =
                                self.sys.cpus().iter().map(Cpu::cpu_usage).collect();

                            cpus.sort_by(sorting.method());

                            let bars: Vec<Element<_>> = cpus
                                .into_iter()
                                .enumerate()
                                .map(|(core_idx, usage)| {
                                    let container = self.aspect_ratio_container(
                                        PercentageBar::new(self.is_horizontal(), usage, *color),
                                        *aspect_ratio,
                                    );
                                    let tooltip_text = format!("CPU{}: {:.1}%", core_idx, usage);
                                    if self.config.tooltip_enabled {
                                        self.core
                                            .applet
                                            .applet_tooltip(
                                                container,
                                                tooltip_text,
                                                false,
                                                Message::Surface,
                                                None,
                                            )
                                            .into()
                                    } else {
                                        container.into()
                                    }
                                })
                                .collect();

                            let container = self
                                .panel_collection(bars, *spacing, 0.0)
                                .apply(container)
                                .style(base_background);
                            let tooltip_text =
                                format!("CPU: {} cores total", self.sys.cpus().len());
                            if self.config.tooltip_enabled {
                                self.core
                                    .applet
                                    .applet_tooltip(
                                        container,
                                        tooltip_text,
                                        false,
                                        Message::Surface,
                                        None,
                                    )
                                    .into()
                            } else {
                                container.into()
                            }
                        }
                        CpuView::Run {
                            aspect_ratio,
                            color,
                        } => {
                            let chart = SimpleHistoryChart::new(&self.global_cpu, 100.0, *color);
                            let container = self.aspect_ratio_container(chart, *aspect_ratio);
                            let tooltip_text = self.format_cpu_tooltip(self.sys.global_cpu_usage());
                            if self.config.tooltip_enabled {
                                self.core
                                    .applet
                                    .applet_tooltip(
                                        container,
                                        tooltip_text,
                                        false,
                                        Message::Surface,
                                        None,
                                    )
                                    .into()
                            } else {
                                container.into()
                            }
                        }
                    })
                    .collect::<Vec<Element<_>>>(),
                ComponentConfig::Mem(vis) => vis
                    .iter()
                    .map(|v| match v {
                        PercentView::Bar {
                            color_left,
                            color_right,
                            spacing,
                            aspect_ratio,
                        } => {
                            let bars: Vec<Element<_>> = vec![
                                {
                                    let container = self.aspect_ratio_container(
                                        PercentageBar::from_pair(
                                            self.is_horizontal(),
                                            self.sys.used_memory(),
                                            self.sys.total_memory(),
                                            *color_left,
                                        ),
                                        *aspect_ratio,
                                    );
                                    let tooltip_text = self.format_ram_tooltip();
                                    if self.config.tooltip_enabled {
                                        self.core
                                            .applet
                                            .applet_tooltip(
                                                container,
                                                tooltip_text,
                                                false,
                                                Message::Surface,
                                                None,
                                            )
                                            .into()
                                    } else {
                                        container.into()
                                    }
                                },
                                {
                                    let container = self.aspect_ratio_container(
                                        PercentageBar::from_pair(
                                            self.is_horizontal(),
                                            self.sys.used_swap(),
                                            self.sys.total_swap(),
                                            *color_right,
                                        ),
                                        *aspect_ratio,
                                    );
                                    let tooltip_text = self.format_swap_tooltip();
                                    if self.config.tooltip_enabled {
                                        self.core
                                            .applet
                                            .applet_tooltip(
                                                container,
                                                tooltip_text,
                                                false,
                                                Message::Surface,
                                                None,
                                            )
                                            .into()
                                    } else {
                                        container.into()
                                    }
                                },
                            ];
                            let container = self
                                .panel_collection(bars, *spacing, 0.0)
                                .apply(container)
                                .style(base_background);
                            let tooltip_text = self.format_mem_tooltip();
                            if self.config.tooltip_enabled {
                                self.core
                                    .applet
                                    .applet_tooltip(
                                        container,
                                        tooltip_text,
                                        false,
                                        Message::Surface,
                                        None,
                                    )
                                    .into()
                            } else {
                                container.into()
                            }
                        }
                        PercentView::BarLeft {
                            color,
                            aspect_ratio,
                        } => {
                            let content = PercentageBar::from_pair(
                                self.is_horizontal(),
                                self.sys.used_memory(),
                                self.sys.total_memory(),
                                *color,
                            );
                            let container = self.aspect_ratio_container(content, *aspect_ratio);
                            let tooltip_text = self.format_ram_tooltip();
                            if self.config.tooltip_enabled {
                                self.core
                                    .applet
                                    .applet_tooltip(
                                        container,
                                        tooltip_text,
                                        false,
                                        Message::Surface,
                                        None,
                                    )
                                    .into()
                            } else {
                                container.into()
                            }
                        }
                        PercentView::BarRight {
                            color,
                            aspect_ratio,
                        } => {
                            let content = PercentageBar::from_pair(
                                self.is_horizontal(),
                                self.sys.used_swap(),
                                self.sys.total_swap(),
                                *color,
                            );
                            let container = self.aspect_ratio_container(content, *aspect_ratio);
                            let tooltip_text = self.format_swap_tooltip();
                            if self.config.tooltip_enabled {
                                self.core
                                    .applet
                                    .applet_tooltip(
                                        container,
                                        tooltip_text,
                                        false,
                                        Message::Surface,
                                        None,
                                    )
                                    .into()
                            } else {
                                container.into()
                            }
                        }
                        PercentView::Run {
                            aspect_ratio,
                            color_back,
                            color_front,
                        } => {
                            let ram =
                                HistoryChart::new(&self.ram, self.sys.total_memory(), *color_back);
                            let swap =
                                HistoryChart::new(&self.swap, self.sys.total_swap(), *color_front);

                            let content = SuperimposedHistoryChart {
                                back: ram,
                                front: swap,
                            };

                            let container = self.aspect_ratio_container(content, *aspect_ratio);
                            let tooltip_text = self.format_mem_tooltip();
                            if self.config.tooltip_enabled {
                                self.core
                                    .applet
                                    .applet_tooltip(
                                        container,
                                        tooltip_text,
                                        false,
                                        Message::Surface,
                                        None,
                                    )
                                    .into()
                            } else {
                                container.into()
                            }
                        }
                        PercentView::RunBack {
                            color,
                            aspect_ratio,
                        } => {
                            let ram =
                                SimpleHistoryChart::new(&self.ram, self.sys.total_memory(), *color);
                            let container = self.aspect_ratio_container(ram, *aspect_ratio);
                            let tooltip_text = self.format_ram_tooltip();
                            if self.config.tooltip_enabled {
                                self.core
                                    .applet
                                    .applet_tooltip(
                                        container,
                                        tooltip_text,
                                        false,
                                        Message::Surface,
                                        None,
                                    )
                                    .into()
                            } else {
                                container.into()
                            }
                        }
                        PercentView::RunFront {
                            color,
                            aspect_ratio,
                        } => {
                            let swap =
                                SimpleHistoryChart::new(&self.swap, self.sys.total_swap(), *color);
                            let container = self.aspect_ratio_container(swap, *aspect_ratio);
                            let tooltip_text = self.format_swap_tooltip();
                            if self.config.tooltip_enabled {
                                self.core
                                    .applet
                                    .applet_tooltip(
                                        container,
                                        tooltip_text,
                                        false,
                                        Message::Surface,
                                        None,
                                    )
                                    .into()
                            } else {
                                container.into()
                            }
                        }
                    })
                    .collect(),
                ComponentConfig::Net(vis) => vis
                    .iter()
                    .map(|v| match v {
                        IoView::Run {
                            aspect_ratio,
                            color_front,
                            color_back,
                        } => {
                            let mut download = HistoryChart::auto_max(&self.download, *color_back);
                            let mut upload = HistoryChart::auto_max(&self.upload, *color_front);
                            HistoryChart::link_max(&mut download, &mut upload);

                            let content = SuperimposedHistoryChart {
                                back: download,
                                front: upload,
                            };

                            let container =
                                self.aspect_ratio_container_with_padding(content, *aspect_ratio);
                            let tooltip_text = self.format_network_tooltip();
                            if self.config.tooltip_enabled {
                                self.core
                                    .applet
                                    .applet_tooltip(
                                        container,
                                        tooltip_text,
                                        false,
                                        Message::Surface,
                                        None,
                                    )
                                    .into()
                            } else {
                                container.into()
                            }
                        }
                        IoView::RunBack {
                            color,
                            aspect_ratio,
                        } => {
                            let down = SimpleHistoryChart::auto_max(&self.download, *color);

                            let container =
                                self.aspect_ratio_container_with_padding(down, *aspect_ratio);
                            let tooltip_text = self.format_network_tooltip_inner(false);
                            if self.config.tooltip_enabled {
                                self.core
                                    .applet
                                    .applet_tooltip(
                                        container,
                                        tooltip_text,
                                        false,
                                        Message::Surface,
                                        None,
                                    )
                                    .into()
                            } else {
                                container.into()
                            }
                        }
                        IoView::RunFront {
                            color,
                            aspect_ratio,
                        } => {
                            let up = SimpleHistoryChart::auto_max(&self.upload, *color);
                            let container =
                                self.aspect_ratio_container_with_padding(up, *aspect_ratio);
                            let tooltip_text = self.format_network_tooltip_inner(true);
                            if self.config.tooltip_enabled {
                                self.core
                                    .applet
                                    .applet_tooltip(
                                        container,
                                        tooltip_text,
                                        false,
                                        Message::Surface,
                                        None,
                                    )
                                    .into()
                            } else {
                                container.into()
                            }
                        }
                    })
                    .collect(),
                ComponentConfig::Disk(vis) => vis
                    .iter()
                    .map(|v| match v {
                        IoView::Run {
                            color_front,
                            color_back,
                            aspect_ratio,
                        } => {
                            let mut read = HistoryChart::auto_max(&self.disk_read, *color_back);
                            let mut write = HistoryChart::auto_max(&self.disk_write, *color_front);
                            HistoryChart::link_max(&mut read, &mut write);

                            let content = SuperimposedHistoryChart {
                                back: read,
                                front: write,
                            };
                            let container =
                                self.aspect_ratio_container_with_padding(content, *aspect_ratio);
                            let tooltip_text = format!(
                                "{}\n{}",
                                self.format_disk_tooltip(false),
                                self.format_disk_tooltip(true)
                            );
                            if self.config.tooltip_enabled {
                                self.core
                                    .applet
                                    .applet_tooltip(
                                        container,
                                        tooltip_text,
                                        false,
                                        Message::Surface,
                                        None,
                                    )
                                    .into()
                            } else {
                                container.into()
                            }
                        }
                        IoView::RunBack {
                            color,
                            aspect_ratio,
                        } => {
                            let read = SimpleHistoryChart::auto_max(&self.disk_read, *color);
                            let container =
                                self.aspect_ratio_container_with_padding(read, *aspect_ratio);
                            let tooltip_text = self.format_disk_tooltip(false);
                            if self.config.tooltip_enabled {
                                self.core
                                    .applet
                                    .applet_tooltip(
                                        container,
                                        tooltip_text,
                                        false,
                                        Message::Surface,
                                        None,
                                    )
                                    .into()
                            } else {
                                container.into()
                            }
                        }
                        IoView::RunFront {
                            color,
                            aspect_ratio,
                        } => {
                            let write = SimpleHistoryChart::auto_max(&self.disk_write, *color);
                            let container =
                                self.aspect_ratio_container_with_padding(write, *aspect_ratio);
                            let tooltip_text = self.format_disk_tooltip(true);
                            if self.config.tooltip_enabled {
                                self.core
                                    .applet
                                    .applet_tooltip(
                                        container,
                                        tooltip_text,
                                        false,
                                        Message::Surface,
                                        None,
                                    )
                                    .into()
                            } else {
                                container.into()
                            }
                        }
                    })
                    .collect(),
                ComponentConfig::Gpu(vis) => self
                    .gpus
                    .data()
                    .iter()
                    .enumerate()
                    .flat_map(|(idx, data)| {
                        let idx = &idx;
                        vis.iter()
                            .map(|v| match v {
                                PercentView::Bar {
                                    color_left,
                                    color_right,
                                    spacing,
                                    aspect_ratio,
                                } => {
                                    let bars: Vec<Element<_>> = vec![
                                        {
                                            let container = self.aspect_ratio_container(
                                                PercentageBar::from_pair(
                                                    self.is_horizontal(),
                                                    data.usage,
                                                    100,
                                                    *color_left,
                                                ),
                                                *aspect_ratio,
                                            );
                                            let tooltip_text =
                                                self.format_gpu_usage_tooltip(*idx, data);
                                            if self.config.tooltip_enabled {
                                                self.core
                                                    .applet
                                                    .applet_tooltip(
                                                        container,
                                                        tooltip_text,
                                                        false,
                                                        Message::Surface,
                                                        None,
                                                    )
                                                    .into()
                                            } else {
                                                container.into()
                                            }
                                        },
                                        {
                                            let container = self.aspect_ratio_container(
                                                PercentageBar::from_pair(
                                                    self.is_horizontal(),
                                                    data.used_vram,
                                                    data.total_vram,
                                                    *color_right,
                                                ),
                                                *aspect_ratio,
                                            );
                                            let vram_tooltip =
                                                self.format_gpu_vram_tooltip(*idx, data);
                                            if self.config.tooltip_enabled {
                                                self.core
                                                    .applet
                                                    .applet_tooltip(
                                                        container,
                                                        vram_tooltip,
                                                        false,
                                                        Message::Surface,
                                                        None,
                                                    )
                                                    .into()
                                            } else {
                                                container.into()
                                            }
                                        },
                                    ];
                                    let container = self
                                        .panel_collection(bars, *spacing, 0.0)
                                        .apply(container)
                                        .style(base_background);
                                    let tooltip_text = self.format_gpu_tooltip(*idx, data);
                                    if self.config.tooltip_enabled {
                                        self.core
                                            .applet
                                            .applet_tooltip(
                                                container,
                                                tooltip_text,
                                                false,
                                                Message::Surface,
                                                None,
                                            )
                                            .into()
                                    } else {
                                        container.into()
                                    }
                                }
                                PercentView::BarLeft {
                                    color,
                                    aspect_ratio,
                                } => {
                                    let content = PercentageBar::from_pair(
                                        self.is_horizontal(),
                                        data.usage,
                                        100,
                                        *color,
                                    );
                                    let container =
                                        self.aspect_ratio_container(content, *aspect_ratio);
                                    let tooltip_text = self.format_gpu_usage_tooltip(*idx, data);
                                    if self.config.tooltip_enabled {
                                        self.core
                                            .applet
                                            .applet_tooltip(
                                                container,
                                                tooltip_text,
                                                false,
                                                Message::Surface,
                                                None,
                                            )
                                            .into()
                                    } else {
                                        container.into()
                                    }
                                }
                                PercentView::BarRight {
                                    color,
                                    aspect_ratio,
                                } => {
                                    let content = PercentageBar::from_pair(
                                        self.is_horizontal(),
                                        data.used_vram,
                                        data.total_vram,
                                        *color,
                                    );
                                    let container =
                                        self.aspect_ratio_container(content, *aspect_ratio);
                                    let vram_tooltip = self.format_gpu_vram_tooltip(*idx, data);
                                    if self.config.tooltip_enabled {
                                        self.core
                                            .applet
                                            .applet_tooltip(
                                                container,
                                                vram_tooltip,
                                                false,
                                                Message::Surface,
                                                None,
                                            )
                                            .into()
                                    } else {
                                        container.into()
                                    }
                                }
                                PercentView::Run {
                                    aspect_ratio,
                                    color_back,
                                    color_front,
                                } => {
                                    let usage =
                                        HistoryChart::new(&self.gpu_usage[*idx], 100, *color_back);
                                    let vram = HistoryChart::new(
                                        &self.vram[*idx],
                                        data.total_vram,
                                        *color_front,
                                    );

                                    let content = SuperimposedHistoryChart {
                                        back: usage,
                                        front: vram,
                                    };

                                    let container =
                                        self.aspect_ratio_container(content, *aspect_ratio);
                                    let tooltip_text = self.format_gpu_tooltip(*idx, data);
                                    if self.config.tooltip_enabled {
                                        self.core
                                            .applet
                                            .applet_tooltip(
                                                container,
                                                tooltip_text,
                                                false,
                                                Message::Surface,
                                                None,
                                            )
                                            .into()
                                    } else {
                                        container.into()
                                    }
                                }
                                PercentView::RunBack {
                                    color,
                                    aspect_ratio,
                                } => {
                                    let usage =
                                        SimpleHistoryChart::new(&self.gpu_usage[*idx], 100, *color);
                                    let container =
                                        self.aspect_ratio_container(usage, *aspect_ratio);
                                    let tooltip_text = self.format_gpu_usage_tooltip(*idx, data);
                                    if self.config.tooltip_enabled {
                                        self.core
                                            .applet
                                            .applet_tooltip(
                                                container,
                                                tooltip_text,
                                                false,
                                                Message::Surface,
                                                None,
                                            )
                                            .into()
                                    } else {
                                        container.into()
                                    }
                                }
                                PercentView::RunFront {
                                    color,
                                    aspect_ratio,
                                } => {
                                    let vram = SimpleHistoryChart::new(
                                        &self.vram[*idx],
                                        data.total_vram,
                                        *color,
                                    );
                                    let container =
                                        self.aspect_ratio_container(vram, *aspect_ratio);
                                    let vram_tooltip = self.format_gpu_vram_tooltip(*idx, data);
                                    if self.config.tooltip_enabled {
                                        self.core
                                            .applet
                                            .applet_tooltip(
                                                container,
                                                vram_tooltip,
                                                false,
                                                Message::Surface,
                                                None,
                                            )
                                            .into()
                                    } else {
                                        container.into()
                                    }
                                }
                            })
                            .collect::<Vec<_>>()
                    })
                    .collect(),
            }
            .apply(|elements| {
                self.panel_collection(elements, self.config.layout.inner_spacing, 0.0)
            })
        });

        let items = self.panel_collection(item_iter, self.config.layout.spacing, self.padding());

        self.core.applet.autosize_window(items).into()
    }

    fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::Config(config) => {
                self.config = config;
                let sampĺing = &self.config.sampling;
                self.global_cpu.resize(sampĺing.cpu.sampling_window);
                self.ram.resize(sampĺing.mem.sampling_window);
                self.swap.resize(sampĺing.mem.sampling_window);
                self.upload.resize(sampĺing.net.sampling_window);
                self.download.resize(sampĺing.net.sampling_window);
                self.disk_read.resize(sampĺing.disk.sampling_window);
                self.disk_write.resize(sampĺing.disk.sampling_window);
                for i in 0..self.gpus.num_gpus() {
                    self.gpu_usage[i].resize(sampĺing.cpu.sampling_window);
                    self.vram[i].resize(sampĺing.cpu.sampling_window);
                }
            }
            Message::TickCpu => {
                self.sys.refresh_cpu_usage();
                self.global_cpu.push(self.sys.global_cpu_usage());
            }
            Message::Surface(a) => {
                return cosmic::task::message(cosmic::Action::Cosmic(
                    cosmic::app::Action::Surface(a),
                ));
            }
            Message::TickMem => {
                self.sys.refresh_memory();
                self.ram.push(self.sys.used_memory());
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
                self.disks
                    .refresh_specifics(true, DiskRefreshKind::nothing().with_io_usage());
                let (read, written) = self
                    .disks
                    .iter()
                    .map(Disk::usage)
                    .fold((0, 0), |(acc_r, acc_w), usage| {
                        (acc_r + usage.read_bytes, acc_w + usage.written_bytes)
                    });
                self.disk_read.push(read);
                self.disk_write.push(written);
            }
            Message::TickGpu => {
                self.gpus.refresh();
                for (idx, data) in self.gpus.data().iter().enumerate() {
                    self.gpu_usage[idx].push(data.usage);
                    self.vram[idx].push(data.used_vram);
                }
            }
        }
        Task::none()
    }

    fn subscription(&self) -> Subscription<Self::Message> {
        let mut subs = Vec::new();
        let sampling = &self.config.sampling;
        for chart in &self.config.components {
            let tick = {
                match chart {
                    ComponentConfig::Cpu(_) => cosmic::iced::time::every(Duration::from_millis(
                        sampling.cpu.update_interval,
                    ))
                    .map(|_| Message::TickCpu),
                    ComponentConfig::Mem(_) => cosmic::iced::time::every(Duration::from_millis(
                        sampling.mem.update_interval,
                    ))
                    .map(|_| Message::TickMem),
                    ComponentConfig::Net(_) => cosmic::iced::time::every(Duration::from_millis(
                        sampling.net.update_interval,
                    ))
                    .map(|_| Message::TickNet),
                    ComponentConfig::Disk(_) => cosmic::iced::time::every(Duration::from_millis(
                        sampling.disk.update_interval,
                    ))
                    .map(|_| Message::TickDisk),
                    ComponentConfig::Gpu(_) => cosmic::iced::time::every(Duration::from_millis(
                        sampling.gpu.update_interval,
                    ))
                    .map(|_| Message::TickGpu),
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

pub fn base_background(theme: &Theme) -> container::Style {
    container::Style {
        background: Some(cosmic::iced::Color::from(theme.cosmic().primary.base).into()),
        ..container::Style::default()
    }
}
