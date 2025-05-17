use cosmic::{
    cosmic_theme::palette::WithAlpha,
    iced::{core::mouse, Point, Rectangle},
    widget::{
        canvas::{path, stroke, Fill, Frame, Geometry, Program, Stroke},
        Canvas,
    },
    Element, Renderer, Theme,
};

use crate::{applet::Message, color::Color, history::History};

#[derive(Debug)]
pub struct HistoryChart<'a, T = u64> {
    history: &'a History<T>,
    max: T,
    color: Color,
}

// Implementation block for generic HistoryChart functionality, specifically for auto-calculating max value.
// Requires T to support PartialOrd for comparison and Default for initial fold value.
impl<'a, T: Copy + PartialOrd + Default + 'a> HistoryChart<'a, T> {
    /// Creates a `HistoryChart` automatically determining the maximum value from the history data.
    /// Uses `fold` with `partial_cmp` to correctly find the maximum among `T` which might be `f32` (handling NaN).
    pub fn auto_max(history: &'a History<T>, color: Color) -> HistoryChart<'a, T> {
        let max_val = history
            .iter()
            .fold(T::default(), |acc, item| match acc.partial_cmp(item) {
                Some(std::cmp::Ordering::Less) => *item,
                _ => acc,
            });
        HistoryChart::new(history, max_val, color)
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

// Implementation block for generic HistoryChart functionality, specifically for linking max values.
// Requires T to support PartialOrd for comparison.
impl<T: Copy + PartialOrd> HistoryChart<'_, T> {
    /// Links the maximum displayable value between two `HistoryChart` instances.
    /// Ensures both charts use the same scale by setting their `max` to the greater of the two.
    /// Handles `f32` comparison using `partial_cmp`.
    pub fn link_max(front: &mut HistoryChart<'_, T>, back: &mut HistoryChart<'_, T>) {
        let max_val = match front.max.partial_cmp(&back.max) {
            Some(std::cmp::Ordering::Less) => back.max,
            Some(std::cmp::Ordering::Equal | std::cmp::Ordering::Greater) | None => front.max, // Nested or-patterns
        };
        front.max = max_val;
        back.max = max_val;
    }
}

/// This was made generic to handle `u64` and `f32` chart data.
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
                if self.history.len() < 2 {
                    return vec![]; // Early return if not enough data to draw a line
                }

                let mut fill_frame = Frame::new(renderer, bounds.size());
                let mut line_frame = Frame::new(renderer, bounds.size());
                let color = self.color.as_cosmic_color(theme);

                let mut path_builder = path::Builder::new();
                // Adjusted x_step calculation to prevent division by zero if history has less than 2 items.
                let x_step = if self.history.len() > 1 {
                    bounds.width / (self.history.len() - 1) as f32
                } else {
                    bounds.width
                };
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
                fill_frame.fill(
                    &path,
                    Fill {
                        style: stroke::Style::Solid(color.with_alpha(0.5).into()),
                        ..Default::default()
                    },
                );
                line_frame.stroke(
                    &path,
                    Stroke {
                        style: stroke::Style::Solid(color.into()),
                        width: 1.5,
                        ..Default::default()
                    },
                );
                vec![fill_frame.into_geometry(),line_frame.into_geometry()]
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
                    let mut geometries = vec![];

                    // Use the Background implementation
                    geometries.extend(Background.draw(state, renderer, theme, bounds, cursor));

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

impl<'a, T: Copy + PartialOrd + Default + 'a> SimpleHistoryChart<'a, T> {
    /// Creates a `SimpleHistoryChart` automatically determining the maximum value.
    /// Leverages `HistoryChart::auto_max`.
    pub fn auto_max(history: &'a History<T>, color: Color) -> SimpleHistoryChart<'a, T> {
        SimpleHistoryChart::new(
            history,
            HistoryChart::auto_max(history, color).max, // Use the max from the generic auto_max
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

/// A chart that superimposes two data series (front and back).
/// Made generic over `T`.
#[derive(Debug)]
pub struct SuperimposedHistoryChart<'a, T = u64> {
    pub back: HistoryChart<'a, T>,
    pub front: HistoryChart<'a, T>,
}

/// This was made generic for `u64` and `f32` data types.
macro_rules! impl_program_superimposed_history_chart {
    ($($t:ty),+) => {
        $(
            impl<'a> From<SuperimposedHistoryChart<'a, $t>> for Element<'a, Message> {
                fn from(value: SuperimposedHistoryChart<'a, $t>) -> Self {
                    Canvas::new(value).into()
                }
            }

            impl<'a> Program<Message, Theme, Renderer> for SuperimposedHistoryChart<'a, $t>{
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
                    geometries.extend(back.into_iter().zip(front).flat_map(|(b, f)| [b, f]));
                    geometries
                }
            }
        )*
    };
}
impl_program_superimposed_history_chart!(u64, f32);

/// Simple background drawing program for theme adherence.
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

        // Create a rounded rectangle path for the background
        let chart_area_path = path::Path::rounded_rectangle(
            Point::ORIGIN,
            bounds.size(),
            cosmic::iced::core::border::Radius::from(5.0),
        );

        // Use a theme-aware color with appropriate opacity
        // Using neutral_5 with alpha for better visibility in both light and dark themes
        let bg_color = cosmic::iced::Color::from(theme.cosmic().palette.neutral_5.with_alpha(0.3));

        // Fill the rounded rectangle with the background color
        frame.fill(&chart_area_path, bg_color);

        vec![frame.into_geometry()]
    }
}
