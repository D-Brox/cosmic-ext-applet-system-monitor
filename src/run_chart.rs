use cosmic::{
    cosmic_theme::palette::WithAlpha,
    iced::{core::mouse, Point, Rectangle, Vector},
    widget::{
        canvas::{path, stroke, Fill, Frame, Geometry, Program, Stroke},
        Canvas,
    },
    Element, Renderer, Theme,
};

use crate::applet::{History, Message};
use crate::color::Color;

#[derive(Debug)]
pub struct HistoryChart<'a, T = u64> {
    history: &'a History<T>,
    max: T,
    color: Color,
}

impl<'a> HistoryChart<'a, u64> {
    pub fn auto_max(history: &'a History, color: Color) -> HistoryChart<'a> {
        HistoryChart {
            history,
            max: *history.iter().max().unwrap_or(&0),
            color,
        }
    }
}

impl<'a, T> HistoryChart<'a, T> {
    pub fn new(history: &'a History<T>, max: T, color: Color) -> Self {
        HistoryChart {
            history,
            max,
            color,
        }
    }
}

impl<'a> From<HistoryChart<'a>> for Element<'a, Message> {
    fn from(value: HistoryChart<'a>) -> Self {
        Canvas::new(value).into()
    }
}
impl<'a> From<HistoryChart<'a, f32>> for Element<'a, Message> {
    fn from(value: HistoryChart<'a, f32>) -> Self {
        Canvas::new(value).into()
    }
}

macro_rules! impl_program {
    ($($t:ty),+) => {
        $(
        impl Program<Message, Theme, Renderer> for HistoryChart<'_,$t> {
            type State = ();

            fn draw(
                &self,
                _state: &Self::State,
                renderer: &Renderer,
                theme: &Theme,
                bounds: Rectangle,
                _cursor: mouse::Cursor,
            ) -> Vec<Geometry<Renderer>> {
                let mut frame = Frame::new(renderer, bounds.size());
                let line_color = self.color.as_cosmic_color(theme);
                let bg_color = theme.cosmic().background.base;

                let mut bg_builder = path::Builder::new();
                let external_bounds = bounds.expand(10.0);
                let Point { x, y } = external_bounds.position();
                bg_builder.move_to(Point { x, y });
                bg_builder.line_to(Point {
                    x: x + external_bounds.width,
                    y,
                });

                let Point { x, y } = bounds.position();
                let mut path_builder = path::Builder::new();
                let x_step = bounds.width / (self.history.len() - 1) as f32;
                let y_step = if self.max as f32 != 0.0 {
                     bounds.height / self.max as f32
                } else {
                    1.0
                };
                path_builder.move_to(Point {
                    x: 0.0,
                    y: bounds.height,
                });

                for (i, j) in self.history.asc_iter().enumerate() {
                    let x = i as f32 * x_step;
                    let y = bounds.height - *j as f32 * y_step;
                    path_builder.line_to(Point{x,y});
                }

                path_builder.line_to(Point {
                    x: bounds.width,
                    y: bounds.height,
                });

                let path = path_builder.build();
                let background = bg_builder.build();

                frame.fill(
                    &background,
                    Fill {
                        style: stroke::Style::Solid(bg_color.into()),
                        ..Default::default()
                    },
                );
                frame.stroke(
                    &path,
                    Stroke {
                        style: stroke::Style::Solid(line_color.into()),
                        width: 1.0,
                        ..Default::default()
                    },
                );
                frame.fill(
                    &path,
                    Fill {
                        style: stroke::Style::Solid(line_color.with_alpha(0.5).into()),
                        ..Default::default()
                    },
                );
                vec![frame.into_geometry()]
            }
        })*
    };

}
impl_program!(u64, f32);

pub struct SuperimposedHistoryChart<'a> {
    pub back: HistoryChart<'a>,
    pub front: HistoryChart<'a>,
}

impl<'a> From<SuperimposedHistoryChart<'a>> for Element<'a, Message> {
    fn from(value: SuperimposedHistoryChart<'a>) -> Self {
        Canvas::new(value).into()
    }
}

impl<'a> Program<Message, Theme, Renderer> for SuperimposedHistoryChart<'a> {
    type State = ();

    fn draw(
        &self,
        state: &Self::State,
        renderer: &Renderer,
        theme: &Theme,
        bounds: Rectangle,
        cursor: mouse::Cursor,
    ) -> Vec<Geometry<Renderer>> {
        let mut geometries = Program::draw(&self.back, state, renderer, theme, bounds, cursor);
        geometries.extend(Program::draw(
            &self.back, state, renderer, theme, bounds, cursor,
        ));
        geometries
    }
}
