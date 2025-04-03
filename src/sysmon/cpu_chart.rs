// use crate::color::plot_color;
// use crate::helpers::base_background;
// use crate::sysmon::monitor_item::MonitorItem;

// use circular_queue::CircularQueue;
// use cosmic::cosmic_theme::palette::WithAlpha;
// use cosmic::iced_widget::container;
// use cosmic::theme::CosmicColor;
// use cosmic::{applet, Also, Apply, Element, Theme};
// use plotters::backend::DrawingBackend;
// use plotters::chart::ChartBuilder;
// use plotters::prelude::*;
// use plotters_iced::{Chart, ChartWidget};
// use sysinfo::Cpu;

// use crate::applet::Message;
// use crate::config::{self, CpuView};
// use crate::sysmon::bar_chart::{percentage_histogram, BarConfig, PercentageBar};

// use super::SourceCollection;

// /// `CpuChart` exists because CPU monitoring has a view style not covered by `SingleChart`: Viewing the usage of each core
// #[derive(Debug)]
// pub struct CpuData {
//     data_points: CircularQueue<i64>,
//     visualization: Vec<CpuView>,
//     color: CosmicColor,
//     // cpus: &'static [Cpu]
//     latest_per_core: Box<[f32]>,
// }

// impl CpuData {
//     pub fn update_colors(&mut self, color: CosmicColor) {
//         self.color = color;
//     }
// }

// impl Chart<Message> for CpuData {
//     type State = ();

//     fn build_chart<DB: DrawingBackend>(&self, _state: &Self::State, mut builder: ChartBuilder<DB>) {
//         let mut chart = builder
//             .build_cartesian_2d(0..self.data_points.len() as i64, 0..100_i64)
//             .expect("Error: failed to build chart");

//         // fill background moved to the ChartWidget that contains this chart

//         let iter = (0..self.data_points.len() as i64)
//             .zip(self.data_points.asc_iter())
//             .map(|x| (x.0, *x.1));

//         chart
//             .draw_series(AreaSeries::new(
//                 iter.clone(),
//                 0,
//                 plot_color(self.color.with_alpha(0.5)),
//             ))
//             .expect("Error: failed to draw data series");
//         chart
//             .draw_series(LineSeries::new(
//                 iter,
//                 ShapeStyle::from(plot_color(self.color)).stroke_width(1),
//             ))
//             .expect("Error: failed to draw data series");
//     }
// }

// impl MonitorItem for CpuData {
//     type ViewEnum = CpuView;
//     type ConfigStruct = (config::Cpu, usize);

//     fn new(c: (config::Cpu, usize), theme: &Theme) -> Self {
//         let (c, cpu_count) = c;
//         let mut data_points = CircularQueue::with_capacity(c.samples);
//         for _ in 0..c.samples {
//             data_points.push(0);
//         }
//         Self {
//             data_points,
//             visualization: c.visualization,
//             color: c.color.as_cosmic_color(theme),
//             latest_per_core: vec![0.0; cpu_count].into(),
//         }
//     }

//     fn view_single(&self, chart_view: &CpuView, context: &applet::Context) -> Element<Message> {
//         let (suggested_width, suggested_height) = context.suggested_size(false);

//         match chart_view {
//             CpuView::GlobalUsageRunChart => ChartWidget::new(self)
//                 .width(suggested_width.into())
//                 .height(suggested_height.into())
//                 .apply(container)
//                 .style(base_background)
//                 .into(),
//             CpuView::PerCoreUsageHistogram => {
//                 // let cpu_values: Box<[_]> = self.cpus.iter().map(Cpu::cpu_usage).collect();

//                 percentage_histogram(
//                     self.latest_per_core.clone(),
//                     BarConfig::default()
//                         .also(|bc| bc.full_length = context.suggested_size(false).1.into()),
//                     self.color,
//                 )
//                 .style(base_background)
//                 .into()
//             }
//             CpuView::GlobalUsageBarChart => {
//                 PercentageBar::new(
//                     context.is_horizontal(),
//                     /* sys.global_cpu_usage().clone(), */
//                     *self.data_points.iter().next().unwrap() as f32,
//                     suggested_width,
//                     suggested_height,
//                     self.color,
//                 )
//                 .apply(container)
//                 .style(base_background)
//                 .into()
//             }
//         }
//     }

//     fn view_order(&self) -> &[CpuView] {
//         self.visualization.as_slice()
//     }

//     fn resize_queue(&mut self, samples: usize) {
//         let mut data_points = CircularQueue::with_capacity(samples);
//         for data in self.data_points.asc_iter() {
//             data_points.push(*data);
//         }
//         self.data_points = data_points;
//     }

//     fn update_aspect_ratio(&mut self, _aspect_ratio: f32) {
//         // N/A
//     }
// }

// impl CpuData {
//     // type DataSource = System;
//     pub fn tick(&mut self, source: &mut SourceCollection) {
//         let sys = &mut source.sys;
//         sys.refresh_cpu_usage();
//         self.latest_per_core = sys.cpus().iter().map(Cpu::cpu_usage).collect();
//         self.data_points.push(sys.global_cpu_usage() as i64);
//     }
// }
