use crate::layout::{LogicalLength, LogicalSize};
use crate::style::ComputedValues;
use std::ops;

pub mod attribute;
pub mod func;
pub mod keyword;
pub mod state;
pub mod types;

pub trait Attribute {}

pub trait AttributeHasValue<Value> {
    fn set(&self, values: &mut ComputedValues, value: Value);
}

pub struct Inherit;

pub fn apply<Attr, Value>(values: &mut ComputedValues, attribute: Attr, value: Value)
where
    Attr: Attribute,
    Attr: AttributeHasValue<Value>,
{
    attribute.set(values, value);
}

#[derive(Copy, Clone, PartialEq, Eq)]
pub struct Length(isize);

impl Into<LogicalLength> for Length {
    fn into(self) -> LogicalLength {
        LogicalLength::new(self.0 as f32 / 60.0)
    }
}

impl ops::Add for Length {
    type Output = Self;

    fn add(self, rhs: Self) -> Self {
        Length(self.0 + rhs.0)
    }
}

impl ops::Sub for Length {
    type Output = Self;

    fn sub(self, rhs: Self) -> Self {
        Length(self.0 - rhs.0)
    }
}

pub fn pixels(value: f64) -> Length {
    Length((value * 60.0) as isize)
}

pub fn ems(_value: f64) -> Length {
    unimplemented!()
}

#[illicit::from_env(viewport_size: &LogicalSize)]
pub fn view_width(value: f64) -> Length {
    Length((value / 100.0 * viewport_size.width as f64 * 60.0) as isize)
}

#[illicit::from_env(viewport_size: &LogicalSize)]
pub fn view_height(value: f64) -> Length {
    Length((value / 100.0 * viewport_size.height as f64 * 60.0) as isize)
}
