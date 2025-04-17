use std::cmp::Ordering;

use crate::{applet::Message, color::Color};
use cosmic::{
    cosmic_theme::palette::WithAlpha,
    iced::{
        self,
        core::{layout, mouse, renderer, widget::Tree, Layout, Length, Rectangle, Size},
        Length::Fill,
    },
    widget::Widget,
    Element, Renderer, Theme,
};
use renderer::Style;
use serde::{Deserialize, Serialize};

#[derive(Copy, Clone, Debug, Serialize, Deserialize, PartialEq)]
pub enum SortMethod {
    Ascending,
    Descending,
}
impl SortMethod {
    pub fn method(&self) -> impl FnMut(&f32, &f32) -> Ordering {
        match self {
            SortMethod::Descending => {
                |a: &f32, b: &f32| b.partial_cmp(a).unwrap_or(Ordering::Equal)
            }
            SortMethod::Ascending => |a: &f32, b: &f32| a.partial_cmp(b).unwrap_or(Ordering::Equal),
        }
    }
}

pub enum PercentageBar {
    Vertical(VerticalPercentageBar),
    Horizontal(HorizontalPercentageBar),
}
impl PercentageBar {
    pub(crate) fn new(is_horizontal: bool, value: f32, color: Color) -> Self {
        if is_horizontal {
            Self::Vertical(VerticalPercentageBar::new(value, color))
        } else {
            Self::Horizontal(HorizontalPercentageBar::new(value, color))
        }
    }
    pub(crate) fn from_pair(is_horizontal: bool, current: u64, max: u64, color: Color) -> Self {
        let value = current as f32 / max as f32 * 100.0;
        Self::new(is_horizontal, value, color)
    }
}

impl From<PercentageBar> for Element<'_, Message> {
    fn from(value: PercentageBar) -> Self {
        Element::new(value)
    }
}

impl Widget<Message, Theme, Renderer> for PercentageBar {
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
    color: Color,
}

impl VerticalPercentageBar {
    pub fn new(value: f32, color: Color) -> Self {
        VerticalPercentageBar {
            percentage: value.clamp(0.0, 100.0),
            color,
        }
    }

    pub fn from_pair(current: u64, max: u64, color: Color) -> Self {
        let value = current as f32 / max as f32 * 100.0;
        Self::new(value, color)
    }
}

impl Widget<Message, Theme, Renderer> for VerticalPercentageBar {
    fn size(&self) -> Size<Length> {
        Size::new(Fill, Fill)
    }

    fn layout(
        &self,
        _tree: &mut Tree,
        _renderer: &Renderer,
        limits: &layout::Limits,
    ) -> layout::Node {
        let Size { width, height } = Widget::size(self);

        layout::atomic(limits, width, height)
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
        let outer_rect = &layout.bounds();
        let fill_height = self.percentage / 100.0 * outer_rect.height;
        let fill_rect = Rectangle {
            y: outer_rect.y + outer_rect.height - fill_height,
            height: fill_height,
            ..*outer_rect
        };

        let edge_line_thickness = 0.01 * outer_rect.height;

        // line
        let line_color = self.color.as_cosmic_color(theme);
        iced::core::Renderer::fill_quad(
            renderer,
            renderer::Quad {
                bounds: Rectangle {
                    y: fill_rect.y - edge_line_thickness,
                    height: edge_line_thickness,
                    ..*outer_rect
                },
                ..renderer::Quad::default()
            },
            iced::Color::from(line_color),
        );

        // fill below line
        iced::core::Renderer::fill_quad(
            renderer,
            renderer::Quad {
                bounds: fill_rect,
                ..renderer::Quad::default()
            },
            // make the fill more transparent
            iced::Color::from(line_color.with_alpha(line_color.alpha / 2.0)),
        );
    }
}

impl<'a> From<VerticalPercentageBar> for Element<'a, Message> {
    fn from(percentage_tower: VerticalPercentageBar) -> Element<'a, Message> {
        Element::new(percentage_tower)
    }
}

pub struct HorizontalPercentageBar {
    percentage: f32,
    color: Color,
}
impl HorizontalPercentageBar {
    pub fn new(value: f32, color: Color) -> Self {
        Self {
            percentage: value.clamp(0.0, 100.0),
            color,
        }
    }
}

impl Widget<Message, Theme, Renderer> for HorizontalPercentageBar {
    fn size(&self) -> Size<Length> {
        Size::new(Fill, Fill)
    }

    fn layout(
        &self,
        _tree: &mut Tree,
        _renderer: &Renderer,
        limits: &layout::Limits,
    ) -> layout::Node {
        let Size { width, height } = Widget::size(self);

        layout::atomic(limits, width, height)
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
        let outer_rect = &layout.bounds();
        let fill_width = self.percentage / 100.0 * outer_rect.height;
        let fill_rect = Rectangle {
            x: outer_rect.x + outer_rect.width - fill_width,
            width: fill_width,
            ..*outer_rect
        };

        let edge_line_thickness = 0.01 * outer_rect.height;

        let line_color = self.color.as_cosmic_color(theme);
        iced::core::Renderer::fill_quad(
            renderer,
            renderer::Quad {
                bounds: Rectangle {
                    x: fill_rect.x,
                    width: edge_line_thickness,
                    ..*outer_rect
                },
                ..renderer::Quad::default()
            },
            iced::Color::from(line_color),
        );
        iced::core::Renderer::fill_quad(
            renderer,
            renderer::Quad {
                bounds: fill_rect,
                ..renderer::Quad::default()
            },
            // make the fill more transparent
            iced::Color::from(line_color.with_alpha(line_color.alpha / 2.0)),
        );
    }
}

impl From<HorizontalPercentageBar> for Element<'_, Message> {
    fn from(value: HorizontalPercentageBar) -> Self {
        Element::new(value)
    }
}
