use crate::applet::Message;
use cosmic::{
    applet, iced,
    iced::{Alignment::Center, Length, Length::Fixed, Size},
    widget::{container, Column, Row},
    Element, Theme,
};

pub fn collection<'a>(
    context: &'_ cosmic::applet::Context,
    elements: impl IntoIterator<Item = Element<'a, Message>>,
    spacing: impl Into<cosmic::iced_core::Pixels>,
    padding: f32,
) -> Element<'a, Message> {
    if context.is_horizontal() {
        Row::with_children(elements)
            .spacing(spacing)
            .align_y(Center)
            .padding(padding)
            .into()
    } else {
        Column::with_children(elements)
            .spacing(spacing)
            .align_x(Center)
            .padding(padding)
            .into()
    }
}

pub fn base_background(theme: &Theme) -> container::Style {
    container::Style {
        background: Some(iced::Color::from(theme.cosmic().primary.base).into()),
        ..container::Style::default()
    }
}

pub fn get_sized_aspect_ratio(context: &applet::Context, aspect_ratio: f32) -> Size<Length> {
    let (suggested_width, suggested_height) = context.suggested_size(false);

    if context.is_horizontal() {
        Size {
            width: Fixed(suggested_height as f32 * aspect_ratio),
            height: Fixed(suggested_height as f32),
        }
    } else {
        Size {
            width: Fixed(suggested_width as f32),
            height: Fixed(suggested_width as f32 * aspect_ratio),
        }
    }
}