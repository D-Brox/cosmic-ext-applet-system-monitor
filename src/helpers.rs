use crate::applet::Message;
use cosmic::iced::Alignment::Center;
use cosmic::widget::{container, Column, Row};
use cosmic::{iced, Element, Theme};

pub fn collection<'a>(
    context: &'a cosmic::applet::Context,
    elements: impl IntoIterator<Item = Element<'a, Message>>,
    spacing: impl Into<cosmic::iced_core::Pixels>,
) -> Element<'a, Message> {
    if context.is_horizontal() {
        Row::with_children(elements)
            .spacing(spacing)
            .align_y(Center)
            .into()
    } else {
        Column::with_children(elements)
            .spacing(spacing)
            .align_x(Center)
            .into()
    }
}

pub fn base_background(theme: &Theme) -> container::Style {
    container::Style {
        background: Some(iced::Color::from(theme.cosmic().primary.base).into()),
        ..container::Style::default()
    }
}

// pub enum CollectionWidget {
//     Row(Row),
//     Column(Column),
// }
