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

#[derive(Clone, Copy)]
pub enum SortMethod {
    Descending,
    #[allow(dead_code)]
    Ascending,
}

#[derive(Clone)]
pub enum Orientation {
    PointingUp,
    #[allow(dead_code)]
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

/*#[derive(Clone)]
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
*/
// pub fn percentage_histogram<'a>(
//     mut values: Box<[f32]>,
//     config: BarConfig,
//     color: CosmicColor,
// ) -> Container<'a, Message, Theme> {
//     let full_length = if let Length::Fixed(config_length) = config.full_length {
//         config_length
//     } else {
//         50.0
//     };
//
//     let static_dimension = full_length * config.width_fraction;
//
//     if let Some(sort_method) = config.sort_method {
//         match sort_method {
//             SortMethod::Descending => values.sort_by(|a, b| b.partial_cmp(a).unwrap()),
//             SortMethod::Ascending => values.sort_by(|a, b| a.partial_cmp(b).unwrap()),
//         }
//     }
//
//     let inner: Element<Message> = match config.orientation {
//         Orientation::PointingUp => Row::with_children(values.iter().map(|&val| {
//             VerticalPercentageBar::new(val, color)
//                 .apply(container)
//                 .width(static_dimension)
//                 .apply(Element::new)
//         }))
//             .height(full_length)
//             .align_y(Vertical::Bottom)
//             .spacing(config.spacing)
//             .into(),
//         Orientation::PointingRight => Column::with_children(values.iter().map(|&val| {
//             HorizontalPercentageBar::new(val, color)
//                 .apply(container)
//                 .height(static_dimension)
//                 .apply(Element::new)
//         }))
//             .width(full_length)
//             .align_x(Horizontal::Left)
//             .spacing(config.spacing)
//             .into(),
//     };
//
//     let outer = cosmic::widget::container(inner).style(|_| container::Style {
//         ..container::Style::default()
//     });
//     outer
// }

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
}

const HELLO: &str = "Hello, world!";

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
    color: Color,
}

impl<'a> VerticalPercentageBar {
    pub fn new(value: f32, color: Color) -> Self {
        VerticalPercentageBar {
            percentage: value.clamp(0.0, 100.0),
            color,
        }
    }
}

impl<'a> Widget<Message, Theme, Renderer> for VerticalPercentageBar {
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
