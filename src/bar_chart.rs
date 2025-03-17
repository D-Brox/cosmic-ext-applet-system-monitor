use cosmic::{
    applet::cosmic_panel_config::PanelAnchor,
    iced::{
        self,
        alignment::{Alignment, Horizontal, Vertical},
        core::{layout, mouse, renderer, widget::Tree, Layout, Length, Rectangle, Size},
        Background,
    },
    widget::{container, Container, Column, Row, Widget},
    Element, Renderer, Theme,
};
use cosmic::cosmic_theme::palette::Srgba;
use renderer::Style;
use sysinfo::Cpu;

use crate::applet::Message;

#[derive(Clone, Copy)]
pub enum SortMethod {
    Descending,
    Ascending,
}

#[derive(Clone)]
pub enum Orientation {
    PointingUp,
    PointingRight, // todo more
}
impl Orientation {
    pub(crate) fn default_for(anchor: PanelAnchor) -> Orientation {
        match anchor {
            PanelAnchor::Left => Orientation::PointingRight,
            PanelAnchor::Right => Orientation::PointingRight,
            PanelAnchor::Top | PanelAnchor::Bottom => Orientation::PointingUp,
        }
    }
}

#[derive(Clone)]
pub struct BarConfig {
    pub orientation: Orientation, // todo replace with ~core.applet.Anchor~
    /// The length in the direction the bar varies it's length
    pub full_length: Length,
    pub width_fraction: f32,
    pub spacing: Length,
    pub sort_method: Option<SortMethod>,
}

impl Default for BarConfig {
    fn default() -> Self {
        Self {
            orientation: Orientation::PointingUp,
            full_length: Length::Fill,
            width_fraction: 0.25,
            spacing: Length::Fixed(1.0),
            sort_method: Some(SortMethod::Descending),
        }
    }
}

pub fn per_core_cpu_container(cpus: &[Cpu], config: BarConfig, color: Srgba) -> Container<Message, Theme> {
    let (Length::Fixed(spacing), Length::Fixed(full_length)) = (config.spacing, config.full_length)
    else {
        unimplemented!()
    };

    let static_dimension = full_length * config.width_fraction;

    let mut cpu_values: Box<[_]> = cpus.iter().map(Cpu::cpu_usage).collect();

    if let Some(sort_method) = config.sort_method {
        match sort_method {
            SortMethod::Descending => cpu_values.sort_by(|a, b| b.partial_cmp(a).unwrap()),
            SortMethod::Ascending => cpu_values.sort_by(|a, b| a.partial_cmp(b).unwrap()),
        }
    }
    // dbg!(full_length, static_dimension);

    let inner: Element<Message> =
        match config.orientation {
            Orientation::PointingUp => {
                Row::with_children(cpu_values.iter().enumerate().map(|(i, &val)| {
                    VerticalPercentageBar::new(
                        val,
                        full_length,
                        static_dimension,
                        color,
                    )
                    .into()
                }))
                .height(full_length)
                .align_y(Vertical::Bottom)
                .spacing(spacing)
                .into()
            }
            Orientation::PointingRight => Column::with_children(cpu_values.iter().map(|&val| {
                HorizontalPercentageBar::new(val, full_length, static_dimension, color).into()
            }))
            .width(full_length)
            .align_x(Horizontal::Left)
            .spacing(spacing)
            .into(),
        };

    let outer = cosmic::widget::container(inner).style(|_| container::Style {
/*        background: Some(Background::Color(iced::Color {
            r: 20.0 / 255.0,
            g: 20.0 / 255.0,
            b: 20.0 / 255.0,
            a: 1.0,
        })),*/
        ..container::Style::default()
    });
    outer
}

pub enum PercentageBar {
    Vertical(VerticalPercentageBar),
    Horizontal(HorizontalPercentageBar),
}
impl PercentageBar {
    pub(crate) fn new(
        is_horizontal: bool,
        value: f32,
        width: f32,
        height: f32,
        theme_color: Srgba,
    ) -> Self {
        if is_horizontal {
            Self::Vertical(VerticalPercentageBar::new(
                value,
                height,
                width,
                theme_color,
            ))
        } else {
            Self::Horizontal(HorizontalPercentageBar::new(value, width, height, theme_color))
        }
    }
}

impl<'a> Widget<Message, Theme, Renderer> for PercentageBar {
    fn size(&self) -> Size<Length> {
        match self {
            PercentageBar::Vertical(v) => Widget::<Message, Theme, Renderer>::size(v),
            PercentageBar::Horizontal(h) => Widget::<Message, Theme, Renderer>::size(h),
        }
    }

    fn layout(
        &self,
        tree: &mut Tree,
        renderer: &Renderer,
        limits: &layout::Limits,
    ) -> layout::Node {
        match self {
            PercentageBar::Vertical(v) => {
                Widget::<Message, Theme, Renderer>::layout(v, tree, renderer, limits)
            }
            PercentageBar::Horizontal(h) => {
                Widget::<Message, Theme, Renderer>::layout(h, tree, renderer, limits)
            }
        }
    }

    fn draw(
        &self,
        tree: &Tree,
        renderer: &mut Renderer,
        theme: &Theme,
        style: &Style,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        viewport: &Rectangle,
    ) {
        match self {
            PercentageBar::Vertical(v) => Widget::<Message, Theme, Renderer>::draw(
                v, tree, renderer, theme, style, layout, cursor, viewport,
            ),
            PercentageBar::Horizontal(h) => Widget::<Message, Theme, Renderer>::draw(
                h, tree, renderer, theme, style, layout, cursor, viewport,
            ),
        }
    }
}

#[allow(missing_debug_implementations)]
pub struct VerticalPercentageBar {
    percentage: f32,
    width: Length,
    varying_length_max: Length,
    theme_color: Srgba,
}

impl<'a> VerticalPercentageBar {
    pub fn new(
        value: f32,
        height: impl Into<Length>,
        width: impl Into<Length>,
        theme_color: Srgba,
    ) -> Self {
        VerticalPercentageBar {
            percentage: value.clamp(0.0, 100.0),
            width: width.into(),
            varying_length_max: height.into(),
            theme_color,
        }
    }
}

impl<'a> Widget<Message, Theme, Renderer> for VerticalPercentageBar {
    fn size(&self) -> Size<Length> {
        Size {
            width: self.width,
            height: self.varying_length_max,
        }
    }

    fn layout(
        &self,
        _tree: &mut Tree,
        _renderer: &Renderer,
        limits: &layout::Limits,
    ) -> layout::Node {
        let Size { width, height } = Widget::size(self);

        let layout = layout::atomic(limits, width, height);
        layout
    }

    fn draw(
        &self,
        _state: &Tree,
        renderer: &mut Renderer,
        theme: &Theme,
        _style: &Style,
        layout: Layout<'_>,
        _cursor: mouse::Cursor,
        _viewport: &Rectangle,
    ) {
        if self.percentage <= 0.0 {
            return;
        }
        let outer = &layout.bounds();
        let fill_height = self.percentage / 100.0 * outer.height;
        let rect = Rectangle {
            y: outer.y + outer.height - fill_height,
            height: fill_height,
            ..*outer
        };
        iced::core::Renderer::fill_quad(
            renderer,
            renderer::Quad {
                bounds: rect,
                ..renderer::Quad::default()
            },
            iced::Color::from(self.theme_color),
        );
    }
}

impl<'a> From<VerticalPercentageBar> for Element<'a, Message> {
    fn from(percentage_tower: VerticalPercentageBar) -> Element<'a, Message> {
        Element::new(percentage_tower)
    }
}

pub struct HorizontalPercentageBar {
    value: f32,
    bar_thickness: Length,
    varying_length_max: Length,
    color: Srgba
}
impl HorizontalPercentageBar {
    pub fn new(
        value: f32,
        varying_length_max: impl Into<Length>,
        static_length: impl Into<Length>,
        color: impl Into<Srgba>
    ) -> Self {
        let color = color.into();
        Self {
            value: value.clamp(0.0, 100.0),
            bar_thickness: static_length.into(),
            varying_length_max: varying_length_max.into(),
            color,
        }
    }
}

impl Widget<Message, Theme, Renderer> for HorizontalPercentageBar {
    fn size(&self) -> Size<Length> {
        let Length::Fixed(max_length) = self.varying_length_max else {
            unimplemented!()
        };

        Size::<Length> {
            width: (max_length * self.value / 100.).into(),
            height: self.bar_thickness,
        }
    }

    fn layout(
        &self,
        _tree: &mut Tree,
        _renderer: &Renderer,
        limits: &layout::Limits,
    ) -> layout::Node {
        let Size { width, height } = Widget::<Message, Theme, Renderer>::size(self);
        layout::atomic(limits, width, height).align(
            Alignment::Start,
            Alignment::Center,
            limits.max(),
        )
    }

    fn draw(
        &self,
        _tree: &Tree,
        renderer: &mut Renderer,
        theme: &Theme,
        _style: &Style,
        layout: Layout<'_>,
        _cursor: mouse::Cursor,
        _viewport: &Rectangle,
    ) {
        iced::core::Renderer::fill_quad(
            renderer,
            renderer::Quad {
                bounds: layout.bounds(),
                ..renderer::Quad::default()
            },
            iced::Color::from(self.color),
        );
    }
}

impl<'a> From<HorizontalPercentageBar> for Element<'a, Message> {
    fn from(value: HorizontalPercentageBar) -> Self {
        Element::new(value)
    }
}
