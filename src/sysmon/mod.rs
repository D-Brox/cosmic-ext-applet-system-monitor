// SPDX-License-Identifier: GPL-3.0-only
mod chart;
use chart::{DoubleChart, SingleChart, UsedChart};
use plotters_iced::ChartWidget;
// mod gpu;
// use gpu::Gpus,
use crate::{
    applet::Message,
    config::{ChartConfig, Config},
};

use cosmic::iced::{
    alignment::{Horizontal, Vertical},
    Length,
};
use cosmic::widget::{layer_container, Text};
use cosmic::{Apply, Element, Theme};

use plotters::style::RGBAColor;
use sysinfo::{Disks, MemoryRefreshKind, Networks, System};

use crate::fl;

pub struct SystemMonitor {
    /// used to calculate the aspect ratio of the entire collection in the panel. This is mereley a cache for the monolithic implementation of [Chart]
    relative_size: f32,
    breakpoints: Vec<f32>,
    /// background color for all the Charts
    pub bg_color: RGBAColor,

    sys: System,
    nets: Networks,
    disks: Disks,
    // gpus: Gpus,
    charts: Vec<UsedChart>,
    cpu: Option<SingleChart>,
    ram: Option<SingleChart>,
    swap: Option<SingleChart>,
    net: Option<DoubleChart>,
    disk: Option<DoubleChart>,
    // vram: Option<SingleChart>,
}

impl SystemMonitor {
    pub fn new(config: &Config, theme: &Theme) -> Self {
        let primary = theme.cosmic().primary.base;
        let rgb = primary.color.into_components();
        let r = (rgb.0 * 255.0) as u8;
        let g = (rgb.1 * 255.0) as u8;
        let b = (rgb.2 * 255.0) as u8;

        let bg_color = RGBAColor(r, g, b, f64::from(primary.alpha));
        let mut new_self = Self {
            relative_size: 0.0,
            breakpoints: Vec::new(),
            bg_color,

            sys: System::new(),
            nets: Networks::new_with_refreshed_list(),
            disks: Disks::new_with_refreshed_list(),
            // gpus: Gpus {},
            charts: vec![],
            cpu: None,
            ram: None,
            swap: None,
            net: None,
            disk: None,
            // vram: None,
        };
        new_self.update_config(config, theme);
        new_self.update_cpu(theme);
        new_self.update_ram(theme);
        new_self.update_swap(theme);
        new_self.update_net(theme);
        new_self.update_disk(theme);
        // new_self.update_vram(theme);
        new_self.update_size();
        new_self
    }
}

impl SystemMonitor {
    pub fn update_cpu(&mut self, theme: &Theme) {
        if let Some(cpu) = &mut self.cpu {
            self.sys.refresh_cpu_usage();
            let cpu_data = self.sys.global_cpu_usage() as i64;

            cpu.push_data(cpu_data);
            cpu.update_rgb_color(theme);
        }
    }

    pub fn update_ram(&mut self, theme: &Theme) {
        if let Some(ram) = &mut self.ram {
            self.sys
                .refresh_memory_specifics(MemoryRefreshKind::nothing().with_ram());
            let total_ram = self.sys.total_memory() as f64;
            let used_ram = self.sys.used_memory() as f64;
            let ram_data = ((used_ram / total_ram) * 100.0) as i64;

            ram.push_data(ram_data);
            ram.update_rgb_color(theme);
        }
    }
    pub fn update_swap(&mut self, theme: &Theme) {
        if let Some(swap) = &mut self.swap {
            self.sys
                .refresh_memory_specifics(MemoryRefreshKind::nothing().with_swap());
            let total_swap = self.sys.total_swap() as f64;
            let used_swap = self.sys.used_swap() as f64;
            let ram_swap = ((used_swap / total_swap) * 100.0) as i64;

            swap.push_data(ram_swap);
            swap.update_rgb_color(theme);
        }
    }

    pub fn update_net(&mut self, theme: &Theme) {
        if let Some(net) = &mut self.net {
            self.nets.refresh(true);
            let mut upload = 0;
            let mut download = 0;

            for (_, data) in &self.nets {
                upload += data.transmitted();
                download += data.received();
            }

            net.push_data(upload, download);
            net.update_rgb_color(theme);
        }
    }

    pub fn update_disk(&mut self, theme: &Theme) {
        if let Some(disk) = &mut self.disk {
            self.disks.refresh(true);
            let mut write = 0;
            let mut read = 0;

            for disk in &self.disks {
                let usage = disk.usage();
                write += usage.written_bytes;
                read += usage.read_bytes;
            }

            disk.push_data(write, read);
            disk.update_rgb_color(theme);
        }
    }

    // pub fn update_vram(&mut self, _theme: &Theme) {
    //     if let Some(vram) = self.vram {}
    // }

    pub fn update_config(&mut self, config: &Config, theme: &Theme) {
        let mut charts = Vec::new();
        for chart in &config.charts {
            match chart {
                ChartConfig::CPU(c) => {
                    charts.push(UsedChart::Cpu);
                    if let Some(cpu) = &mut self.cpu {
                        cpu.update_colors(c.color.clone(), theme);
                        cpu.resize_queue(c.samples);
                        cpu.update_aspect_ratio(c.aspect_ratio);
                    } else {
                        self.cpu = Some(SingleChart::new(
                            c.color.clone(),
                            c.aspect_ratio,
                            c.samples,
                            theme,
                        ));
                    }
                }
                ChartConfig::RAM(c) => {
                    charts.push(UsedChart::Ram);
                    if let Some(ram) = &mut self.ram {
                        ram.update_colors(c.color.clone(), theme);
                        ram.resize_queue(c.samples);
                        ram.update_aspect_ratio(c.aspect_ratio);
                    } else {
                        self.ram = Some(SingleChart::new(
                            c.color.clone(),
                            c.aspect_ratio,
                            c.samples,
                            theme,
                        ));
                    }
                }
                ChartConfig::Swap(c) => {
                    charts.push(UsedChart::Swap);
                    if let Some(swap) = &mut self.swap {
                        swap.update_colors(c.color.clone(), theme);
                        swap.resize_queue(c.samples);
                        swap.update_aspect_ratio(c.aspect_ratio);
                    } else {
                        self.swap = Some(SingleChart::new(
                            c.color.clone(),
                            c.aspect_ratio,
                            c.samples,
                            theme,
                        ));
                    }
                }
                ChartConfig::Net(c) => {
                    charts.push(UsedChart::Net);
                    if let Some(net) = &mut self.net {
                        net.update_colors(c.color_up.clone(), c.color_down.clone(), theme);
                        net.resize_queue(c.samples);
                        net.update_aspect_ratio(c.aspect_ratio);
                    } else {
                        self.net = Some(DoubleChart::new(
                            c.color_up.clone(),
                            c.color_down.clone(),
                            c.aspect_ratio,
                            c.samples,
                            theme,
                            10 << 10,
                        ));
                    }
                }
                ChartConfig::Disk(c) => {
                    charts.push(UsedChart::Disk);
                    if let Some(disk) = &mut self.disk {
                        disk.update_colors(c.color_write.clone(), c.color_read.clone(), theme);
                        disk.resize_queue(c.samples);
                        disk.update_aspect_ratio(c.aspect_ratio);
                    } else {
                        self.disk = Some(DoubleChart::new(
                            c.color_write.clone(),
                            c.color_read.clone(),
                            c.aspect_ratio,
                            c.samples,
                            theme,
                            1 << 10,
                        ));
                    }
                }
                ChartConfig::VRAM(_) => (),
                // ChartConfig::VRAM(c) => {
                //     charts.push(UsedChart::Vram);
                // if let Some(vram) = &mut self.vram {
                //         vram.update_colors(c.color.clone(), theme);
                //         vram.resize_queue(c.samples);
                //     } else {
                //         self.vram = Some(SingleChart::new(
                //             c.color.clone(),
                //             c.size,
                //             c.samples,
                //             theme,
                //         ));
                //     }
                // },
            }
        }
        self.update_size();
        println!("{}", self.relative_size);
        self.charts = charts;
    }

    fn update_size(&mut self) {
        let mut size = 0.0;
        let mut breakpoints = Vec::new();
        for chart in &self.charts {
            size += match chart {
                UsedChart::Cpu => self.cpu.as_ref().map_or(0.0, |chart| (chart.aspect_ratio)),
                UsedChart::Ram => self.ram.as_ref().map_or(0.0, |chart| (chart.aspect_ratio)),
                UsedChart::Swap => self.swap.as_ref().map_or(0.0, |chart| (chart.aspect_ratio)),
                UsedChart::Net => self.net.as_ref().map_or(0.0, |chart| (chart.aspect_ratio)),
                UsedChart::Disk => self.disk.as_ref().map_or(0.0, |chart| (chart.aspect_ratio)),
            };
            breakpoints.push(size);
        }
        self.relative_size = size;
        if size != 0.0 {
            breakpoints.pop();
            self.breakpoints = breakpoints.iter().map(|bp| bp / size).collect();
        }
    }

    pub fn view(&self, size: f32, pad: f32, is_horizontal: bool) -> Element<Message> {
        if self.charts.is_empty() {
            return Text::new(fl!("loading"))
                .align_x(Horizontal::Center)
                .align_y(Vertical::Center)
                .into();
        }

        let (height, width) = if is_horizontal {
            let h = size + 2.0 * pad;
            (h, h * self.relative_size)
        } else {
            let w = size + 2.0 * pad;
            (w * self.relative_size, w)
        };

        ChartWidget::new((self, self.breakpoints.clone(), pad, is_horizontal))
            .height(Length::Fixed(height))
            .apply(layer_container)
            .width(Length::Fixed(width))
            .padding(0)
            .into()
    }
}
