use super::Length;
use crate::style::{Border, BorderStyle};
use crate::Color;

pub fn rgb(red: f64, green: f64, blue: f64) -> Color {
    Color::new(red as u8, green as u8, blue as u8, 255)
}

pub fn rgba(red: f64, green: f64, blue: f64, alpha: f64) -> Color {
    Color::new(red as u8, green as u8, blue as u8, alpha as u8)
}

pub fn border(width: Length, style: impl Into<BorderStyle>, color: Color) -> Border {
    let width = width.into();
    let style = style.into();
    Border {
        width,
        style,
        color,
    }
}
