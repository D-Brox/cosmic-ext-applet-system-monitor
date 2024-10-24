// SPDX-License-Identifier: GPL-3.0-only

use std::{cmp::max, collections::VecDeque};

use crate::{
    app::Message,
    color::Color,
    config::{ChartConfig, Config},
};
use circular_queue::CircularQueue;
use cosmic::cosmic_theme::Layer;
use cosmic::iced::{
    alignment::{Horizontal, Vertical},
    widget::Row,
    Alignment, Length, Size,
};
use cosmic::iced_widget::canvas::{Cache, Frame};
use cosmic::widget::canvas::Geometry;
use cosmic::widget::{layer_container, Text};
use cosmic::{Apply, Element, Theme};
use plotters::prelude::*;
use plotters::style::Color as plottersColor;
use plotters_iced::{Chart, ChartBuilder, ChartWidget};
use sysinfo::{Disks, MemoryRefreshKind, Networks, System};

use crate::fl;

pub struct SystemMonitorChart {
    sys: System,
    nets: Networks,
    disks: Disks,

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
        let mut new_self = Self {
            sys: System::new(),
            nets: Networks::new_with_refreshed_list(),
            disks: Disks::new_with_refreshed_list(),
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
                .refresh_memory_specifics(MemoryRefreshKind::new().with_swap());
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
                .refresh_memory_specifics(MemoryRefreshKind::new().with_swap());
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
            self.nets.refresh();
            let mut upload = 0;
            let mut download = 0;

            for (_, data) in self.nets.iter() {
                upload += data.transmitted();
                download += data.received();
                // println!("{}", data.received());
            }

            let net = self.net.as_mut().expect("Error: uninitialized swap chart");
            net.push_data(upload, download);
            net.update_rgb_color(theme);
        }
    }

    pub fn update_disk(&mut self, theme: &Theme) {
        if self.is_initialized_disk() {
            self.disks.refresh();
            let mut write = 0;
            let mut read = 0;

            for disk in self.disks.iter() {
                write += disk.bytes_write();
                read += disk.bytes_read();
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
        self.charts = charts;
    }

    pub fn view(&self, size: f32, pad: f32) -> Element<Message> {
        let height = size + 2.0 * pad;
        let mut charts = Vec::new();
        for chart in &self.charts {
            match chart {
                UsedChart::Cpu => {
                    if self.is_initialized_cpu() {
                        let cpu_chart = self
                            .cpu
                            .as_ref()
                            .expect("Error: uninitialized CPU chart")
                            .view(height);
                        charts.push(cpu_chart);
                    }
                }
                UsedChart::Ram => {
                    if self.is_initialized_ram() {
                        let ram_chart = self
                            .ram
                            .as_ref()
                            .expect("Error: uninitialized RAM chart")
                            .view(height);
                        charts.push(ram_chart);
                    }
                }
                UsedChart::Swap => {
                    if self.is_initialized_swap() {
                        let swap_chart = self
                            .swap
                            .as_ref()
                            .expect("Error: uninitialized swap chart")
                            .view(height);
                        charts.push(swap_chart);
                    }
                }
                UsedChart::Net => {
                    if self.is_initialized_net() {
                        let net_chart = self
                            .net
                            .as_ref()
                            .expect("Error: uninitialized net chart")
                            .view(height);
                        charts.push(net_chart);
                    }
                }
                UsedChart::Disk => {
                    if self.is_initialized_disk() {
                        let disk_chart = self
                            .disk
                            .as_ref()
                            .expect("Error: uninitialized disk chart")
                            .view(height);
                        charts.push(disk_chart);
                    }
                } // UsedChart::Vram => {
                  //     if self.is_initialized_swap() {
                  //         let vram_chart = self
                  //             .vram
                  //             .as_ref()
                  //             .expect("Error: uninitialized swap chart")
                  //             .view(height);
                  //         charts.push(vram_chart);
                  //     }
                  // },
            }
        }
        if charts.is_empty() {
            return Text::new(fl!("loading"))
                .align_x(Horizontal::Center)
                .align_y(Vertical::Center)
                .into();
        }

        let row = Row::with_children(charts)
            .width(Length::Shrink)
            .height(Length::Shrink)
            .align_y(Alignment::Center)
            .spacing(pad / 2.0);
        row.into()
    }
}

struct SingleChart {
    cache: Cache,
    samples: usize,
    size: f32,

    data_points: CircularQueue<i64>,
    theme_color: Color,
    rgb_color: RGBColor,
}

impl SingleChart {
    fn new(theme_color: Color, size: f32, samples: usize, theme: &Theme) -> Self {
        let data_points = CircularQueue::with_capacity(samples);

        Self {
            cache: Cache::new(),
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

    fn update_rgb_color(&mut self, theme: &Theme) {
        self.rgb_color = self.theme_color.clone().as_rgb_color(theme);
    }

    fn update_colors(&mut self, color: Color, theme: &Theme) {
        self.theme_color = color;
        self.update_rgb_color(theme);
    }

    fn push_data(&mut self, value: i64) {
        self.data_points.push(value);
        self.cache.clear();
    }

    fn view(&self, height: f32) -> Element<Message> {
        ChartWidget::new(self)
            .height(Length::Fixed(height))
            .width(Length::Fixed(height * self.size))
            .apply(layer_container)
            .layer(Layer::Primary)
            .padding(0)
            .into()
    }
}

impl Chart<Message> for SingleChart {
    type State = ();

    #[inline]
    fn draw<R: plotters_iced::Renderer, F: Fn(&mut Frame)>(
        &self,
        renderer: &R,
        bounds: Size,
        draw_fn: F,
    ) -> Geometry {
        renderer.draw_cache(&self.cache, bounds, draw_fn)
    }

    fn build_chart<DB: DrawingBackend>(&self, _state: &Self::State, mut builder: ChartBuilder<DB>) {
        // Acquire time range
        let mut chart = builder
            .build_cartesian_2d(0..self.samples as i64, 0..100_i64)
            .expect("Error: failed to build chart");

        let mut data: VecDeque<_> = self.data_points.iter().collect();
        data.push_front(data.front().unwrap_or(&&0));
        data.push_back(data.back().unwrap_or(&&0));
        chart
            .draw_series(
                AreaSeries::new(
                    ((-1)..=self.samples as i64)
                        .rev()
                        .zip(data.iter().chain(std::iter::repeat(&&0)))
                        .map(|x| (x.0, **x.1)),
                    0,
                    self.rgb_color.mix(0.5),
                )
                .border_style(ShapeStyle::from(self.rgb_color).stroke_width(1)),
            )
            .expect("Error: failed to draw data series");
    }
}

struct DoubleChart {
    cache: Cache,
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
        let data_points1 = CircularQueue::with_capacity(samples);
        let data_points2 = CircularQueue::with_capacity(samples);

        Self {
            cache: Cache::new(),
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
        self.cache.clear();
    }

    fn view(&self, height: f32) -> Element<Message> {
        ChartWidget::new(self)
            .height(Length::Fixed(height))
            .width(Length::Fixed(height * self.size))
            .apply(layer_container)
            .layer(Layer::Primary)
            .padding(0)
            .into()
    }
}

impl Chart<Message> for DoubleChart {
    type State = ();

    #[inline]
    fn draw<R: plotters_iced::Renderer, F: Fn(&mut Frame)>(
        &self,
        renderer: &R,
        bounds: Size,
        draw_fn: F,
    ) -> Geometry {
        renderer.draw_cache(&self.cache, bounds, draw_fn)
    }

    fn build_chart<DB: DrawingBackend>(&self, _state: &Self::State, mut builder: ChartBuilder<DB>) {
        let mut chart = builder
            .build_cartesian_2d(0..self.samples as i64, 0..100_i64)
            .expect("Error: failed to build chart");

        let max = self
            .data_points1
            .iter()
            .zip(self.data_points2.iter())
            .fold(self.min_scale, |a, (&b, &c)| max(a, max(b, c)));
        let scale = 80.0 / max as f64;

        let mut data1: VecDeque<_> = self.data_points1.iter().collect();
        data1.push_front(data1.front().unwrap_or(&&0));
        data1.push_back(data1.back().unwrap_or(&&0));
        let series_iter1 = ((-1)..=self.samples as i64)
            .rev()
            .zip(data1.iter().chain(std::iter::repeat(&&0)))
            .map(|x| (x.0, (**x.1 as f64 * scale) as i64));

        let mut data2: VecDeque<_> = self.data_points2.iter().collect();
        data2.push_front(data2.front().unwrap_or(&&0));
        data2.push_back(data2.back().unwrap_or(&&0));
        let series_iter2 = ((-1)..=self.samples as i64)
            .rev()
            .zip(data2.iter().chain(std::iter::repeat(&&0)))
            .map(|x| (x.0, (**x.1 as f64 * scale) as i64));
        chart
            .draw_series(AreaSeries::new(
                series_iter1.clone(),
                0,
                self.rgb_color1.mix(0.5),
            ))
            .expect("Error: failed to draw data series");

        chart
            .draw_series(AreaSeries::new(
                series_iter2.clone(),
                0,
                self.rgb_color2.mix(0.5),
            ))
            .expect("Error: failed to draw data series");

        chart
            .draw_series(LineSeries::new(
                series_iter1,
                ShapeStyle::from(self.rgb_color1).stroke_width(1),
            ))
            .expect("Error: failed to draw data series");
        chart
            .draw_series(LineSeries::new(
                series_iter2,
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
