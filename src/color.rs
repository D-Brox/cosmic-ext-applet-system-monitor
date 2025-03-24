use cosmic::{
    cosmic_theme::palette::{encoding::srgb::Srgb, rgb::Rgb},
    Theme,
};
use plotters::style::RGBColor;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[allow(non_camel_case_types)]
/// Enum that bundles [Theme] dependent colors with ordinary RGB colors
///
/// All fieldless variants are mapped into the field of [CosmicPaletteInner](cosmic::cosmic_theme::CosmicPaletteInner) with the same name.
///
/// Any RGB color can be created from using the [Color::rgb] containing a hexcode
pub enum Color {
    gray_1,
    gray_2,
    neutral_0,
    neutral_1,
    neutral_2,
    neutral_3,
    neutral_4,
    neutral_5,
    neutral_6,
    neutral_7,
    neutral_8,
    neutral_9,
    neutral_10,
    bright_green,
    bright_red,
    bright_orange,
    ext_warm_grey,
    ext_orange,
    ext_yellow,
    ext_blue,
    ext_purple,
    ext_pink,
    ext_indigo,
    accent_blue,
    accent_red,
    accent_green,
    accent_warm_grey,
    accent_orange,
    accent_yellow,
    accent_purple,
    accent_pink,
    accent_indigo,
    rgb(String),
}

impl Color {
    pub fn as_rgb_color(&self, theme: &Theme) -> RGBColor {
        let accent_color = theme.cosmic().accent_color();
        let palette = &theme.cosmic().palette;
        color_to_rgb(match self {
            Color::gray_1 => palette.gray_1.color,
            Color::gray_2 => palette.gray_2.color,
            Color::neutral_0 => palette.neutral_0.color,
            Color::neutral_1 => palette.neutral_1.color,
            Color::neutral_2 => palette.neutral_2.color,
            Color::neutral_3 => palette.neutral_3.color,
            Color::neutral_4 => palette.neutral_4.color,
            Color::neutral_5 => palette.neutral_5.color,
            Color::neutral_6 => palette.neutral_6.color,
            Color::neutral_7 => palette.neutral_7.color,
            Color::neutral_8 => palette.neutral_8.color,
            Color::neutral_9 => palette.neutral_9.color,
            Color::neutral_10 => palette.neutral_10.color,
            Color::bright_green => palette.bright_green.color,
            Color::bright_red => palette.bright_red.color,
            Color::bright_orange => palette.bright_orange.color,
            Color::ext_warm_grey => palette.ext_warm_grey.color,
            Color::ext_orange => palette.ext_orange.color,
            Color::ext_yellow => palette.ext_yellow.color,
            Color::ext_blue => palette.ext_blue.color,
            Color::ext_purple => palette.ext_purple.color,
            Color::ext_pink => palette.ext_pink.color,
            Color::ext_indigo => palette.ext_indigo.color,
            Color::accent_blue => palette.accent_blue.color,
            Color::accent_red => palette.accent_red.color,
            Color::accent_green => palette.accent_green.color,
            Color::accent_warm_grey => palette.accent_warm_grey.color,
            Color::accent_orange => palette.accent_orange.color,
            Color::accent_yellow => palette.accent_yellow.color,
            Color::accent_purple => palette.accent_purple.color,
            Color::accent_pink => palette.accent_pink.color,
            Color::accent_indigo => palette.accent_indigo.color,
            Color::rgb(s) => s
                .parse::<Rgb<Srgb, u8>>()
                .map(Rgb::into_format::<f32>)
                .unwrap_or(*accent_color),
        })
    }
}

pub fn color_to_rgb(color: Rgb<Srgb, f32>) -> RGBColor {
    let rgb = color.into_format::<u8>();
    RGBColor(rgb.red, rgb.green, rgb.blue)
}
