use crate::{
    applet::{Message, SystemMonitorApplet, base_background},
    color::Color,
    components::{
        bar::PercentageBar,
        gpu::GpuData,
        run::{SimpleHistoryChart, SuperimposedHistoryChart},
    },
    config::{CpuView, IoView, PaddingOption, PercentView},
};
use cosmic::{
    Apply, Element, Renderer, Theme,
    iced::{Alignment, Padding, Pixels, Size, padding},
    widget::{Column, Container, Row, container},
};
use sysinfo::Cpu;

fn sized_container<'a>(
    content: impl Into<Element<'a, Message>>,
    size: Size,
) -> Container<'a, Message, Theme> {
    container(content.into())
        .width(size.width)
        .height(size.height)
        .style(base_background)
}

fn format_bytes(bytes: u64) -> String {
    const UNITS: &[&str] = &["B", "KB", "MB", "GB", "TB"];
    #[allow(clippy::cast_precision_loss)]
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

fn format_percentage(current: u64, total: u64) -> String {
    if total == 0 {
        "0.0%".to_string()
    } else {
        #[allow(clippy::cast_precision_loss)]
        let percentage = (current as f64 / total as f64) * 100.0;
        format!("{percentage:.1}%")
    }
}

pub fn format_cpu_tooltip(usage: f32) -> String {
    format!("CPU: {usage:.1}%")
}

fn format_gpu_tooltip(gpu_index: usize, gpu_data: &GpuData) -> String {
    format!(
        "{}\n{}",
        format_gpu_usage_tooltip(gpu_index, gpu_data),
        format_gpu_vram_tooltip(gpu_index, gpu_data)
    )
}

fn format_gpu_usage_tooltip(gpu_index: usize, gpu_data: &GpuData) -> String {
    format!("GPU{} Usage: {}%", gpu_index, gpu_data.usage)
}

fn format_gpu_vram_tooltip(gpu_index: usize, gpu_data: &GpuData) -> String {
    let vram_percentage = format_percentage(gpu_data.used_vram, gpu_data.total_vram);
    format!(
        "GPU{} VRAM: {}/{} ({})",
        gpu_index,
        format_bytes(gpu_data.used_vram),
        format_bytes(gpu_data.total_vram),
        vram_percentage
    )
}

impl SystemMonitorApplet {
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
        let percentage = format_percentage(used, total);
        format!(
            "RAM: {} / {} ({})",
            format_bytes(used),
            format_bytes(total),
            percentage
        )
    }

    fn format_swap_tooltip(&self) -> String {
        let used = self.sys.used_swap();
        let total = self.sys.total_swap();
        if total == 0 {
            "Swap: Not available".to_string()
        } else {
            let percentage = format_percentage(used, total);
            format!(
                "Swap: {} / {} ({})",
                format_bytes(used),
                format_bytes(total),
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
        format!("{}: {}/s", direction, format_bytes(current_rate))
    }

    fn format_disk_tooltip(&self) -> String {
        format!(
            "{}\n{}",
            self.format_disk_tooltip_inner(false),
            self.format_disk_tooltip_inner(true)
        )
    }

    fn format_disk_tooltip_inner(&self, is_write: bool) -> String {
        let history = if is_write {
            &self.disk_write
        } else {
            &self.disk_read
        };
        let current_rate = history.iter().last().copied().unwrap_or(0);
        let operation = if is_write { "Write" } else { "Read" };
        format!("Disk {}: {}/s", operation, format_bytes(current_rate))
    }

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

    pub fn padding(&self) -> Padding {
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
        sized_container(content, self.size_aspect_ratio(aspect_ratio))
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

    fn maybe_tooltip<'a>(
        &self,
        container: Container<'a, Message, Theme>,
        tooltip_text: String,
    ) -> Element<'a, Message> {
        if self.config.tooltip_enabled {
            self.core
                .applet
                .applet_tooltip(container, tooltip_text, false, Message::Surface, None)
                .into()
        } else {
            container.into()
        }
    }

    fn single_run_view<'a, T>(
        &self,
        content: SimpleHistoryChart<'a, T>,
        tooltip_text: String,
        aspect_ratio: f32,
    ) -> Element<'a, Message>
    where
        SimpleHistoryChart<'a, T>: Into<Element<'a, Message>>,
    {
        self.aspect_ratio_container(content, aspect_ratio)
            .apply(|c| self.maybe_tooltip(c, tooltip_text))
    }

    fn double_run_view<'a>(
        &'a self,
        content: SuperimposedHistoryChart<'a>,
        tooltip_text: String,
        aspect_ratio: f32,
    ) -> Element<'a, Message> {
        self.aspect_ratio_container_with_padding(content, aspect_ratio)
            .apply(|c| self.maybe_tooltip(c, tooltip_text))
    }

    fn single_bar_view(
        &'_ self,
        data: u64,
        max: u64,
        color: &Color,
        tooltip_text: String,
        aspect_ratio: f32,
    ) -> Element<'_, Message> {
        self.aspect_ratio_container(
            PercentageBar::from_pair(self.is_horizontal(), data, max, *color),
            aspect_ratio,
        )
        .apply(|c| self.maybe_tooltip(c, tooltip_text))
    }

    fn double_bar_view<'a>(
        &'a self,
        content_left: Element<'a, Message>,
        content_right: Element<'a, Message>,
        tooltip_text: String,
        spacing: f32,
    ) -> Element<'a, Message> {
        self.panel_collection(vec![content_left, content_right], spacing, 0.0)
            .apply(container)
            .style(base_background)
            .apply(|c| self.maybe_tooltip(c, tooltip_text))
    }

    pub fn cpu_bar_view(
        &'_ self,
        data: f32,
        color: &Color,
        tooltip_text: String,
        aspect_ratio: f32,
    ) -> Element<'_, Message> {
        self.aspect_ratio_container(
            PercentageBar::new(self.is_horizontal(), data, *color),
            aspect_ratio,
        )
        .apply(|c| self.maybe_tooltip(c, tooltip_text))
    }

    pub fn cpu_view(&'_ self, vis: &[CpuView]) -> Vec<Element<'_, Message>> {
        vis.iter()
            .map(|v| match v {
                CpuView::BarGlobal {
                    aspect_ratio,
                    color,
                } => self.cpu_bar_view(
                    self.sys.global_cpu_usage(),
                    color,
                    format_cpu_tooltip(self.sys.global_cpu_usage()),
                    *aspect_ratio,
                ),
                CpuView::BarCores {
                    aspect_ratio,
                    color,
                    spacing,
                    sorting,
                } => {
                    let mut cpus: Vec<_> = self.sys.cpus().iter().map(Cpu::cpu_usage).collect();
                    cpus.sort_by(sorting.method());

                    let bars: Vec<Element<_>> = cpus
                        .into_iter()
                        .enumerate()
                        .map(|(core_idx, usage)| {
                            self.cpu_bar_view(
                                usage,
                                color,
                                format!("CPU{core_idx}: {usage:.1}%"),
                                *aspect_ratio,
                            )
                        })
                        .collect();

                    self.panel_collection(bars, *spacing, 0.0)
                        .apply(container)
                        .style(base_background)
                        .apply(|c| {
                            self.maybe_tooltip(
                                c,
                                format!("CPU: {} cores total", self.sys.cpus().len()),
                            )
                        })
                }
                CpuView::Run {
                    aspect_ratio,
                    color,
                } => self.single_run_view(
                    SimpleHistoryChart::new(&self.global_cpu, 100.0, *color),
                    format_cpu_tooltip(self.sys.global_cpu_usage()),
                    *aspect_ratio,
                ),
            })
            .collect::<Vec<Element<_>>>()
    }

    pub fn mem_view(&'_ self, vis: &[PercentView]) -> Vec<Element<'_, Message>> {
        vis.iter()
            .map(|v| match v {
                PercentView::Bar {
                    color_left,
                    color_right,
                    spacing,
                    aspect_ratio,
                } => self.double_bar_view(
                    self.single_bar_view(
                        self.sys.used_memory(),
                        self.sys.total_memory(),
                        color_left,
                        self.format_ram_tooltip(),
                        *aspect_ratio,
                    ),
                    self.single_bar_view(
                        self.sys.used_swap(),
                        self.sys.total_swap(),
                        color_right,
                        self.format_swap_tooltip(),
                        *aspect_ratio,
                    ),
                    self.format_mem_tooltip(),
                    *spacing,
                ),
                PercentView::BarLeft {
                    color,
                    aspect_ratio,
                } => self.single_bar_view(
                    self.sys.used_memory(),
                    self.sys.total_memory(),
                    color,
                    self.format_ram_tooltip(),
                    *aspect_ratio,
                ),

                PercentView::BarRight {
                    color,
                    aspect_ratio,
                } => self.single_bar_view(
                    self.sys.used_swap(),
                    self.sys.total_swap(),
                    color,
                    self.format_swap_tooltip(),
                    *aspect_ratio,
                ),
                PercentView::Run {
                    aspect_ratio,
                    color_back,
                    color_front,
                } => self.double_run_view(
                    SuperimposedHistoryChart::new(
                        &self.swap,
                        self.sys.total_swap(),
                        color_front,
                        &self.ram,
                        self.sys.total_memory(),
                        color_back,
                    ),
                    self.format_mem_tooltip(),
                    *aspect_ratio,
                ),
                PercentView::RunBack {
                    color,
                    aspect_ratio,
                } => self.single_run_view(
                    SimpleHistoryChart::new(&self.ram, self.sys.total_memory(), *color),
                    self.format_ram_tooltip(),
                    *aspect_ratio,
                ),
                PercentView::RunFront {
                    color,
                    aspect_ratio,
                } => self.single_run_view(
                    SimpleHistoryChart::new(&self.swap, self.sys.total_swap(), *color),
                    self.format_swap_tooltip(),
                    *aspect_ratio,
                ),
            })
            .collect()
    }

    pub fn net_view(&'_ self, vis: &[IoView]) -> Vec<Element<'_, Message>> {
        vis.iter()
            .map(|v| match v {
                IoView::Run {
                    aspect_ratio,
                    color_front,
                    color_back,
                } => self.double_run_view(
                    SuperimposedHistoryChart::new_linked(
                        &self.upload,
                        color_front,
                        &self.download,
                        color_back,
                    ),
                    self.format_network_tooltip(),
                    *aspect_ratio,
                ),
                IoView::RunBack {
                    color,
                    aspect_ratio,
                } => self.single_run_view(
                    SimpleHistoryChart::auto_max(&self.download, *color),
                    self.format_swap_tooltip(),
                    *aspect_ratio,
                ),
                IoView::RunFront {
                    color,
                    aspect_ratio,
                } => self.single_run_view(
                    SimpleHistoryChart::auto_max(&self.upload, *color),
                    self.format_swap_tooltip(),
                    *aspect_ratio,
                ),
            })
            .collect()
    }

    pub fn disk_view(&'_ self, vis: &[IoView]) -> Vec<Element<'_, Message>> {
        vis.iter()
            .map(|v| match v {
                IoView::Run {
                    color_front,
                    color_back,
                    aspect_ratio,
                } => self.double_run_view(
                    SuperimposedHistoryChart::new_linked(
                        &self.disk_write,
                        color_front,
                        &self.disk_read,
                        color_back,
                    ),
                    self.format_disk_tooltip(),
                    *aspect_ratio,
                ),
                IoView::RunBack {
                    color,
                    aspect_ratio,
                } => self.single_run_view(
                    SimpleHistoryChart::auto_max(&self.disk_read, *color),
                    self.format_swap_tooltip(),
                    *aspect_ratio,
                ),
                IoView::RunFront {
                    color,
                    aspect_ratio,
                } => self.single_run_view(
                    SimpleHistoryChart::auto_max(&self.disk_write, *color),
                    self.format_swap_tooltip(),
                    *aspect_ratio,
                ),
            })
            .collect()
    }

    pub fn gpu_view(&'_ self, vis: &[PercentView]) -> Vec<Element<'_, Message>> {
        self.gpus
            .data()
            .iter()
            .enumerate()
            .flat_map(|(idx, data)| {
                vis.iter()
                    .map(|v| match v {
                        PercentView::Bar {
                            color_left,
                            color_right,
                            spacing,
                            aspect_ratio,
                        } => self.double_bar_view(
                            self.single_bar_view(
                                data.usage,
                                100,
                                color_left,
                                format_gpu_usage_tooltip(idx, data),
                                *aspect_ratio,
                            ),
                            self.single_bar_view(
                                data.used_vram,
                                data.total_vram,
                                color_right,
                                format_gpu_vram_tooltip(idx, data),
                                *aspect_ratio,
                            ),
                            format_gpu_tooltip(idx, data),
                            *spacing,
                        ),
                        PercentView::BarLeft {
                            color,
                            aspect_ratio,
                        } => self.single_bar_view(
                            data.usage,
                            100,
                            color,
                            format_gpu_usage_tooltip(idx, data),
                            *aspect_ratio,
                        ),
                        PercentView::BarRight {
                            color,
                            aspect_ratio,
                        } => self.single_bar_view(
                            data.used_vram,
                            data.total_vram,
                            color,
                            format_gpu_vram_tooltip(idx, data),
                            *aspect_ratio,
                        ),

                        PercentView::Run {
                            aspect_ratio,
                            color_back,
                            color_front,
                        } => self.double_run_view(
                            SuperimposedHistoryChart::new(
                                &self.vram[idx],
                                data.total_vram,
                                color_front,
                                &self.gpu_usage[idx],
                                100,
                                color_back,
                            ),
                            format_gpu_tooltip(idx, data),
                            *aspect_ratio,
                        ),
                        PercentView::RunBack {
                            color,
                            aspect_ratio,
                        } => self.single_run_view(
                            SimpleHistoryChart::new(&self.gpu_usage[idx], 100, *color),
                            format_gpu_usage_tooltip(idx, data),
                            *aspect_ratio,
                        ),
                        PercentView::RunFront {
                            color,
                            aspect_ratio,
                        } => self.single_run_view(
                            SimpleHistoryChart::new(&self.vram[idx], data.total_vram, *color),
                            format_gpu_usage_tooltip(idx, data),
                            *aspect_ratio,
                        ),
                    })
                    .collect::<Vec<_>>()
            })
            .collect()
    }
}
