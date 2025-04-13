use plotters::{
    prelude::{ChartBuilder, DrawingBackend},
    series::{AreaSeries, LineSeries},
    style::{Color, RGBAColor, ShapeStyle},
};
use plotters_iced::Chart;

use crate::applet::{History, Message};

#[derive(Debug)]
pub struct HistoryChart<'a, T = u64> {
    pub history: &'a History<T>,
    pub max: T,
    pub color: RGBAColor,
}

impl<'a> HistoryChart<'a, u64> {
    pub fn auto_max(history: &'a History, color: RGBAColor) -> HistoryChart<'a> {
        HistoryChart {
            history,
            max: *history.iter().max().unwrap_or(&0),
            color,
        }
    }
}
impl Chart<Message> for HistoryChart<'_> {
    type State = ();

    fn build_chart<DB: DrawingBackend>(&self, _state: &Self::State, mut builder: ChartBuilder<DB>) {
        let mut chart = builder
            .build_cartesian_2d(0..self.history.len() as u64, 0..self.max)
            .expect("Error: failed to build chart");

        // fill background moved to the ChartWidget that contains this chart

        let iter = (0..self.history.len() as u64)
            .zip(self.history.asc_iter())
            .map(|(x, y)| (x, *y));

        chart
            .draw_series(AreaSeries::new(iter.clone(), 0, self.color.mix(0.5)))
            .expect("Error: failed to draw data series");
        chart
            .draw_series(LineSeries::new(
                iter,
                ShapeStyle::from(self.color).stroke_width(1),
            ))
            .expect("Error: failed to draw data series");
    }
}
impl Chart<Message> for HistoryChart<'_, f32> {
    type State = ();

    fn build_chart<DB: DrawingBackend>(&self, _state: &Self::State, mut builder: ChartBuilder<DB>) {
        let mut chart = builder
            .build_cartesian_2d(0..self.history.len() as u64, 0.0..self.max)
            .expect("Error: failed to build chart");

        // fill background moved to the ChartWidget that contains this chart

        let iter = (0..self.history.len() as u64)
            .zip(self.history.asc_iter())
            .map(|(x, y)| (x, *y));

        chart
            .draw_series(AreaSeries::new(iter.clone(), 0.0, self.color.mix(0.5)))
            .expect("Error: failed to draw data series");
        chart
            .draw_series(LineSeries::new(
                iter,
                ShapeStyle::from(self.color).stroke_width(1),
            ))
            .expect("Error: failed to draw data series");
    }
}

pub struct SuperimposedHistoryChart<'a> {
    pub h1: HistoryChart<'a>,
    pub h2: HistoryChart<'a>,
}

impl Chart<Message> for SuperimposedHistoryChart<'_> {
    type State = ();

    ///
    /// # !!
    /// Assumes length of both [History] are the same
    fn build_chart<DB: DrawingBackend>(&self, _state: &Self::State, mut builder: ChartBuilder<DB>) {
        let SuperimposedHistoryChart { h1, h2 } = self;

        // invariant of this implementation
        assert_eq!(h1.history.len(), h2.history.len());

        #[cfg(debug_assertions)]
        {
            // Expect the history to be full. Checks that CircularQueue was initialized as expected.
            assert!(h1.history.is_full());
            assert!(h1.history.is_full());
        }

        let samples = h1.history.len() as u64;

        let mut chart_1 = builder
            .build_cartesian_2d(0..samples, 0..h1.max)
            .expect("Error: failed to build chart");

        let mut chart_2 = builder
            .build_cartesian_2d(0..samples, 0..h2.max)
            .expect("Error: failed to build chart");

        let iter1 = (0..samples)
            .zip(h1.history.asc_iter())
            .map(|(x, y)| (x, *y));

        let iter2 = (0..samples)
            .zip(h2.history.asc_iter())
            .map(|(x, y)| (x, *y));

        chart_1 // h1 area
            .draw_series(AreaSeries::new(iter1.clone(), 0, h1.color.mix(0.5)))
            .expect("Error: failed to draw data series");

        chart_2 // h2 area
            .draw_series(AreaSeries::new(iter2.clone(), 0, h2.color.mix(0.5)))
            .expect("Error: failed to draw data series");

        chart_1 // h1 line
            .draw_series(LineSeries::new(
                iter1,
                ShapeStyle::from(h1.color).stroke_width(1),
            ))
            .expect("Error: failed to draw data series");
        chart_2 // h2 line
            .draw_series(LineSeries::new(
                iter2,
                ShapeStyle::from(h2.color).stroke_width(1),
            ))
            .expect("Error: failed to draw data series");
    }
}
