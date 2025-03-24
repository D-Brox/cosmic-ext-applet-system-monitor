use std::cmp::max;

use circular_queue::CircularQueue;
use cosmic::Theme;
use plotters::style::{Color as plottersColor, RelativeSize};
use plotters::{coord::Shift, prelude::*};
use plotters_iced::{Chart, ChartBuilder};

use crate::applet::Message;
use crate::color::Color;
use crate::sysmon::SystemMonitor;

impl Chart<Message> for (&SystemMonitor, Vec<f32>, f32, bool) {
    type State = ();

    fn build_chart<DB: DrawingBackend>(&self, _state: &Self::State, _builder: ChartBuilder<DB>) {}

    fn draw_chart<DB: DrawingBackend>(&self, _state: &Self::State, root: DrawingArea<DB, Shift>) {
        let children = if self.3 {
            root.split_by_breakpoints(
                self.1
                    .iter()
                    .map(|bp| RelativeSize::Width(f64::from(*bp)))
                    .collect::<Vec<_>>(),
                vec![0.0; 0],
            )
        } else {
            root.split_by_breakpoints(
                vec![0.0; 0],
                self.1
                    .iter()
                    .map(|bp| RelativeSize::Height(f64::from(*bp)))
                    .collect::<Vec<_>>(),
            )
        };

        for (child, chart) in children.iter().zip(self.0.charts.iter()) {
            let mut on = ChartBuilder::on(child);
            let builder = on.margin(self.2 / 4.0);

            match chart {
                UsedChart::Cpu => {
                    match &self.0.cpu {
                        Some(data) => {
                            data.draw_chart(builder, self.0.bg_color);
                        }
                        _ => ()
                    }
                }
                UsedChart::Ram => {
                    match &self.0.ram {
                        Some(data) => {
                            data.draw_chart(builder, self.0.bg_color);
                        }
                        _ => (),
                    }
                }
                UsedChart::Swap => {
                    match &self.0.swap {
                        Some(data) => {
                            data.draw_chart(builder, self.0.bg_color);
                        }
                        _ => (),
                    }
                }
                UsedChart::Net => {
                    match &self.0.net {
                        Some(data) => {
                            data.draw_chart(builder, self.0.bg_color);
                        }
                        _ => (),
                    }
                }
                UsedChart::Disk => {
                    match &self.0.disk {
                        Some(data) => {
                            data.draw_chart(builder, self.0.bg_color);
                        }
                        _ => (),
                    }
                }
            }
        }
    }
}

#[derive(Clone)]
pub(super) struct SingleChart {
    samples: usize,
    pub size: f32,

    data_points: CircularQueue<i64>,
    theme_color: Color,
    rgb_color: RGBColor,
}

impl SingleChart {
    pub fn new(theme_color: Color, size: f32, samples: usize, theme: &Theme) -> Self {
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

    pub fn resize_queue(&mut self, samples: usize) {
        let mut data_points = CircularQueue::with_capacity(samples);
        for data in self.data_points.asc_iter() {
            data_points.push(*data);
        }
        self.samples = samples;
        self.data_points = data_points;
    }

    pub fn update_size(&mut self, size: f32) {
        self.size = size;
    }

    pub fn update_rgb_color(&mut self, theme: &Theme) {
        self.rgb_color = self.theme_color.clone().as_rgb_color(theme);
    }

    pub fn update_colors(&mut self, color: Color, theme: &Theme) {
        self.theme_color = color;
        self.update_rgb_color(theme);
    }

    pub fn push_data(&mut self, value: i64) {
        self.data_points.push(value);
    }

    fn draw_chart<DB: DrawingBackend>(&self, builder: &mut ChartBuilder<DB>, color: RGBAColor) {
        let mut chart = builder
            .build_cartesian_2d(0..self.samples as i64, 0..100_i64)
            .expect("Error: failed to build chart");

        chart.plotting_area().fill(&color).expect("Error: failed to fill chart backgournd");
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
pub(super) struct DoubleChart {
    samples: usize,
    pub size: f32,
    min_scale: u64,

    data_points1: CircularQueue<u64>,
    theme_color1: Color,
    rgb_color1: RGBColor,

    data_points2: CircularQueue<u64>,
    theme_color2: Color,
    rgb_color2: RGBColor,
}

impl DoubleChart {
    pub fn new(
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

    pub fn resize_queue(&mut self, samples: usize) {
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

    pub fn update_size(&mut self, size: f32) {
        self.size = size;
    }

    pub fn update_rgb_color(&mut self, theme: &Theme) {
        self.rgb_color1 = self.theme_color1.clone().as_rgb_color(theme);
        self.rgb_color2 = self.theme_color2.clone().as_rgb_color(theme);
    }

    pub fn update_colors(&mut self, color1: Color, color2: Color, theme: &Theme) {
        self.theme_color1 = color1;
        self.theme_color2 = color2;
        self.update_rgb_color(theme);
    }

    pub fn push_data(&mut self, value1: u64, value2: u64) {
        self.data_points1.push(value1);
        self.data_points2.push(value2);
    }

    fn draw_chart<DB: DrawingBackend>(&self, builder: &mut ChartBuilder<DB>, color: RGBAColor) {
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
pub enum UsedChart {
    Cpu,
    Ram,
    Swap,
    Net,
    Disk,
    // Vram,
}
