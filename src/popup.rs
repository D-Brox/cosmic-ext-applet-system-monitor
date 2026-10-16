// SPDX-License-Identifier: GPL-3.0-only

//! Popup shown when `popup_mode` is enabled: one section per component,
//! each with a header, a larger run chart and a color legend.

use cosmic::{
    Element, Theme,
    iced::{Alignment, Length, Size},
    widget::{Column, Row, button, container, icon, text},
};

use crate::{
    fl,
    applet::{Message, SystemMonitorApplet, base_background},
    color::Color,
    components::{
        gpu::GpuData,
        storage::Drive,
        run::{SimpleHistoryChart, SuperimposedHistoryChart},
    },
    config::{ComponentConfig, CpuView, IoView, PercentView},
};

const POPUP_WIDTH: f32 = 340.0;
const CHART_SIZE: Size = Size {
    width: POPUP_WIDTH,
    height: 44.0,
};

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
        format!("{bytes} {}", UNITS[0])
    } else {
        format!("{size:.1} {}", UNITS[unit_index])
    }
}

#[allow(clippy::cast_precision_loss)]
fn percent(used: u64, total: u64) -> f64 {
    if total == 0 {
        0.0
    } else {
        used as f64 / total as f64 * 100.0
    }
}

fn cpu_color(vis: &[CpuView]) -> Color {
    match vis.first() {
        Some(
            CpuView::Run { color, .. }
            | CpuView::BarGlobal { color, .. }
            | CpuView::BarCores { color, .. },
        ) => *color,
        None => Color::accent_blue,
    }
}

/// (back, front) colors
fn percent_colors(vis: &[PercentView], default: (Color, Color)) -> (Color, Color) {
    match vis.first() {
        Some(
            PercentView::Run {
                color_back,
                color_front,
                ..
            },
        ) => (*color_back, *color_front),
        Some(
            PercentView::Bar {
                color_left,
                color_right,
                ..
            },
        ) => (*color_left, *color_right),
        Some(
            PercentView::RunBack { color, .. }
            | PercentView::RunFront { color, .. }
            | PercentView::BarLeft { color, .. }
            | PercentView::BarRight { color, .. },
        ) => (*color, *color),
        None => default,
    }
}

/// (back, front) colors
fn io_colors(vis: &[IoView], default: (Color, Color)) -> (Color, Color) {
    match vis.first() {
        Some(IoView::Run {
            color_back,
            color_front,
            ..
        }) => (*color_back, *color_front),
        Some(IoView::RunBack { color, .. } | IoView::RunFront { color, .. }) => (*color, *color),
        None => default,
    }
}

fn legend<'a>(color: Color, label: String) -> Element<'a, Message> {
    let swatch = container(Column::new()).width(10).height(10).style(move |theme: &Theme| {
        container::Style {
            background: Some(cosmic::iced::Color::from(color.as_cosmic_color(theme)).into()),
            border: cosmic::iced::Border {
                radius: 2.0.into(),
                ..Default::default()
            },
            ..container::Style::default()
        }
    });
    Row::new()
        .push(swatch)
        .push(text::caption(label))
        .spacing(6)
        .align_y(Alignment::Center)
        .into()
}

/// Thin horizontal bar filled to `used / total`
fn usage_bar<'a>(used: u64, total: u64, color: Color) -> Element<'a, Message> {
    #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
    let filled = (percent(used, total) * 10.0).round().clamp(0.0, 1000.0) as u16;
    let fill = container(Column::new())
        .width(Length::FillPortion(filled.max(1)))
        .height(Length::Fill)
        .style(move |theme: &Theme| container::Style {
            background: Some(cosmic::iced::Color::from(color.as_cosmic_color(theme)).into()),
            border: cosmic::iced::Border {
                radius: 2.0.into(),
                ..Default::default()
            },
            ..container::Style::default()
        });
    let rest = container(Column::new())
        .width(Length::FillPortion((1000 - filled).max(1)))
        .height(Length::Fill);
    container(Row::new().push(fill).push(rest))
        .width(Length::Fill)
        .height(5)
        .style(base_background)
        .into()
}

fn drive_row<'a>(drive: &Drive, color: Color) -> Element<'a, Message> {
    let label = format!("{} ({})", drive.model, drive.name);
    let usage = match drive.used {
        Some(used) => format!("{} / {}", format_bytes(used), format_bytes(drive.size)),
        None => fl!("drive-not-mounted", size = format_bytes(drive.size)),
    };
    let mut column = Column::new()
        .push(
            Row::new()
                .push(container(text::caption(label)).width(Length::Fill))
                .push(text::caption(usage))
                .spacing(8),
        )
        .spacing(3);
    if let Some(used) = drive.used {
        column = column.push(usage_bar(used, drive.size, color));
    }
    column.into()
}

const ARROW_DOWN: &[u8] = include_bytes!("../res/arrow-down-symbolic.svg");
const ARROW_UP: &[u8] = include_bytes!("../res/arrow-up-symbolic.svg");

impl SystemMonitorApplet {
    /// A section with a clickable header. Sections start collapsed and only show the
    /// header with the current value, which keeps the popup short on smaller screens.
    fn section<'a>(
        &self,
        key: String,
        title: String,
        value: String,
        chart: Element<'a, Message>,
        legends: Vec<Element<'a, Message>>,
    ) -> Element<'a, Message> {
        let collapsed = !self.config.popup_expanded.contains(&key);
        // Arrow points down while the section is collapsed and up when it is open
        let arrow = icon::from_svg_bytes(if collapsed { ARROW_DOWN } else { ARROW_UP })
            .symbolic(true)
            .icon()
            .size(16);
        let header = Row::new()
            .push(container(text::heading(title)).width(Length::Fill))
            .push(text::heading(value))
            .push(arrow)
            .spacing(8)
            .align_y(Alignment::Center);
        let header = button::custom(header)
            .class(cosmic::theme::Button::Text)
            .padding([4, 6])
            .width(Length::Fill)
            .on_press(Message::ToggleExpanded(key));

        let mut column = Column::new().push(header).spacing(4).width(POPUP_WIDTH);
        if !collapsed {
            let chart = container(chart)
                .width(CHART_SIZE.width)
                .height(CHART_SIZE.height)
                .style(base_background);
            column = column
                .push(chart)
                .push(Column::with_children(legends).spacing(2));
        }
        column.into()
    }

    pub fn popup_view(&'_ self) -> Element<'_, Message> {
        let sections: Vec<Element<'_, Message>> = self
            .config
            .components
            .iter()
            .flat_map(|component| match component {
                ComponentConfig::Cpu(vis) => vec![self.cpu_section(cpu_color(vis))],
                ComponentConfig::Mem(vis) => vec![self.mem_section(percent_colors(
                    vis,
                    (Color::accent_green, Color::accent_purple),
                ))],
                ComponentConfig::Net(vis) => vec![self.net_section(io_colors(
                    vis,
                    (Color::accent_red, Color::accent_yellow),
                ))],
                ComponentConfig::Disk(vis) => vec![self.disk_section(io_colors(
                    vis,
                    (Color::accent_orange, Color::accent_pink),
                ))],
                ComponentConfig::Gpu(vis) => {
                    let colors =
                        percent_colors(vis, (Color::accent_warm_grey, Color::accent_indigo));
                    self.gpus
                        .data()
                        .into_iter()
                        .enumerate()
                        .map(|(idx, data)| self.gpu_section(idx, data, colors))
                        .collect()
                }
            })
            .collect();

        Column::with_children(sections)
            .spacing(8)
            .padding(12)
            .into()
    }

    fn cpu_section(&'_ self, color: Color) -> Element<'_, Message> {
        let usage = self.sys.global_cpu_usage();
        self.section(
            "cpu".into(),
            fl!("cpu"),
            format!("{usage:.1}%"),
            SimpleHistoryChart::new(&self.global_cpu, 100.0, color).into(),
            vec![legend(
                color,
                fl!("cpu-usage", threads = self.sys.cpus().len()),
            )],
        )
    }

    fn mem_section(&'_ self, (back, front): (Color, Color)) -> Element<'_, Message> {
        let (used, total) = (self.sys.used_memory(), self.sys.total_memory());
        let (swap_used, swap_total) = (self.sys.used_swap(), self.sys.total_swap());
        let swap_label = if swap_total == 0 {
            fl!("swap-unavailable")
        } else {
            fl!(
                "swap-usage",
                used = format_bytes(swap_used),
                total = format_bytes(swap_total)
            )
        };
        self.section(
            "mem".into(),
            fl!("memory"),
            format!("{:.1}%", percent(used, total)),
            SuperimposedHistoryChart::new(
                &self.swap, swap_total, &front, &self.ram, total, &back,
            )
            .into(),
            vec![
                legend(
                    back,
                    fl!("ram-usage", used = format_bytes(used), total = format_bytes(total)),
                ),
                legend(front, swap_label),
            ],
        )
    }

    /// Histories hold bytes per tick, convert the latest one to bytes per second
    fn rate(history: &crate::history::History, interval_ms: u64) -> u64 {
        let last = history.iter().last().copied().unwrap_or(0);
        last.saturating_mul(1000) / interval_ms.max(1)
    }

    fn net_section(&'_ self, (back, front): (Color, Color)) -> Element<'_, Message> {
        let interval = self.config.sampling.net.update_interval;
        let down = Self::rate(&self.download, interval);
        let up = Self::rate(&self.upload, interval);
        self.section(
            "net".into(),
            fl!("network"),
            format!("↓ {}/s  ↑ {}/s", format_bytes(down), format_bytes(up)),
            SuperimposedHistoryChart::new_linked(&self.upload, &front, &self.download, &back)
                .into(),
            vec![
                legend(back, fl!("download-rate", rate = format_bytes(down))),
                legend(front, fl!("upload-rate", rate = format_bytes(up))),
            ],
        )
    }

    fn disk_section(&'_ self, (back, front): (Color, Color)) -> Element<'_, Message> {
        let interval = self.config.sampling.disk.update_interval;
        let read = Self::rate(&self.disk_read, interval);
        let write = Self::rate(&self.disk_write, interval);

        let mut rows = vec![
            legend(back, fl!("read-rate", rate = format_bytes(read))),
            legend(front, fl!("write-rate", rate = format_bytes(write))),
        ];
        if !self.drives.is_empty() {
            let total: u64 = self.drives.iter().map(|d| d.size).sum();
            rows.push(
                Row::new()
                    .push(container(text::caption_heading(fl!("drives"))).width(Length::Fill))
                    .push(text::caption(fl!("drives-total", size = format_bytes(total))))
                    .padding(cosmic::iced::padding::top(6.0))
                    .into(),
            );
            // Drives in use first, the rest behind an expandable row
            let (mounted, others): (Vec<&Drive>, Vec<&Drive>) =
                self.drives.iter().partition(|d| d.used.is_some());
            rows.extend(mounted.into_iter().map(|drive| drive_row(drive, back)));
            if !others.is_empty() {
                let expanded = self.config.popup_expanded.iter().any(|k| k == "disk-others");
                let arrow = icon::from_svg_bytes(if expanded { ARROW_UP } else { ARROW_DOWN })
                    .symbolic(true)
                    .icon()
                    .size(14);
                let toggle = Row::new()
                    .push(
                        container(text::caption(fl!("other-drives", count = others.len())))
                            .width(Length::Fill),
                    )
                    .push(arrow)
                    .align_y(Alignment::Center);
                rows.push(
                    button::custom(toggle)
                        .class(cosmic::theme::Button::Text)
                        .padding([2, 0])
                        .width(Length::Fill)
                        .on_press(Message::ToggleExpanded("disk-others".into()))
                        .into(),
                );
                if expanded {
                    rows.extend(others.into_iter().map(|drive| drive_row(drive, back)));
                }
            }
        }
        self.section(
            "disk".into(),
            fl!("disk"),
            fl!(
                "disk-summary",
                read = format_bytes(read),
                write = format_bytes(write)
            ),
            SuperimposedHistoryChart::new_linked(&self.disk_write, &front, &self.disk_read, &back)
                .into(),
            rows,
        )
    }

    fn gpu_section(
        &'_ self,
        idx: usize,
        data: GpuData,
        (back, front): (Color, Color),
    ) -> Element<'_, Message> {
        let title = if self.gpus.num_gpus() > 1 {
            fl!("gpu-numbered", index = idx)
        } else {
            fl!("gpu")
        };
        self.section(
            format!("gpu{idx}"),
            title,
            format!("{}%", data.usage),
            SuperimposedHistoryChart::new(
                &self.vram[idx],
                data.total_vram,
                &front,
                &self.gpu_usage[idx],
                100,
                &back,
            )
            .into(),
            vec![
                legend(back, fl!("gpu-usage", usage = data.usage)),
                legend(
                    front,
                    fl!(
                        "gpu-vram",
                        used = format_bytes(data.used_vram),
                        total = format_bytes(data.total_vram),
                        percent = format!("{:.1}", percent(data.used_vram, data.total_vram))
                    ),
                ),
            ],
        )
    }
}
