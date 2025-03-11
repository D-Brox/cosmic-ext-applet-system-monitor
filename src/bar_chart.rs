use cosmic::{
    iced::{
        self,
        alignment::{Alignment, Horizontal, Vertical},
        core::{
            self, layout, mouse, renderer, widget::Tree, Color, Element, Layout, Length, Rectangle,
            Size,
        },
        Background,
    },
    widget::{container, progress_bar, Column, Row, Widget},
    Renderer, Theme,
};
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

pub fn per_core_cpu_container<'a>(
    cpus: &'a [Cpu],
    config: BarConfig,
) -> container::Container<'a, Message, Theme> {
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

    let inner: Element<'_, Message, Theme, Renderer> =
        match config.orientation {
            Orientation::PointingUp => Row::with_children(cpu_values.iter().map(|&val| {
                Element::from(VerticalPercentageBar::new(
                    val,
                    full_length,
                    static_dimension,
                ))
            }))
            .align_y(Vertical::Bottom)
            .spacing(spacing)
            .into(),
            Orientation::PointingRight => Column::with_children(cpu_values.iter().map(|&val| {
                HorizontalPercentageBar::new(val, full_length, static_dimension).into()
            }))
            .align_x(Horizontal::Left)
            .spacing(spacing)
            .into(),
        };

    let outer = cosmic::widget::container(inner).style(|_| container::Style {
        background: Some(Background::Color(Color {
            r: 20.0 / 255.0,
            g: 20.0 / 255.0,
            b: 20.0 / 255.0,
            a: 1.0,
        })),
        ..container::Style::default()
    });

    let outer = match config.orientation {
        Orientation::PointingUp => outer.align_bottom(full_length),
        Orientation::PointingRight => outer.align_left(full_length),
    };
    outer
}

#[allow(missing_debug_implementations)]
pub struct VerticalPercentageBar<'a, T>
where
    T: progress_bar::Catalog,
{
    value: f32,
    width: Length,
    varying_length_max: Length,
    class: T::Class<'a>,
}

impl<'a, T> VerticalPercentageBar<'a, T>
where
    T: progress_bar::Catalog,
{
    pub fn new(value: f32, height: impl Into<Length>, width: impl Into<Length>) -> Self {
        VerticalPercentageBar {
            value: value.clamp(0.0, 100.0),
            width: width.into(),
            varying_length_max: height.into(),
            class: T::default(),
        }
    }
}

impl<'a, Message, Theme, Renderer> Widget<Message, Theme, Renderer>
    for VerticalPercentageBar<'a, Theme>
where
    Theme: progress_bar::Catalog,
    Renderer: core::Renderer,
{
    fn size(&self) -> Size<Length> {
        let Length::Fixed(max_length) = self.varying_length_max else {
            todo!("solve the flex length issue")
        };

        Size {
            width: self.width,
            height: (max_length * self.value / 100.0).into(),
        }
    }

    fn layout(
        &self,
        _tree: &mut Tree,
        _renderer: &Renderer,
        limits: &layout::Limits,
    ) -> layout::Node {
        let Size { width, height } =
            <VerticalPercentageBar<'_, _> as Widget<Message, _, Renderer>>::size(self);

        layout::atomic(limits, width, height)
    }

    fn draw(
        &self,
        _state: &Tree,
        renderer: &mut Renderer,
        theme: &Theme,
        _style: &renderer::Style,
        layout: Layout<'_>,
        _cursor: mouse::Cursor,
        _viewport: &Rectangle,
    ) {
        renderer.fill_quad(
            renderer::Quad {
                bounds: layout.bounds(),
                ..renderer::Quad::default()
            },
            theme.style(&self.class).bar,
        );
    }
}

impl<'a, Message, Theme, Renderer> From<VerticalPercentageBar<'a, Theme>>
    for Element<'a, Message, Theme, Renderer>
where
    Message: 'a,
    Theme: 'a + progress_bar::Catalog,
    Renderer: 'a + core::Renderer,
{
    fn from(
        percentage_tower: VerticalPercentageBar<'a, Theme>,
    ) -> Element<'a, Message, Theme, Renderer> {
        Element::new(percentage_tower)
    }
}

pub(crate) struct HorizontalPercentageBar<'a, T>
where
    T: progress_bar::Catalog,
{
    value: f32,
    bar_thickness: Length,
    varying_length_max: Length,
    class: T::Class<'a>,
}
impl<'a, T> HorizontalPercentageBar<'a, T>
where
    T: progress_bar::Catalog,
{
    pub fn new(
        value: f32,
        varying_length_max: impl Into<Length>,
        static_length: impl Into<Length>,
    ) -> Self {
        Self {
            value: value.clamp(0.0, 100.0),
            bar_thickness: static_length.into(),
            varying_length_max: varying_length_max.into(),
            class: T::default(),
        }
    }
}

impl<'a, M, T, R> Widget<M, T, R> for HorizontalPercentageBar<'a, T>
where
    R: iced::core::Renderer,
    T: progress_bar::Catalog,
{
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
        _tree: &mut cosmic::iced_core::widget::Tree,
        _renderer: &R,
        limits: &cosmic::iced_core::layout::Limits,
    ) -> cosmic::iced_core::layout::Node {
        let Size { width, height } =
            <HorizontalPercentageBar<'a, T> as Widget<M, T, R>>::size(self);
        layout::atomic(limits, width, height).align(
            Alignment::Start,
            Alignment::Center,
            limits.max(),
        )
    }

    fn draw(
        &self,
        _tree: &cosmic::iced_core::widget::Tree,
        renderer: &mut R,
        theme: &T,
        _style: &cosmic::iced_core::renderer::Style,
        layout: cosmic::iced_core::Layout<'_>,
        _cursor: cosmic::iced_core::mouse::Cursor,
        _viewport: &cosmic::iced::Rectangle,
    ) {
        renderer.fill_quad(
            renderer::Quad {
                bounds: layout.bounds(),
                ..renderer::Quad::default()
            },
            theme.style(&self.class).bar,
        );
    }
}

impl<'a, M, T, R> From<HorizontalPercentageBar<'a, T>> for iced::core::Element<'a, M, T, R>
where
    M: 'a,
    T: 'a + progress_bar::Catalog,
    R: 'a + iced::core::Renderer,
{
    fn from(value: HorizontalPercentageBar<'a, T>) -> Self {
        iced::core::Element::new(value)
    }
}
