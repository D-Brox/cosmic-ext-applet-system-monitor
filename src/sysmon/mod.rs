// SPDX-License-Identifier: GPL-3.0-only
mod bar_chart;
mod chart;
mod cpu_chart;
mod viewable;
// mod gpu;
// use gpu::Gpus,
use crate::{
    applet::Message,
    config::{ChartConfig, Config},
};
use chart::{DoubleChart, SingleChart, UsedChart};

use crate::config::{ChartView, CpuView};
use crate::sysmon::chart::MonitorItem;
use crate::sysmon::cpu_chart::CpuChart;
use cosmic::iced::{alignment, Padding};
use cosmic::widget::{Column, Row};
use cosmic::{applet, Apply, Element, Theme};
use plotters::style::RGBAColor;
use sysinfo::{Disks, MemoryRefreshKind, Networks, System};

use crate::fl;

pub struct SystemMonitor {
    relative_size: f32,
    breakpoints: Vec<f32>,
    pub bg_color: RGBAColor,

    sys: System,
    nets: Networks,
    disks: Disks,
    // gpus: Gpus,
    charts: Vec<UsedChart>,
    cpu: Option<CpuChart>,
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
        if self.cpu.is_some() {
            self.sys.refresh_cpu_usage();
            let cpu_data = self.sys.global_cpu_usage() as i64;

            let cpu = self.cpu.as_mut().expect("Error: uninitialized CPU chart");
            cpu.update_latest_per_core(self.sys.cpus());
            cpu.push_data(cpu_data);
        }
    }

    pub fn update_ram(&mut self, theme: &Theme) {
        if self.ram.is_some() {
            self.sys
                .refresh_memory_specifics(MemoryRefreshKind::nothing().with_ram());
            let total_ram = self.sys.total_memory() as f64;
            let used_ram = self.sys.used_memory() as f64;
            let ram_data = ((used_ram / total_ram) * 100.0) as i64;

            let ram = self.ram.as_mut().expect("Error: uninitialized RAM chart");
            ram.push_data(ram_data);
            ram.update_rgb_color(theme);
        }
    }
    pub fn update_swap(&mut self, theme: &Theme) {
        if self.swap.is_some() {
            self.sys
                .refresh_memory_specifics(MemoryRefreshKind::nothing().with_swap());
            let total_swap = self.sys.total_swap() as f64;
            let used_swap = self.sys.used_swap() as f64;
            let ram_swap = ((used_swap / total_swap) * 100.0) as i64;

            let swap = self.swap.as_mut().expect("Error: uninitialized swap chart");
            swap.push_data(ram_swap);
            swap.update_rgb_color(theme);
        }
    }

    pub fn update_net(&mut self, theme: &Theme) {
        if self.net.is_some() {
            self.nets.refresh(true);
            let mut upload = 0;
            let mut download = 0;

            for (_, data) in &self.nets {
                upload += data.transmitted();
                download += data.received();
            }

            let net = self.net.as_mut().expect("Error: uninitialized swap chart");
            net.push_data(upload, download);
            net.update_rgb_color(theme);
        }
    }

    pub fn update_disk(&mut self, theme: &Theme) {
        if self.disk.is_some() {
            self.disks.refresh(true);
            let mut write = 0;
            let mut read = 0;

            for disk in &self.disks {
                let usage = disk.usage();
                write += usage.written_bytes;
                read += usage.read_bytes;
            }

            let disk = self.disk.as_mut().expect("Error: uninitialized disk chart");
            disk.push_data(write, read);
            disk.update_rgb_color(theme);
        }
    }

    // pub fn update_vram(&mut self, _theme: &Theme) {
    //     if self.vram.is_some() {}
    // }

    pub fn update_config(&mut self, config: &Config, theme: &Theme) {
        let mut charts = Vec::new();
        for chart in &config.charts {
            match chart {
                ChartConfig::CPU(c) => {
                    charts.push(UsedChart::Cpu);
                    if self.cpu.is_some() {
                        let cpu = self.cpu.as_mut().expect("Error: uninitialized CPU chart");
                        cpu.update_colors(c.color.clone(), theme);
                        cpu.resize_queue(c.samples);
                        cpu.update_size(c.size);
                    } else {
                        self.cpu = Some(CpuChart::new(
                            c.visualization,
                            c.color.clone(),
                            c.samples,
                            c.size,
                            theme,
                            self.sys.cpus(),
                        ))
                    }
                }
                ChartConfig::RAM(c) => {
                    charts.push(UsedChart::Ram);
                    if self.ram.is_some() {
                        let ram = self.ram.as_mut().expect("Error: uninitialized RAM chart");
                        ram.update_colors(c.color.clone(), theme);
                        ram.resize_queue(c.samples);
                        ram.update_size(c.size);
                    } else {
                        self.ram = Some(SingleChart::new(
                            c.color.clone(),
                            c.size,
                            c.samples,
                            theme,
                            c.visualization,
                        ));
                    }
                }
                ChartConfig::Swap(c) => {
                    charts.push(UsedChart::Swap);
                    if self.swap.is_some() {
                        let swap = self.swap.as_mut().expect("Error: uninitialized swap chart");
                        swap.update_colors(c.color.clone(), theme);
                        swap.resize_queue(c.samples);
                        swap.update_size(c.size);
                    } else {
                        self.swap = Some(SingleChart::new(
                            c.color.clone(),
                            c.size,
                            c.samples,
                            theme,
                            c.visualization,
                        ));
                    }
                }
                ChartConfig::Net(c) => {
                    charts.push(UsedChart::Net);
                    if self.net.is_some() {
                        let net = self.net.as_mut().expect("Error: uninitialized swap chart");
                        net.update_colors(c.color_up.clone(), c.color_down.clone(), theme);
                        net.resize_queue(c.samples);
                        net.update_size(c.size);
                    } else {
                        self.net = Some(DoubleChart::new(
                            c.color_up.clone(),
                            c.color_down.clone(),
                            c.size,
                            c.samples,
                            theme,
                            10 << 10,
                            c.visualization,
                        ));
                    }
                }
                ChartConfig::Disk(c) => {
                    charts.push(UsedChart::Disk);
                    if self.disk.is_some() {
                        let disk = self.disk.as_mut().expect("Error: uninitialized swap chart");
                        disk.update_colors(c.color_write.clone(), c.color_read.clone(), theme);
                        disk.resize_queue(c.samples);
                        disk.update_size(c.size);
                    } else {
                        self.disk = Some(DoubleChart::new(
                            c.color_write.clone(),
                            c.color_read.clone(),
                            c.size,
                            c.samples,
                            theme,
                            1 << 10,
                            c.visualization,
                        ));
                    }
                }
                ChartConfig::VRAM(_) => (),
                // ChartConfig::VRAM(c) => {
                //     charts.push(UsedChart::Vram);
                //     if self.vram.is_some() {
                //         let vram = self.vram.as_mut().expect("Error: uninitialized swap chart");
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
                UsedChart::Cpu => {
                    if self.cpu.is_some() {
                        self.cpu.as_ref().unwrap().size
                    } else {
                        0.0
                    }
                }
                UsedChart::Ram => {
                    if self.ram.is_some() {
                        self.ram.as_ref().unwrap().size
                    } else {
                        0.0
                    }
                }
                UsedChart::Swap => {
                    if self.swap.is_some() {
                        self.swap.as_ref().unwrap().size
                    } else {
                        0.0
                    }
                }
                UsedChart::Net => {
                    if self.net.is_some() {
                        self.cpu.as_ref().unwrap().size
                    } else {
                        0.0
                    }
                }
                UsedChart::Disk => {
                    if self.disk.is_some() {
                        self.disk.as_ref().unwrap().size
                    } else {
                        0.0
                    }
                }
            };
            breakpoints.push(size);
        }
        self.relative_size = size;
        if size != 0.0 {
            breakpoints.pop();
            self.breakpoints = breakpoints.iter().map(|bp| bp / size).collect();
        }
    }

    // old version
    /*    pub fn view(&self, size: f32, pad: f32, is_horizontal: bool) -> Element<Message> {
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
    }*/
    pub fn view(&self, context: &applet::Context) -> Element<Message> {
        let mut vec: Vec<Element<Message>> = Vec::with_capacity(5);

        if let Some(cpu) = &self.cpu {
            vec.push(cpu.view_as_configured(context));
            // vec.push(cpu.view_as(CpuView::PerCoreUsageHistogram, context));
            // vec.push(cpu.view_as(CpuView::GlobalUsageBarChart, context));
        }

        if let Some(ram) = &self.ram {
            vec.push(ram.view_as_configured(context));
            // vec.push(ram.view_as(ChartView::BarChart, context));
        }

        if let Some(swap) = &self.swap {
            vec.push(swap.view_as_configured(context));
            // vec.push(swap.view_as(ChartView::BarChart, context));
        }

        if let Some(net) = &self.net {
            vec.push(net.view_as_configured(context));
            // vec.push(net.view_as(ChartView::BarChart, context));
        }

        if let Some(disk) = &self.disk {
            vec.push(disk.view_as_configured(context));
            // vec.push(disk.view_as(ChartView::BarChart, context));
        }

        if context.is_horizontal() {
            Row::with_children(vec)
                .spacing(30)
                .align_y(alignment::Vertical::Bottom)
                .padding(Padding {
                    left: 200.0.into(),
                    ..Default::default()
                })
                .into()
        } else {
            Column::with_children(vec).spacing(30).into()
        }
    }
}
