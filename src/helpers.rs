use crate::applet::{History, Message};
use cosmic::{
    iced::{self, Alignment::Center, Padding},
    iced_core::Pixels,
    widget::{container, Column, Row},
    Element, Theme,
};

pub fn panel_collection<'a>(
    is_horizontal: bool,
    elements: impl IntoIterator<Item = impl Into<Element<'a, Message>>>,
    spacing: impl Into<Pixels>,
    padding: impl Into<Padding>,
) -> Element<'a, Message> {
    if is_horizontal {
        Row::with_children(elements.into_iter().map(Into::into))
            .spacing(spacing)
            .align_y(Center)
            .padding(padding)
            .into()
    } else {
        Column::with_children(elements.into_iter().map(Into::into))
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

pub fn init_history_with_default<T: Default>(size: impl Into<usize>) -> History<T> {
    let size = size.into();
    let mut history = History::<T>::with_capacity(size);

    for _ in 0..size {
        history.push(Default::default());
    }

    history
}
