use circular_queue::CircularQueue;
use cosmic::iced_widget::container;
use cosmic::{applet, Also, Apply, Element, Theme};
use plotters::backend::DrawingBackend;
use plotters::chart::ChartBuilder;
use plotters::prelude::*;
use plotters_iced::{Chart, ChartWidget};
use sysinfo::Cpu;

use crate::applet::Message;
use crate::config::CpuView;
use crate::sysmon::bar_chart::{percentage_histogram, BarConfig, PercentageBar};
use crate::sysmon::chart::base_background;
use crate::sysmon::viewable::MonitorItem;

/// CpuChart exists because CPU monitoring has a view style not covered by SingleChart: Viewing the usage of each core
#[derive(Debug)]
pub struct CpuChart {
    data_points: CircularQueue<i64>,
    visualization: CpuView,
    theme_color: crate::color::Color,
    rgb_color: RGBColor,
    pub size: f32,
    // cpus: &'static [Cpu]
    latest_per_core: Box<[f32]>,
}

impl CpuChart {
    pub fn update_latest_per_core(&mut self, cpus: &[Cpu]) {
        self.latest_per_core = cpus.iter().map(Cpu::cpu_usage).collect();
    }
}

impl CpuChart {
    pub fn resize_queue(&mut self, samples: usize) {
        let mut data_points = CircularQueue::with_capacity(samples);
        for data in self.data_points.asc_iter() {
            data_points.push(*data);
        }
        self.data_points = data_points;
    }

    pub fn update_colors(&mut self, color: crate::color::Color, theme: &Theme) {
        self.theme_color = color;
    }

    pub fn update_size(&mut self, size: f32) {
        self.size = size;
    }

    pub fn push_data(&mut self, value: i64) {
        self.data_points.push(value);
    }
}

impl CpuChart {
    pub fn new(
        visualization: CpuView,
        theme_color: crate::color::Color,
        history_size: usize,
        size: f32,
        theme: &Theme,
        cpus: &[Cpu],
    ) -> Self {
        let mut data_points = CircularQueue::with_capacity(history_size);
        for _ in 0..history_size {
            data_points.push(0);
        }
        let latest_per_core = cpus.iter().map(Cpu::cpu_usage).collect();
        Self {
            data_points,
            visualization,
            theme_color: theme_color.clone(),
            rgb_color: theme_color.as_rgb_color(theme),
            size,
            latest_per_core,
        }
    }
}

impl Chart<Message> for CpuChart {
    type State = ();

    fn build_chart<DB: DrawingBackend>(&self, state: &Self::State, mut builder: ChartBuilder<DB>) {
        let mut chart = builder
            .build_cartesian_2d(0..self.data_points.len() as i64, 0..100_i64)
            .expect("Error: failed to build chart");

        // fill background moved to the ChartWidget that contains this chart

        let iter = (0..self.data_points.len() as i64)
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

impl MonitorItem<CpuView> for CpuChart {
    fn view_as_configured(&self, context: &applet::Context) -> Element<Message> {
        self.view_as(self.visualization, context)
    }

    fn view_as(&self, chart_view: CpuView, context: &applet::Context) -> Element<Message> {
        let (suggested_width, suggested_height) = context.suggested_size(false);

        let theme = &context.theme().unwrap_or_default();

        match chart_view {
            CpuView::GlobalUsageRunChart => {
                ChartWidget::new(self)
                    .width(suggested_width.into())
                    .height(suggested_height.into())
                    .apply(container)
                    .style(base_background)
                    .into()
            }
            CpuView::PerCoreUsageHistogram => {
                // let cpu_values: Box<[_]> = self.cpus.iter().map(Cpu::cpu_usage).collect();

                percentage_histogram(
                    self.latest_per_core.clone(),
                    BarConfig::default().also(|bc| bc.full_length = context.suggested_size(false).1.into()),
                    self.theme_color.as_srgba(theme),
                )
                    .style(base_background)
                    .into()
            }
            CpuView::GlobalUsageBarChart => {
                PercentageBar::new(
                    context.is_horizontal(),
                    /* sys.global_cpu_usage().clone(), */
                    *self.data_points.iter().next().unwrap() as f32,
                    suggested_width,
                    suggested_height,
                    self.theme_color.as_srgba(theme),
                )
                    .apply(container)
                    .style(base_background)
                    .into()
            }
        }
    }
}
