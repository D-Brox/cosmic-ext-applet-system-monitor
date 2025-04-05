mod ram;
mod swap;

use crate::color::Color;
use crate::config::DoubleView;
use crate::sysmon::bar_chart::PercentageBar;
use crate::{
    applet::Message, color::plotters_color, config::SingleView, helpers::get_sized_aspect_ratio,
    sysmon::SourceCollection,
};
use cosmic::iced_core::Pixels;
use cosmic::{
    applet, cosmic_theme::palette::WithAlpha as _, iced::Size, style, theme::CosmicColor,
    widget::container, Apply as _, Element,
};
use plotters_iced::Chart;
use std::{borrow::Borrow, marker::PhantomData};

use circular_queue::CircularQueue;
use plotters::series::{AreaSeries, LineSeries};
use plotters::style::ShapeStyle;

pub use ram::RamModule;
pub use swap::SwapModule;

pub type History = CircularQueue<i64>;
pub type SingleModule<C> = Module<History, Box<[SingleView]>, C>;

pub trait ViewableModule {
    fn view<'a>(
        &'a self,
        context: &'a applet::Context,
        spacing: impl Into<Pixels>,
    ) -> Element<'a, Message>;
}

pub trait Refresh {
    fn tick(&mut self, source: &mut SourceCollection);
}

pub trait Configurable {
    type Config;
    fn configure(&mut self, config: impl Into<Self::Config>);
}

/// Specific instances of this struct make up the available modules of the panel module of the SystemMonitor
#[derive(Debug)]
pub struct Module<D, V, C> {
    data: D,
    vis: V,
    config: PhantomData<C>,
}

// A module can be created from it's default config
impl<D, V, C> Default for Module<D, V, C>
where
    C: Into<Self> + Default,
{
    fn default() -> Self {
        C::default().into()
    }
}

impl Configurable for History {
    type Config = usize;
    fn configure(&mut self, new_size: impl Into<usize>) {
        let new_size = new_size.into();
        if new_size == self.capacity() {
            return; // no changes to make
        }

        let new = History::with_capacity(new_size);
        let old = std::mem::replace(self, new);
        let data = old.into_vec();

        for (_, value) in (0..new_size).zip(data).rev() {
            // todo test this this correct

            self.push(value);
        }
        while !self.is_full() {
            self.push(0);
        }
    }
}

pub fn init_data_points(size: impl Into<usize>) -> History {
    let size = size.into();
    let mut data_points = History::with_capacity(size);
    for _ in 0..size {
        data_points.push(0);
    }
    data_points
}

impl<D: Borrow<History>> Chart<Message> for (&D, CosmicColor) {
    type State = ();

    fn build_chart<DB: plotters::prelude::DrawingBackend>(
        &self,
        _state: &Self::State,
        mut builder: plotters::prelude::ChartBuilder<DB>,
    ) {
        let (history, color) = (self.0.borrow(), self.1);

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
                    color: plotters_color(color),
                    stroke_width: 1,
                    filled: true,
                },
            ))
            .expect("Error: failed to draw data series");
    }
}

/*
impl<'a> View<'a> for SingleView {
    fn view(
        &self,
        data: &'a impl Borrow<HistoryStruct>,
        context: &applet::Context,
        fallback_color: crate::color::Color,
    ) -> Element<'a, Message> {
        let history = data.borrow();
        let theme = &context.theme().unwrap_or_default();
        match self {
            SingleView::Bar {
                aspect_ratio,
                color,
            } => {
                let Size { width, height } = get_sized_aspect_ratio(context, *aspect_ratio);
                PercentageBar::new(
                    context.is_horizontal(),
                    *history.iter().next().unwrap_or(&0) as f32,
                    color.unwrap_or(fallback_color).as_cosmic_color(theme),
                )
                .apply(container)
                .width(width)
                .height(height)
            }
            SingleView::Run {
                aspect_ratio,
                color,
            } => {
                let Size { width, height } = get_sized_aspect_ratio(context, *aspect_ratio);

                plotters_iced::ChartWidget::new((
                    history,
                    color.unwrap_or(fallback_color).as_cosmic_color(theme),
                ))
                .apply(container)
                .width(width)
                .height(height)
            }
        }
        .style(|t| style::Container::primary(t.cosmic())) // todo make styling good. try apply rounding in Renderer::fill_quad
        // .style(base_background)
        .apply(Element::new)
    }
}
*/

/*
impl<'a> View<'a, &usize> for DoubleView {
    type Data = DoubleData;
    type Styling = [Color; 2];

    fn view<'a>(
        &self,
        data: &'a impl Borrow<Self::Data>,
        context: &applet::Context,
        styling: Self::Styling,
    ) -> Element<'a, Message> {
        let histories = data.borrow();

        match self {
            DoubleView::SuperimposedRunChart { aspect_ratio } => {
                let Size { width, height } = get_sized_aspect_ratio(context, *aspect_ratio);

                plotters_iced::ChartWidget::new((histories, color1, color2))
                    .apply(container)
                    .width(width)
                    .height(height)
            }
        }
        // todo!()
    }
}
*/

#[derive(Debug)]
pub struct DoubleData {
    history1: History,
    history2: History,
}
impl Configurable for DoubleData {
    type Config = (usize, usize);

    fn configure(&mut self, config: impl Into<Self::Config>) {
        let (c1, c2) = config.into();
        self.history1.configure(c1);
        self.history2.configure(c2);
    }
}

impl<C> ViewableModule for SingleModule<C> {
    fn view<'a>(
        &'a self,
        context: &'a applet::Context,
        spacing: impl Into<Pixels>,
    ) -> Element<'a, Message> {
        crate::helpers::collection(
            context,
            self.vis.iter().map(|v| self.view_single(v, context)),
            spacing,
            0.0,
        )
    }
}
impl<C> SingleModule<C> {
    fn view_single(&self, v: &SingleView, context: &applet::Context) -> Element<Message> {
        let theme = &context.theme().unwrap_or_default();
        match *v {
            SingleView::Bar {
                aspect_ratio,
                color,
            } => {
                let Size { width, height } = get_sized_aspect_ratio(context, aspect_ratio);
                PercentageBar::new(
                    context.is_horizontal(),
                    *self.data.iter().next().unwrap_or(&0) as f32,
                    color,
                )
                .apply(container)
                .width(width)
                .height(height)
            }
            SingleView::Run {
                aspect_ratio,
                color,
            } => {
                let Size { width, height } = get_sized_aspect_ratio(context, aspect_ratio);
                let chart_object = (&self.data, color.as_cosmic_color(theme));

                plotters_iced::ChartWidget::new(chart_object)
                    .apply(container)
                    .width(width)
                    .height(height)
            }
        }
        .style(|t| style::Container::primary(t.cosmic())) // todo make styling good. try apply rounding in Renderer::fill_quad
        // .style(base_background)
        .apply(Element::new)
    }
}
