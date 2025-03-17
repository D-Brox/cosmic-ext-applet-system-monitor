use std::ops::{Deref, DerefMut};

use circular_queue::CircularQueue;
use cosmic::iced_core::widget::tree::{State, Tag};
use cosmic::widget::layer_container;
use cosmic::{
    applet,
    iced::{Length, Rectangle, Size},
    iced_core::{
        layout::{Limits, Node},
        mouse::Cursor,
        renderer::Style,
        widget::Tree,
        Layout,
    },
    widget::{Container, Widget},
    Apply, Element, Renderer, Theme,
};
use plotters_iced::ChartWidget;
use sysinfo::System;

use crate::{
    applet::Message,
    bar_chart::{per_core_cpu_container, BarConfig, PercentageBar},
    chart::SingleChart,
};

pub struct CpuWidget<'a> {
    sys: &'a System,
    data_points: CircularQueue<i64>,
    visualization: CpuVisualization<'a>,
}

impl<'a> CpuWidget<'a> {
    pub(crate) fn global_bar(
        config: crate::config::Generic,
        sys: &'a System,
        context: &'a applet::Context,
        height: f32,
        width: f32,
    ) -> Self {
        let visualization = CpuVisualization::GlobalBar(PercentageBar::new(
            context.is_horizontal(),
            sys.global_cpu_usage(),
            width,
            height,
            config.color.as_srgba(&context.theme().unwrap_or_default()),
        ));

        Self {
            sys,
            data_points: CircularQueue::with_capacity(config.samples),
            visualization,
        }
    }
    pub(crate) fn multi_bar(
        config: crate::config::Generic,
        sys: &'a System,
        bar_config: BarConfig,
        context: &'a applet::Context,
    ) -> Self {
        let visualization = CpuVisualization::MultiCpuBar(per_core_cpu_container(
            &sys.cpus(),
            bar_config,
            config.color.as_srgba(&context.theme().unwrap_or_default()),
        ));

        Self {
            sys,
            data_points: CircularQueue::with_capacity(config.samples),
            visualization,
        }
    }

    pub(crate) fn run_chart(
        config: crate::config::Generic,
        sys: &'a System,
        context: &'a applet::Context,
        data_points: CircularQueue<i64>
    ) -> Self {
/*        let mut data_points = CircularQueue::<i64>::with_capacity(config.samples);

        for i in 0..config.samples as i64 {

            data_points.push(i*3);
        }*/


        let (suggested_width, suggested_height) = context.suggested_size(false);

        let visualization = CpuVisualization::RunChart(
            ChartWidget::new(SingleChart::with_data(
                data_points.clone(),
                config.color,
                config.size,
                &context.theme().unwrap_or_default(),
            ))
            .height(suggested_height.into())
            .apply(layer_container)
            .width(suggested_width.into())
            .padding(0),
        );

        Self {
            sys,
            data_points,
            visualization,
        }
    }
}

impl Widget<Message, Theme, Renderer> for CpuWidget<'_> {
    fn size(&self) -> Size<Length> {
        self.visualization.size()
    }

    fn layout(&self, tree: &mut Tree, renderer: &Renderer, limits: &Limits) -> Node {
        self.visualization.layout(tree, renderer, limits)
    }

    fn draw(
        &self,
        tree: &Tree,
        renderer: &mut Renderer,
        theme: &Theme,
        style: &Style,
        layout: Layout<'_>,
        cursor: Cursor,
        viewport: &Rectangle,
    ) {
        self.visualization
            .draw(tree, renderer, theme, style, layout, cursor, viewport)
    }

    fn state(&self) -> State {
        self.visualization.state()
    }

    fn children(&self) -> Vec<Tree> {
        self.visualization.children()
    }
}

/*impl<'a> Widget<Message, Theme, Renderer> for CpuWidget<'a> {
    fn size(&self) -> Size<Length> {
        Size {
            width: Length::Shrink,
            height: Length::Shrink,
        }
    }

    fn layout(&self, tree: &mut Tree, renderer: &Renderer, limits: &Limits) -> Node {
        let Size { width, height } = Widget::size(self);

        let layout = layout::contained(limits, width, height, |l| {
            self.visualization.layout(tree, renderer, limits)
        });
        layout
    }

    fn draw(
        &self,
        tree: &Tree,
        renderer: &mut Renderer,
        theme: &Theme,
        style: &Style,
        layout: Layout<'_>,
        cursor: Cursor,
        viewport: &Rectangle,
    ) {
        self.visualization
            .draw(tree, renderer, theme, style, layout, cursor, viewport);
    }

    fn tag(&self) -> tree::Tag {
        self.visualization.tag()
    }

    fn state(&self) -> tree::State {
        self.visualization.state()
    }

    fn children(&self) -> Vec<Tree> {
        self.visualization.children()
    }

    fn diff(&mut self, tree: &mut Tree) {
        self.visualization.diff(tree);
    }
    fn operate(&self, state: &mut Tree, layout: Layout<'_>, renderer: &Renderer, operation: &mut dyn Operation) {
        operation.container(
            self.visualization.id().as_ref(),
            layout.bounds(),
            &mut |operation| {
                self.visualization.operate(
                    state,
                    layout.children().next().unwrap(),
                    renderer,
                    operation,
                );
            },
        );
    }
}
*/
pub enum CpuVisualization<'a> {
    // RunChart(ChartWidget<'a, Message, Theme, Renderer, SingleChart>),
    RunChart(layer_container::LayerContainer<'a, Message, Renderer>),

    GlobalBar(PercentageBar),
    MultiCpuBar(Container<'a, Message, Theme, Renderer>),
}

impl<'a> Deref for CpuVisualization<'a> {
    type Target = dyn Widget<Message, Theme, Renderer> + 'a;

    fn deref(&self) -> &Self::Target {
        match self {
            CpuVisualization::RunChart(chart_widget) => chart_widget,
            CpuVisualization::GlobalBar(percentage_bar) => percentage_bar,
            CpuVisualization::MultiCpuBar(container) => container,
        }
    }
}

impl<'a> DerefMut for CpuVisualization<'a> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        match self {
            CpuVisualization::RunChart(chart_widget) => chart_widget,
            CpuVisualization::GlobalBar(percentage_bar) => percentage_bar,
            CpuVisualization::MultiCpuBar(container) => container,
        }
    }
}

impl<'a> From<CpuVisualization<'a>> for Element<'a, Message> {
    fn from(value: CpuVisualization<'a>) -> Self {
        match value {
            CpuVisualization::RunChart(chart_widget) => chart_widget.into(),
            CpuVisualization::GlobalBar(p_bar) => match p_bar {
                PercentageBar::Vertical(v_p_bar) => v_p_bar.into(),
                PercentageBar::Horizontal(h_p_bar) => h_p_bar.into(),
            },
            CpuVisualization::MultiCpuBar(container) => container.into(),
        }
    }
}

impl<'a> From<CpuWidget<'a>> for Element<'a, Message> {
    fn from(value: CpuWidget<'a>) -> Element<'a, Message> {
        Element::new(value)
    }
}
