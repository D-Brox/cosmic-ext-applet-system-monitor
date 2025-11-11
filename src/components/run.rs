use cosmic::{
    Element, Renderer, Theme,
    cosmic_theme::palette::WithAlpha,
    iced::{Point, Rectangle, core::mouse},
    widget::{
        Canvas,
        canvas::{Fill, Frame, Geometry, Program, Stroke, path, stroke},
    },
};

use crate::{applet::Message, color::Color, history::History};

#[derive(Debug)]
pub struct HistoryChart<'a, T = u64> {
    history: &'a History<T>,
    max: T,
    color: Color,
}

impl<'a> HistoryChart<'a> {
    pub fn auto_max(history: &'a History, color: Color) -> HistoryChart<'a> {
        HistoryChart::new(history, *history.iter().max().unwrap_or(&0), color)
    }
}

impl<'a, T> HistoryChart<'a, T> {
    pub fn new(history: &'a History<T>, max: T, color: Color) -> HistoryChart<'a, T> {
        HistoryChart {
            history,
            max,
            color,
        }
    }
}

impl<T: Copy + Ord> HistoryChart<'_, T> {
    pub fn link_max(front: &mut HistoryChart<T>, back: &mut HistoryChart<T>) {
        let max_front = front.max;
        let max_back = back.max;
        let max = max_front.max(max_back);
        front.max = max;
        back.max = max;
    }
}

macro_rules! impl_program_history_chart {
    ($($t:ty),+) => {
        $(
        impl Program<Message, Theme, Renderer> for HistoryChart<'_,$t> {
            type State = ();

            #[allow(clippy::cast_precision_loss)]
            fn draw(
                &self,
                _state: &Self::State,
                renderer: &Renderer,
                theme: &Theme,
                bounds: Rectangle,
                _cursor: mouse::Cursor,
            ) -> Vec<Geometry<Renderer>> {
                let mut fill = Frame::new(renderer, bounds.size());
                let mut line = Frame::new(renderer, bounds.size());
                let color = self.color.as_cosmic_color(theme);

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

                for (i, j) in self.history.iter().enumerate() {
                    let x = i as f32 * x_step;
                    let y = bounds.height - *j as f32 * y_step;
                    path_builder.line_to(Point{x,y});
                }

                path_builder.line_to(Point {
                    x: bounds.width,
                    y: bounds.height,
                });

                let path = path_builder.build();
                fill.fill(
                    &path,
                    Fill {
                        style: stroke::Style::Solid(color.with_alpha(0.5).into()),
                        ..Default::default()
                    },
                );
                line.stroke(
                    &path,
                    Stroke {
                        style: stroke::Style::Solid(color.into()),
                        width: 1.0,
                        ..Default::default()
                    },
                );
                vec![fill.into_geometry(),line.into_geometry()]
            }
        })*
    };

}
impl_program_history_chart!(u64, f32);

#[derive(Debug)]
pub struct SimpleHistoryChart<'a, T = u64> {
    history: HistoryChart<'a, T>,
}

macro_rules! impl_program_simple_history_chart {
    ($($t:ty),+) => {
        $(
            impl<'a> From<SimpleHistoryChart<'a, $t>> for Element<'a, Message> {
                fn from(value: SimpleHistoryChart<'a, $t>) -> Self {
                    Canvas::new(value).into()
                }
            }

            impl<'a> Program<Message, Theme, Renderer> for SimpleHistoryChart<'a, $t>{
                type State = ();

                fn draw(
                    &self,
                    state: &Self::State,
                    renderer: &Renderer,
                    theme: &Theme,
                    bounds: Rectangle,
                    cursor: mouse::Cursor,
                ) -> Vec<Geometry<Renderer>> {
                    let mut geometries = Background.draw(state, renderer, theme, bounds, cursor);
                    geometries.extend(self.history.draw(
                        state,
                        renderer,
                        theme,
                        bounds,
                        cursor,
                    ));
                    geometries
                }
            }
        )*
    };
}
impl_program_simple_history_chart!(u64, f32);

impl<'a> SimpleHistoryChart<'a> {
    pub fn auto_max(history: &'a History, color: Color) -> SimpleHistoryChart<'a> {
        SimpleHistoryChart::new(
            history,
            *history.iter().max().unwrap_or(&Default::default()),
            color,
        )
    }
}

impl<'a, T> SimpleHistoryChart<'a, T> {
    pub fn new(history: &'a History<T>, max: T, color: Color) -> SimpleHistoryChart<'a, T> {
        SimpleHistoryChart {
            history: HistoryChart {
                history,
                max,
                color,
            },
        }
    }
}

#[derive(Debug)]
pub struct SuperimposedHistoryChart<'a> {
    pub back: HistoryChart<'a>,
    pub front: HistoryChart<'a>,
}

impl<'a> SuperimposedHistoryChart<'a> {
    pub fn new(
        data_front: &'a History,
        max_front: u64,
        color_front: &Color,
        data_back: &'a History,
        max_back: u64,
        color_back: &Color,
    ) -> Self {
        let back = HistoryChart::new(data_back, max_back, *color_back);
        let front = HistoryChart::new(data_front, max_front, *color_front);
        Self { back, front }
    }

    pub fn new_linked(
        data_front: &'a History<u64>,
        color_front: &Color,
        data_back: &'a History<u64>,
        color_back: &Color,
    ) -> Self {
        let mut back = HistoryChart::auto_max(data_back, *color_back);
        let mut front = HistoryChart::auto_max(data_front, *color_front);
        HistoryChart::link_max(&mut front, &mut back);
        Self { back, front }
    }
}

impl<'a> From<SuperimposedHistoryChart<'a>> for Element<'a, Message> {
    fn from(value: SuperimposedHistoryChart<'a>) -> Self {
        Canvas::new(value).into()
    }
}

impl Program<Message, Theme, Renderer> for SuperimposedHistoryChart<'_> {
    type State = ();

    fn draw(
        &self,
        state: &Self::State,
        renderer: &Renderer,
        theme: &Theme,
        bounds: Rectangle,
        cursor: mouse::Cursor,
    ) -> Vec<Geometry<Renderer>> {
        let mut geometries = Background.draw(state, renderer, theme, bounds, cursor);
        let back = self.back.draw(state, renderer, theme, bounds, cursor);
        let front = self.front.draw(state, renderer, theme, bounds, cursor);
        geometries.extend(back.into_iter().zip(front).flat_map(|(f, b)| [f, b]));
        geometries
    }
}

struct Background;

impl Program<Message, Theme, Renderer> for Background {
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
        let bg_color = theme.cosmic().background.base;

        let mut bg_builder = path::Builder::new();
        let external_bounds = bounds.expand(10.0);
        let Point { x, y } = external_bounds.position();
        bg_builder.move_to(Point { x, y });
        bg_builder.line_to(Point {
            x: x + external_bounds.width,
            y,
        });

        let background = bg_builder.build();

        frame.fill(
            &background,
            Fill {
                style: stroke::Style::Solid(bg_color.into()),
                ..Default::default()
            },
        );
        vec![frame.into_geometry()]
    }
}
