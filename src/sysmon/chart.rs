// use crate::helpers::base_background;
// use crate::sysmon::monitor_item::MonitorItem;
// use std::cmp::max;

// use crate::applet::Message;
// use crate::color::{plot_color, Color};
// use crate::config::{self, ChartView, DoubleChartView, DoubleDataConfig};
// use crate::sysmon::bar_chart::PercentageBar;
// use circular_queue::CircularQueue;
// use cosmic::cosmic_theme::palette::WithAlpha;
// use cosmic::iced_widget::container;
// use cosmic::theme::CosmicColor;
// use cosmic::{applet, Apply, Element, Theme};
// use plotters::prelude::*;
// use plotters_iced::{Chart, ChartBuilder, ChartWidget};

// #[derive(Clone)]
// pub(super) struct SingleData {
//     samples: usize,
//     pub aspect_ratio: f32,

//     data_points: CircularQueue<i64>,
//     color: CosmicColor,

//     visualization: Vec<ChartView>,
// }

// impl MonitorItem for SingleData {
//     type ViewEnum = ChartView;
//     type ConfigStruct = config::Generic;

//     fn new(c: Self::ConfigStruct, theme: &Theme) -> Self {
//         let mut data_points = CircularQueue::with_capacity(c.samples);
//         for _ in 0..c.samples {
//             data_points.push(0);
//         }
//         let color = c.color.as_cosmic_color(theme);

//         Self {
//             data_points,
//             samples: c.samples,
//             color,
//             aspect_ratio: c.aspect_ratio,
//             visualization: c.visualization,
//         }
//     }

//     fn view_single(&self, chart_view: &ChartView, context: &applet::Context) -> Element<Message> {
//         let (suggested_width, suggested_height) = context.suggested_size(false);

//         match chart_view {
//             ChartView::RunChart => ChartWidget::new(self)
//                 .width(suggested_width.into())
//                 .height(suggested_height.into())
//                 .apply(container)
//                 .style(base_background)
//                 .into(),
//             ChartView::BarChart => PercentageBar::new(
//                 context.is_horizontal(),
//                 *self.data_points.iter().next().unwrap() as f32,
//                 suggested_width,
//                 suggested_height,
//                 self.color,
//             )
//             .apply(container)
//             .style(base_background)
//             .into(),
//         }
//     }
//     fn view_order(&self) -> &[ChartView] {
//         self.visualization.as_slice()
//     }

//     fn resize_queue(&mut self, samples: usize) {
//         let mut data_points = CircularQueue::with_capacity(samples);
//         for data in self.data_points.asc_iter() {
//             data_points.push(*data);
//         }
//         self.samples = samples;
//         self.data_points = data_points;
//     }

//     fn update_aspect_ratio(&mut self, size: f32) {
//         self.aspect_ratio = size;
//     }
// }

// impl Chart<Message> for SingleData {
//     type State = ();

//     fn build_chart<DB: DrawingBackend>(&self, _state: &Self::State, mut builder: ChartBuilder<DB>) {
//         let mut chart = builder
//             .build_cartesian_2d(0..self.samples as i64, 0..100_i64)
//             .expect("Error: failed to build chart");

//         // fill background moved to the ChartWidget that contains this chart

//         let iter = (0..self.samples as i64)
//             .zip(self.data_points.asc_iter())
//             .map(|x| (x.0, *x.1));

//         chart
//             .draw_series(AreaSeries::new(
//                 iter.clone(),
//                 0,
//                 plot_color(self.color.with_alpha(0.5)), // ShapeStyle::from(self.color.with_alpha(0.5).into_format()),
//             ))
//             .expect("Error: failed to draw data series");
//         chart
//             .draw_series(LineSeries::new(
//                 iter,
//                 ShapeStyle {
//                     color: plot_color(self.color),
//                     stroke_width: 1,
//                     filled: true,
//                 }, // ShapeStyle::from(self.color.into_format()).stroke_width(1),
//             ))
//             .expect("Error: failed to draw data series");
//     }
// }

// impl SingleData {
//     pub fn update_colors(&mut self, color: Color, theme: &Theme) {
//         self.color = color.as_cosmic_color(theme);
//     }

//     pub fn push_data(&mut self, value: i64) {
//         self.data_points.push(value);
//     }
// }

// #[derive(Clone)]
// pub(super) struct DoubleData {
//     samples: usize,
//     pub aspect_ratio: f32,
//     min_scale: u64,

//     data_points1: CircularQueue<u64>,
//     color1: CosmicColor,

//     data_points2: CircularQueue<u64>,
//     color2: CosmicColor,

//     visualization: Vec<DoubleChartView>,
// }

// impl MonitorItem for DoubleData {
//     type ViewEnum = DoubleChartView;
//     type ConfigStruct = DoubleDataConfig;

//     fn new(c: DoubleDataConfig, theme: &Theme) -> Self {
//         let mut data_points1 = CircularQueue::with_capacity(c.samples);
//         let mut data_points2 = CircularQueue::with_capacity(c.samples);
//         for _ in 0..c.samples {
//             data_points1.push(0);
//             data_points2.push(0);
//         }

//         Self {
//             samples: c.samples,
//             aspect_ratio: c.aspect_ratio,
//             min_scale: 1 << 10,
//             data_points1,
//             color1: c.color1.as_cosmic_color(theme),
//             data_points2,
//             color2: c.color2.as_cosmic_color(theme),

//             visualization: c.visualization,
//         }
//     }

//     fn view_single(&self, chart_view: &Self::ViewEnum, context: &applet::Context) -> Element<Message> {
//         let (suggested_width, suggested_height) = context.suggested_size(false);

//         match chart_view {
//             Self::ViewEnum::SuperimposedRunChart => ChartWidget::new(self)
//                 .width(suggested_width.into())
//                 .height(suggested_height.into())
//                 .apply(container),
//         }
//         .style(base_background)
//         .into()
//     }
//     fn view_order(&self) -> &[Self::ViewEnum] {
//         self.visualization.as_slice()
//     }

//     fn resize_queue(&mut self, samples: usize) {
//         let mut data_points1 = CircularQueue::with_capacity(samples);
//         let mut data_points2 = CircularQueue::with_capacity(samples);
//         for data in self.data_points1.asc_iter() {
//             data_points1.push(*data);
//         }
//         for data in self.data_points2.asc_iter() {
//             data_points2.push(*data);
//         }
//         self.samples = samples;
//         self.data_points1 = data_points1;
//         self.data_points2 = data_points2;
//     }

//     fn update_aspect_ratio(&mut self, aspect_ratio: f32) {
//         self.aspect_ratio = aspect_ratio;
//     }
// }

// pub fn resize(new_size: usize, history: CircularQueue<i64>) -> CircularQueue<i64> {
//     let mut new = CircularQueue::with_capacity(new_size);
//     for data in history.asc_iter() {
//         new.push(*data);
//     }
//     new
// }

// impl DoubleData {
//     pub fn update_colors(&mut self, color1: Color, color2: Color, theme: &Theme) {
//         self.color1 = color1.as_cosmic_color(theme);
//         self.color2 = color2.as_cosmic_color(theme);
//     }

//     pub fn push_data(&mut self, value1: u64, value2: u64) {
//         self.data_points1.push(value1);
//         self.data_points2.push(value2);
//     }
// }

// impl Chart<Message> for DoubleData {
//     type State = ();

//     fn build_chart<DB: DrawingBackend>(&self, _state: &Self::State, mut builder: ChartBuilder<DB>) {
//         let mut chart = builder
//             .build_cartesian_2d(0..self.samples as i64, 0..100_i64)
//             .expect("Error: failed to build chart");

//         let max = self
//             .data_points1
//             .iter()
//             .zip(self.data_points2.iter())
//             .fold(self.min_scale, |a, (&b, &c)| max(a, max(b, c)));
//         let scale = 80.0 / max as f64;

//         let iter1 = (0..self.samples as i64)
//             .zip(self.data_points2.asc_iter())
//             .map(|x| (x.0, (*x.1 as f64 * scale) as i64));

//         let iter2 = (0..self.samples as i64)
//             .zip(self.data_points1.asc_iter())
//             .map(|x| (x.0, (*x.1 as f64 * scale) as i64));

//         chart
//             .draw_series(AreaSeries::new(
//                 iter1.clone(),
//                 0,
//                 plot_color(self.color1.with_alpha(0.5)),
//             ))
//             .expect("Error: failed to draw data series");

//         chart
//             .draw_series(AreaSeries::new(
//                 iter2.clone(),
//                 0,
//                 plot_color(self.color2.with_alpha(0.5)),
//             ))
//             .expect("Error: failed to draw data series");

//         chart
//             .draw_series(LineSeries::new(
//                 iter1,
//                 ShapeStyle::from(plot_color(self.color1)).stroke_width(1),
//             ))
//             .expect("Error: failed to draw data series");
//         chart
//             .draw_series(LineSeries::new(
//                 iter2,
//                 ShapeStyle::from(plot_color(self.color2)).stroke_width(1),
//             ))
//             .expect("Error: failed to draw data series");
//     }
// }
