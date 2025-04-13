use crate::applet::{History, Message};
use cosmic::{
    applet,
    iced::{self, Alignment::Center, Size},
    widget::{container, Column, Row},
    Element, Theme,
};

pub fn panel_collection<'a>(
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

pub fn get_sized_aspect_ratio(context: &applet::Context, aspect_ratio: f32) -> Size {
    let (suggested_width, suggested_height) = context.suggested_size(false);

    if context.is_horizontal() {
        Size {
            width: suggested_height as f32 * aspect_ratio,
            height: suggested_height as f32,
        }
    } else {
        Size {
            width: suggested_width as f32,
            height: suggested_width as f32 * aspect_ratio,
        }
    }
}

pub fn init_history_with_default<T: Default>(size: impl Into<usize>) -> History<T> {
    let size = size.into();
    let mut history = History::<T>::with_capacity(size);

    for _ in 0..size {
        history.push(Default::default());
    }

    history
}
