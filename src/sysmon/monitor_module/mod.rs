mod ram;
mod swap;

use crate::color::Color;
use crate::sysmon::bar_chart::PercentageBar;
use crate::{
    applet::Message,
    color::plotters_color,
    config::SingleView,
    helpers::{base_background, get_sized_aspect_ratio},
    sysmon::SourceCollection,
};
use cosmic::{
    applet, cosmic_theme::palette::WithAlpha as _, iced::Size, theme::CosmicColor,
    widget::container, Apply as _, Element,
};
pub use ram::RamModule;
use std::{borrow::Borrow, marker::PhantomData};
pub use swap::SwapModule;

use circular_queue::CircularQueue;
use plotters::series::{AreaSeries, LineSeries};
use plotters::style::ShapeStyle;

pub trait View {
    type Data;
    fn make_element<'a>(
        &self,
        data: &'a impl Borrow<Self::Data>,
        context: &applet::Context,
        color: crate::color::Color,
    ) -> Element<'a, Message>;
}

pub type HistoryStruct = CircularQueue<i64>;

pub trait Refresh {
    fn tick(&mut self, source: &mut SourceCollection);
}

pub trait Configurable {
    type Config;
    fn configure(&mut self, config: Self::Config);
}

/// Specific instances of this struct make up the available modules of the panel module of the SystemMonitor
#[derive(Debug)]
pub struct Module<D: Configurable + Borrow<V::Data>, C: Into<Self>, V: View> {
    data: D,
    vis: Box<[V]>,
    color: Color,
    config: PhantomData<C>,
}

impl<D: Configurable + Borrow<V::Data>, C: Into<Self>, V: View> Module<D, C, V> {
    /*type ViewOption = E;*/

    pub fn view_order(&self) -> &[V] {
        &self.vis
    }

    fn view_single(&self, chart_view: &V, context: &applet::Context) -> Element<Message> {
        chart_view.make_element(&self.data, context, self.color.clone())
    }

    pub fn view<'a>(
        &'a self,
        context: &'a applet::Context,
        spacing: impl Into<cosmic::iced::Pixels>,
    ) -> Element<'a, Message> {
        crate::helpers::collection(
            context,
            self.view_order()
                .iter()
                .map(|v| self.view_single(v, context)),
            spacing,
            0.0,
        )
    }

    pub fn configure(
        &mut self,
        d_conf: impl Into<D::Config>,
        vis: Box<[V]>,
        color: Color,
    ) {
        self.data.configure(d_conf.into());
        self.vis = vis;
        self.color = color;
    }
}

#[derive(Debug)]
pub(crate) struct SingleData {
    history: HistoryStruct,
}

impl Configurable for SingleData {
    type Config = usize;
    fn configure(&mut self, new_size: usize) {
        if new_size == self.history.capacity() {
            return; // no changes to make
        }

        let mut temp = CircularQueue::<i64>::with_capacity(new_size);
        std::mem::swap(&mut self.history, &mut temp);
        // temp now contains the old history
        let data = temp.into_vec();

        for (_, value) in (0..new_size).zip(data).rev() {
            // todo test this this correct

            self.history.push(value);
        }
        while !self.history.is_full() {
            self.history.push(0);
        }
    }
}

impl Borrow<HistoryStruct> for SingleData {
    fn borrow(&self) -> &HistoryStruct {
        &self.history
    }
}

impl From<HistoryStruct> for SingleData {
    fn from(history: HistoryStruct) -> Self {
        Self { history }
    }
}

impl<C, E> Default for Module<SingleData, C, E>
where
    E: View<Data=HistoryStruct>,
    C: Into<Self> + Default,
{
    fn default() -> Self {
        C::default().into()
    }
}

pub fn init_data_points(size: impl Into<usize>) -> CircularQueue<i64> {
    let size = size.into();
    let mut data_points = CircularQueue::with_capacity(size);
    for _ in 0..size {
        data_points.push(0);
    }
    data_points
}

impl plotters_iced::Chart<Message> for (&HistoryStruct, CosmicColor) {
    type State = ();

    fn build_chart<DB: plotters::prelude::DrawingBackend>(
        &self,
        _state: &Self::State,
        mut builder: plotters::prelude::ChartBuilder<DB>,
    ) {
        let (history, color) = self;

        {
            let mut chart = builder
                .build_cartesian_2d(0..history.capacity() as i64, 0..100_i64)
                .expect("Error: failed to build chart");

            // fill background moved to the ChartWidget that contains this chart

            let iter = (0..history.capacity() as i64)
                .zip(history.asc_iter())
                .map(|x| (x.0, *x.1));

            chart
                .draw_series(AreaSeries::new(
                    iter.clone(),
                    0,
                    plotters_color(color.with_alpha(0.5)),
                ))
                .expect("Error: failed to draw data series");
            chart
                .draw_series(LineSeries::new(
                    iter,
                    ShapeStyle {
                        color: plotters_color(*color),
                        stroke_width: 1,
                        filled: true,
                    },
                ))
                .expect("Error: failed to draw data series");
        }
    }
}

impl View for SingleView {
    type Data = HistoryStruct;

    fn make_element<'a>(
        &self,
        data: &'a impl Borrow<HistoryStruct>,
        context: &applet::Context,
        color: crate::color::Color,
    ) -> Element<'a, Message> {
        let history = data.borrow();
        let theme = context.theme().unwrap_or_default();
        let color = color.as_cosmic_color(&theme);
        match self {
            SingleView::Bar { aspect_ratio } => {
                let Size { width, height } = get_sized_aspect_ratio(context, *aspect_ratio);
                PercentageBar::new(
                    context.is_horizontal(),
                    *history.iter().next().unwrap_or(&0) as f32,
                    color,
                )
                    .apply(container)
                    .width(width)
                    .height(height)
            }
            SingleView::Run { aspect_ratio } => {
                let Size { width, height } = get_sized_aspect_ratio(context, *aspect_ratio);

                plotters_iced::ChartWidget::new((history, color))
                    .apply(container)
                    .width(width)
                    .height(height)
            }
        }
            .style(base_background)
            .apply(Element::new)
    }
}
