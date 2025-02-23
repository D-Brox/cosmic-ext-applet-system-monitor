// SPDX-License-Identifier: GPL-3.0-only

use std::cmp::max;

use crate::{
    applet::Message,
    color::Color,
    config::{ChartConfig, Config},
    // gpu::Gpus,
};
use circular_queue::CircularQueue;
use cosmic::iced::{
    alignment::{Horizontal, Vertical},
    Length,
};
use cosmic::widget::{layer_container, Text};
use cosmic::{Apply, Element, Theme};
use plotters::style::{Color as plottersColor, RelativeSize};
use plotters::{coord::Shift, prelude::*};
use plotters_iced::{Chart, ChartBuilder, ChartWidget};
use sysinfo::{Disks, MemoryRefreshKind, Networks, System};

use crate::fl;

pub struct SystemMonitorChart {
    relative_size: f32,
    breakpoints: Vec<f32>,
    bg_color: RGBAColor,

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

impl SystemMonitorChart {
    pub fn new(config: &Config, theme: &Theme) -> Self {
        let primary = theme.cosmic().primary.base;
        let rgb = primary.color.into_components();
        let r = (rgb.0 * 255.0) as u8;
        let g = (rgb.1 * 255.0) as u8;
        let b = (rgb.2 * 255.0) as u8;

        let bg_color = RGBAColor(r, g, b, primary.alpha as f64);
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

impl SystemMonitorChart {
    #[inline]
    fn is_initialized_cpu(&self) -> bool {
        self.cpu.is_some()
    }

    #[inline]
    fn is_initialized_ram(&self) -> bool {
        self.ram.is_some()
    }

    #[inline]
    fn is_initialized_swap(&self) -> bool {
        self.swap.is_some()
    }

    #[inline]
    fn is_initialized_net(&self) -> bool {
        self.net.is_some()
    }

    #[inline]
    fn is_initialized_disk(&self) -> bool {
        self.disk.is_some()
    }

    // #[inline]
    // fn is_initialized_vram(&self) -> bool {
    //     self.vram.is_some()
    // }

    pub fn update_cpu(&mut self, theme: &Theme) {
        if self.is_initialized_cpu() {
            self.sys.refresh_cpu_usage();
            let cpu_data = self.sys.global_cpu_usage() as i64;

            let cpu = self.cpu.as_mut().expect("Error: uninitialized CPU chart");
            cpu.push_data(cpu_data);
            cpu.update_rgb_color(theme);
        }
    }

    pub fn update_ram(&mut self, theme: &Theme) {
        if self.is_initialized_ram() {
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
        if self.is_initialized_swap() {
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
        if self.is_initialized_net() {
            self.nets.refresh(true);
            let mut upload = 0;
            let mut download = 0;

            for (_, data) in self.nets.iter() {
                upload += data.transmitted();
                download += data.received();
            }

            let net = self.net.as_mut().expect("Error: uninitialized swap chart");
            net.push_data(upload, download);
            net.update_rgb_color(theme);
        }
    }

    pub fn update_disk(&mut self, theme: &Theme) {
        if self.is_initialized_disk() {
            self.disks.refresh(true);
            let mut write = 0;
            let mut read = 0;

            for disk in self.disks.iter() {
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
    //     if self.is_initialized_vram() {}
    // }

    pub fn update_config(&mut self, config: &Config, theme: &Theme) {
        let mut charts = Vec::new();
        for chart in &config.charts {
            match chart {
                ChartConfig::CPU(c) => {
                    charts.push(UsedChart::Cpu);
                    if self.is_initialized_cpu() {
                        let cpu = self.cpu.as_mut().expect("Error: uninitialized CPU chart");
                        cpu.update_colors(c.color.clone(), theme);
                        cpu.resize_queue(c.samples);
                        cpu.update_size(c.size);
                    } else {
                        self.cpu =
                            Some(SingleChart::new(c.color.clone(), c.size, c.samples, theme));
                    }
                }
                ChartConfig::RAM(c) => {
                    charts.push(UsedChart::Ram);
                    if self.is_initialized_swap() {
                        let ram = self.ram.as_mut().expect("Error: uninitialized RAM chart");
                        ram.update_colors(c.color.clone(), theme);
                        ram.resize_queue(c.samples);
                        ram.update_size(c.size);
                    } else {
                        self.ram =
                            Some(SingleChart::new(c.color.clone(), c.size, c.samples, theme));
                    }
                }
                ChartConfig::Swap(c) => {
                    charts.push(UsedChart::Swap);
                    if self.is_initialized_swap() {
                        let swap = self.swap.as_mut().expect("Error: uninitialized swap chart");
                        swap.update_colors(c.color.clone(), theme);
                        swap.resize_queue(c.samples);
                        swap.update_size(c.size);
                    } else {
                        self.swap =
                            Some(SingleChart::new(c.color.clone(), c.size, c.samples, theme));
                    }
                }
                ChartConfig::Net(c) => {
                    charts.push(UsedChart::Net);
                    if self.is_initialized_net() {
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
                        ));
                    }
                }
                ChartConfig::Disk(c) => {
                    charts.push(UsedChart::Disk);
                    if self.is_initialized_disk() {
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
                        ));
                    }
                }
                ChartConfig::VRAM(_) => (),
                // ChartConfig::VRAM(c) => {
                //     charts.push(UsedChart::Vram);
                //     if self.is_initialized_vram() {
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
        for chart in self.charts.iter() {
            size += match chart {
                UsedChart::Cpu => {
                    if self.is_initialized_cpu() {
                        self.cpu.as_ref().unwrap().size
                    } else {
                        0.0
                    }
                }
                UsedChart::Ram => {
                    if self.is_initialized_ram() {
                        self.ram.as_ref().unwrap().size
                    } else {
                        0.0
                    }
                }
                UsedChart::Swap => {
                    if self.is_initialized_swap() {
                        self.swap.as_ref().unwrap().size
                    } else {
                        0.0
                    }
                }
                UsedChart::Net => {
                    if self.is_initialized_net() {
                        self.cpu.as_ref().unwrap().size
                    } else {
                        0.0
                    }
                }
                UsedChart::Disk => {
                    if self.is_initialized_disk() {
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

impl Chart<Message> for (&SystemMonitorChart, Vec<f32>, f32, bool) {
    type State = ();

    fn build_chart<DB: DrawingBackend>(&self, _state: &Self::State, _builder: ChartBuilder<DB>) {}

    fn draw_chart<DB: DrawingBackend>(&self, _state: &Self::State, root: DrawingArea<DB, Shift>) {
        let children = if self.3 {
            root.split_by_breakpoints(
                self.1
                    .iter()
                    .map(|bp| RelativeSize::Width(*bp as f64))
                    .collect::<Vec<_>>(),
                vec![0.0; 0],
            )
        } else {
            root.split_by_breakpoints(
                vec![0.0; 0],
                self.1
                    .iter()
                    .map(|bp| RelativeSize::Height(*bp as f64))
                    .collect::<Vec<_>>(),
            )
        };

        for (child, chart) in children.iter().zip(self.0.charts.clone().iter()) {
            let mut on = ChartBuilder::on(child);
            let builder = on.margin(self.2 / 4.0);

            match chart {
                UsedChart::Cpu => {
                    if self.0.is_initialized_cpu() {
                        self.0
                            .cpu
                            .clone()
                            .unwrap()
                            .draw_chart(builder, self.0.bg_color);
                    }
                }
                UsedChart::Ram => {
                    if self.0.is_initialized_ram() {
                        self.0
                            .ram
                            .clone()
                            .unwrap()
                            .draw_chart(builder, self.0.bg_color);
                    }
                }
                UsedChart::Swap => {
                    if self.0.is_initialized_swap() {
                        self.0
                            .swap
                            .clone()
                            .unwrap()
                            .draw_chart(builder, self.0.bg_color);
                    }
                }
                UsedChart::Net => {
                    if self.0.is_initialized_net() {
                        self.0
                            .net
                            .clone()
                            .unwrap()
                            .draw_chart(builder, self.0.bg_color);
                    }
                }
                UsedChart::Disk => {
                    if self.0.is_initialized_disk() {
                        self.0
                            .disk
                            .clone()
                            .unwrap()
                            .draw_chart(builder, self.0.bg_color);
                    }
                }
            }
        }
    }
}

#[derive(Clone)]
struct SingleChart {
    samples: usize,
    size: f32,

    data_points: CircularQueue<i64>,
    theme_color: Color,
    rgb_color: RGBColor,
}

impl SingleChart {
    fn new(theme_color: Color, size: f32, samples: usize, theme: &Theme) -> Self {
        let mut data_points = CircularQueue::with_capacity(samples);
        for _ in 0..samples {
            data_points.push(0);
        }

        Self {
            data_points,
            samples,
            rgb_color: theme_color.clone().as_rgb_color(theme),
            theme_color,
            size,
        }
    }

    fn resize_queue(&mut self, samples: usize) {
        let mut data_points = CircularQueue::with_capacity(samples);
        for data in self.data_points.asc_iter() {
            data_points.push(*data);
        }
        self.samples = samples;
        self.data_points = data_points;
    }

    fn update_size(&mut self, size: f32) {
        self.size = size;
    }

    fn update_rgb_color(&mut self, theme: &Theme) {
        self.rgb_color = self.theme_color.clone().as_rgb_color(theme);
    }

    fn update_colors(&mut self, color: Color, theme: &Theme) {
        self.theme_color = color;
        self.update_rgb_color(theme);
    }

    fn push_data(&mut self, value: i64) {
        self.data_points.push(value);
    }

    fn draw_chart<DB: DrawingBackend>(self, builder: &mut ChartBuilder<DB>, color: RGBAColor) {
        let mut chart = builder
            .build_cartesian_2d(0..self.samples as i64, 0..100_i64)
            .expect("Error: failed to build chart");

        chart.plotting_area().fill(&color).unwrap();
        let iter = (0..self.samples as i64)
            .zip(self.data_points.asc_iter())
            .map(|x| (x.0, *x.1));

        chart
            .draw_series(AreaSeries::new(iter.clone(), 0, self.rgb_color.mix(0.5)))
            .expect("Error: failed to draw data series");
        chart
            .draw_series(LineSeries::new(
                iter,
                ShapeStyle::from(self.rgb_color).stroke_width(1),
            ))
            .expect("Error: failed to draw data series");
    }
}

#[derive(Clone)]
struct DoubleChart {
    samples: usize,
    size: f32,
    min_scale: u64,

    data_points1: CircularQueue<u64>,
    theme_color1: Color,
    rgb_color1: RGBColor,

    data_points2: CircularQueue<u64>,
    theme_color2: Color,
    rgb_color2: RGBColor,
}

impl DoubleChart {
    fn new(
        theme_color1: Color,
        theme_color2: Color,
        size: f32,
        samples: usize,
        theme: &Theme,
        min_scale: u64,
    ) -> Self {
        let mut data_points1 = CircularQueue::with_capacity(samples);
        let mut data_points2 = CircularQueue::with_capacity(samples);
        for _ in 0..samples {
            data_points1.push(0);
            data_points2.push(0);
        }

        Self {
            samples,
            size,
            min_scale,

            data_points1,
            rgb_color1: theme_color1.clone().as_rgb_color(theme),
            theme_color1,

            data_points2,
            rgb_color2: theme_color2.clone().as_rgb_color(theme),
            theme_color2,
        }
    }

    fn resize_queue(&mut self, samples: usize) {
        let mut data_points1 = CircularQueue::with_capacity(samples);
        let mut data_points2 = CircularQueue::with_capacity(samples);
        for data in self.data_points1.asc_iter() {
            data_points1.push(*data);
        }
        for data in self.data_points2.asc_iter() {
            data_points2.push(*data);
        }
        self.samples = samples;
        self.data_points1 = data_points1;
        self.data_points2 = data_points2;
    }

    fn update_size(&mut self, size: f32) {
        self.size = size;
    }

    fn update_rgb_color(&mut self, theme: &Theme) {
        self.rgb_color1 = self.theme_color1.clone().as_rgb_color(theme);
        self.rgb_color2 = self.theme_color2.clone().as_rgb_color(theme);
    }

    fn update_colors(&mut self, color1: Color, color2: Color, theme: &Theme) {
        self.theme_color1 = color1;
        self.theme_color2 = color2;
        self.update_rgb_color(theme);
    }

    fn push_data(&mut self, value1: u64, value2: u64) {
        self.data_points1.push(value1);
        self.data_points2.push(value2);
    }

    fn draw_chart<DB: DrawingBackend>(self, builder: &mut ChartBuilder<DB>, color: RGBAColor) {
        let mut chart = builder
            .build_cartesian_2d(0..self.samples as i64, 0..100_i64)
            .expect("Error: failed to build chart");
        chart.plotting_area().fill(&color).unwrap();

        let max = self
            .data_points1
            .iter()
            .zip(self.data_points2.iter())
            .fold(self.min_scale, |a, (&b, &c)| max(a, max(b, c)));
        let scale = 80.0 / max as f64;

        let iter1 = (0..self.samples as i64)
            .zip(self.data_points2.asc_iter())
            .map(|x| (x.0, (*x.1 as f64 * scale) as i64));

        let iter2 = (0..self.samples as i64)
            .zip(self.data_points1.asc_iter())
            .map(|x| (x.0, (*x.1 as f64 * scale) as i64));

        chart
            .draw_series(AreaSeries::new(iter1.clone(), 0, self.rgb_color1.mix(0.5)))
            .expect("Error: failed to draw data series");

        chart
            .draw_series(AreaSeries::new(iter2.clone(), 0, self.rgb_color2.mix(0.5)))
            .expect("Error: failed to draw data series");

        chart
            .draw_series(LineSeries::new(
                iter1,
                ShapeStyle::from(self.rgb_color1).stroke_width(1),
            ))
            .expect("Error: failed to draw data series");
        chart
            .draw_series(LineSeries::new(
                iter2,
                ShapeStyle::from(self.rgb_color2).stroke_width(1),
            ))
            .expect("Error: failed to draw data series");
    }
}

#[derive(Clone)]
enum UsedChart {
    Cpu,
    Ram,
    Swap,
    Net,
    Disk,
    // Vram,
}
