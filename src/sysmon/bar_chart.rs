use cosmic::cosmic_theme::palette::Srgba;
use cosmic::theme::CosmicColor;
use cosmic::{
    iced::{
        self,
        alignment::{Alignment, Horizontal, Vertical},
        core::{layout, mouse, renderer, widget::Tree, Layout, Length, Rectangle, Size},
    },
    widget::{container, Column, Container, Row, Widget},
    Element, Renderer, Theme,
};
use renderer::Style;

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
/*impl Orientation {
    pub(crate) fn default_for(anchor: PanelAnchor) -> Orientation {
        match anchor {
            PanelAnchor::Left => Orientation::PointingRight,
            PanelAnchor::Right => Orientation::PointingRight,
            PanelAnchor::Top | PanelAnchor::Bottom => Orientation::PointingUp,
        }
    }
}*/

#[derive(Clone)]
pub struct BarConfig {
    pub orientation: Orientation, // todo replace with ~core.applet.Anchor~
    /// The length in the direction the bar varies it's length
    pub full_length: Length,
    pub width_fraction: f32,
    pub spacing: f32,
    pub sort_method: Option<SortMethod>,
}

impl Default for BarConfig {
    fn default() -> Self {
        Self {
            orientation: Orientation::PointingUp,
            full_length: Length::Fill,
            width_fraction: 0.25,
            spacing: 1.0,
            sort_method: Some(SortMethod::Descending),
        }
    }
}

pub fn percentage_histogram<'a>(
    mut values: Box<[f32]>,
    config: BarConfig,
    color: CosmicColor,
) -> Container<'a, Message, Theme> {
    let full_length = if let Length::Fixed(config_length) = config.full_length {
        config_length
    } else {
        50.0
    };

    let static_dimension = full_length * config.width_fraction;

    if let Some(sort_method) = config.sort_method {
        match sort_method {
            SortMethod::Descending => values.sort_by(|a, b| b.partial_cmp(a).unwrap()),
            SortMethod::Ascending => values.sort_by(|a, b| a.partial_cmp(b).unwrap()),
        }
    }

    let inner: Element<Message> = match config.orientation {
        Orientation::PointingUp => Row::with_children(values.iter().map(|&val| {
            VerticalPercentageBar::new(val, full_length, static_dimension, color).into()
        }))
        .height(full_length)
        .align_y(Vertical::Bottom)
        .spacing(config.spacing)
        .into(),
        Orientation::PointingRight => Column::with_children(values.iter().map(|&val| {
            HorizontalPercentageBar::new(val, full_length, static_dimension, color).into()
        }))
        .width(full_length)
        .align_x(Horizontal::Left)
        .spacing(config.spacing)
        .into(),
    };

    let outer = cosmic::widget::container(inner).style(|_| container::Style {
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
        width: impl Into<Length>,
        height: impl Into<Length>,
        color: CosmicColor,
    ) -> Self {
        if is_horizontal {
            Self::Vertical(VerticalPercentageBar::new(value, height, width, color))
        } else {
            Self::Horizontal(HorizontalPercentageBar::new(value, width, height, color))
        }
    }
}

/*impl Deref for PercentageBar {
    type Target = dyn Widget<Message, Theme, Renderer>;

    fn deref(&self) -> &Self::Target {
        match self {
            PercentageBar::Vertical(v) => v,
            PercentageBar::Horizontal(h) => h
        }
    }
}*/

impl From<PercentageBar> for Element<'_, Message> {
    fn from(value: PercentageBar) -> Self {
        Element::new(value)
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
    color: Srgba,
}

impl<'a> VerticalPercentageBar {
    pub fn new(
        value: f32,
        height: impl Into<Length>,
        width: impl Into<Length>,
        color: Srgba,
    ) -> Self {
        VerticalPercentageBar {
            percentage: value.clamp(0.0, 100.0),
            width: width.into(),
            varying_length_max: height.into(),
            color,
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
        _theme: &Theme,
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
            iced::Color::from(self.color),
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
    color: Srgba,
}
impl HorizontalPercentageBar {
    pub fn new(
        value: f32,
        varying_length_max: impl Into<Length>,
        static_length: impl Into<Length>,
        color: impl Into<Srgba>,
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
        _theme: &Theme,
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

impl From<HorizontalPercentageBar> for Element<'_, Message> {
    fn from(value: HorizontalPercentageBar) -> Self {
        Element::new(value)
    }
}
