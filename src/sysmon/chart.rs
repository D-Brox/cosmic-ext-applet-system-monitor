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
        let (monitor, breakpoints, pad, horizontal) = self;
        let children = if *horizontal {
            root.split_by_breakpoints(
                breakpoints
                    .iter()
                    .map(|bp| RelativeSize::Width(f64::from(*bp)))
                    .collect::<Vec<_>>(),
                vec![0.0; 0],
            )
        } else {
            root.split_by_breakpoints(
                vec![0.0; 0],
                breakpoints
                    .iter()
                    .map(|bp| RelativeSize::Height(f64::from(*bp)))
                    .collect::<Vec<_>>(),
            )
        };
        let mut gpu_iter = monitor.gpu.iter();
        for (child, chart) in children.iter().zip(monitor.charts.iter()) {
            let mut on = ChartBuilder::on(child);
            let builder = on.margin(pad / 4.0);

            _ = match chart {
                UsedChart::Cpu => &monitor
                    .cpu
                    .as_ref()
                    .map(|data| data.draw_chart(builder, monitor.bg_color)),
                UsedChart::Ram => &monitor
                    .ram
                    .as_ref()
                    .map(|data| data.draw_chart(builder, monitor.bg_color)),
                UsedChart::Swap => &monitor
                    .swap
                    .as_ref()
                    .map(|data| data.draw_chart(builder, monitor.bg_color)),
                UsedChart::Net => &monitor
                    .net
                    .as_ref()
                    .map(|data| data.draw_chart(builder, monitor.bg_color)),
                UsedChart::Disk => &monitor
                    .disk
                    .as_ref()
                    .map(|data| data.draw_chart(builder, monitor.bg_color)),
                UsedChart::Gpu => &gpu_iter
                    .next()
                    .map(|data| data.draw_chart(builder, monitor.bg_color)),
            }
        }
    }
}

#[derive(Clone)]
pub(super) struct SingleChart {
    samples: usize,
    pub aspect_ratio: f32,

    data_points: CircularQueue<u64>,
    theme_color: Color,
    rgb_color: RGBColor,
}

impl SingleChart {
    pub fn new(theme_color: Color, aspect_ratio: f32, samples: usize, theme: &Theme) -> Self {
        let mut data_points = CircularQueue::with_capacity(samples);
        for _ in 0..samples {
            data_points.push(0);
        }

        Self {
            data_points,
            samples,
            rgb_color: theme_color.clone().as_rgb_color(theme),
            theme_color,
            aspect_ratio,
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

    pub fn update_aspect_ratio(&mut self, size: f32) {
        self.aspect_ratio = size;
    }

    pub fn update_rgb_color(&mut self, theme: &Theme) {
        self.rgb_color = self.theme_color.clone().as_rgb_color(theme);
    }

    pub fn update_colors(&mut self, color: Color, theme: &Theme) {
        self.theme_color = color;
        self.update_rgb_color(theme);
    }

    pub fn push_data(&mut self, value: u64) {
        self.data_points.push(value);
    }

    fn draw_chart<DB: DrawingBackend>(&self, builder: &mut ChartBuilder<DB>, color: RGBAColor) {
        let mut chart = builder
            .build_cartesian_2d(0..self.samples as u64, 0..100_u64)
            .expect("Error: failed to build chart");

        chart
            .plotting_area()
            .fill(&color)
            .expect("Error: failed to fill chart backgournd");
        let iter = (0..self.samples as u64)
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
    pub aspect_ratio: f32,
    min_scale: Option<u64>,

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
        aspect_ratio: f32,
        samples: usize,
        theme: &Theme,
        min_scale: Option<u64>,
    ) -> Self {
        let mut data_points1 = CircularQueue::with_capacity(samples);
        let mut data_points2 = CircularQueue::with_capacity(samples);
        for _ in 0..samples {
            data_points1.push(0);
            data_points2.push(0);
        }

        Self {
            samples,
            aspect_ratio,
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

    pub fn update_aspect_ratio(&mut self, aspect_ratio: f32) {
        self.aspect_ratio = aspect_ratio;
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

        let scale = self.min_scale.map_or(1.0, |min_scale| {
            80.0 / self
                .data_points1
                .iter()
                .zip(self.data_points1.iter())
                .fold(min_scale, |a, (&b, &c)| max(a, max(b, c))) as f64
        });

        let iter1 = (0..self.samples as i64)
            .zip(self.data_points1.asc_iter())
            .map(|x| (x.0, (*x.1 as f64 * scale) as i64));

        let iter2 = (0..self.samples as i64)
            .zip(self.data_points2.asc_iter())
            .map(|x| (x.0, (*x.1 as f64 * scale) as i64));

        chart
            .draw_series(AreaSeries::new(iter1.clone(), 0, self.rgb_color1.mix(0.4)))
            .expect("Error: failed to draw data series");

        chart
            .draw_series(AreaSeries::new(iter2.clone(), 0, self.rgb_color2.mix(0.6)))
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
    Gpu,
}
