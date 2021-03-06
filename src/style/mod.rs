use crate::dom::element::ElementState;
use crate::layout::{LogicalLength, LogicalSideOffsets};
use crate::Color;
use std::any::TypeId;

/// Specifies which direction layout should be performed in.
#[derive(Clone, PartialEq, Copy, Debug)]
pub(crate) enum Direction {
    Vertical,
    Horizontal,
}

#[derive(Default, PartialEq, Clone, Copy, Debug)]
pub(crate) struct InlineValues {}

#[derive(PartialEq, Clone, Copy, Debug)]
pub(crate) struct BlockValues {
    pub direction: Direction,
    pub margin: LogicalSideOffsets,
    pub padding: LogicalSideOffsets,
    pub width: Option<LogicalLength>,
    pub height: Option<LogicalLength>,
    pub min_width: Option<LogicalLength>,
    pub min_height: Option<LogicalLength>,
    pub max_width: Option<LogicalLength>,
    pub max_height: Option<LogicalLength>,
}

impl Default for BlockValues {
    fn default() -> Self {
        BlockValues {
            direction: Direction::Vertical,
            margin: LogicalSideOffsets::new_all_same(0.0),
            padding: LogicalSideOffsets::new_all_same(0.0),
            width: None,
            height: None,
            min_width: None,
            min_height: None,
            max_width: None,
            max_height: None,
        }
    }
}

#[derive(PartialEq, Clone, Copy, Debug)]
pub(crate) enum Display {
    Inline,
    Block,
}

#[derive(PartialEq, Clone, Copy, Debug)]
pub(crate) enum DisplayType {
    Inline(InlineValues),
    Block(BlockValues),
}

#[derive(Copy, Clone, PartialEq, Eq, Debug, Default)]
pub(crate) struct Edges<Value> {
    pub left: Value,
    pub right: Value,
    pub top: Value,
    pub bottom: Value,
}

impl<Value> Edges<Value>
where
    Value: Clone,
{
    pub fn new_all_same(value: Value) -> Edges<Value> {
        Edges {
            left: value.clone(),
            right: value.clone(),
            top: value.clone(),
            bottom: value,
        }
    }
}

impl<Value> Edges<Value> {
    pub fn map<Output>(self, func: impl Fn(Value) -> Output) -> Edges<Output> {
        Edges {
            left: func(self.left),
            right: func(self.right),
            top: func(self.top),
            bottom: func(self.bottom),
        }
    }
}

#[derive(Copy, Clone, PartialEq, Debug)]
pub enum BorderStyle {
    None,
    Solid,
    Double,
    Dotted,
    Dashed,
    Hidden,
    Groove,
    Ridge,
    Inset,
    Outset,
}

#[derive(Copy, Clone, PartialEq, Debug)]
pub struct Border {
    pub width: LogicalLength,
    pub style: BorderStyle,
    pub color: Color,
}

impl Border {
    pub fn visible(&self) -> bool {
        self.width.get() > 0.0 && self.style != BorderStyle::None && self.color.alpha > 0
    }
}

impl Edges<Border> {
    pub fn visible(&self) -> bool {
        self.left.visible() || self.right.visible() || self.top.visible() || self.bottom.visible()
    }
}

#[derive(Copy, Clone, PartialEq, Eq, Debug, Default)]
pub(crate) struct Corners<Value> {
    pub top_left: Value,
    pub top_right: Value,
    pub bottom_left: Value,
    pub bottom_right: Value,
}

impl<Value> Corners<Value>
where
    Value: Clone,
{
    pub fn new_all_same(value: Value) -> Corners<Value> {
        Corners {
            top_left: value.clone(),
            top_right: value.clone(),
            bottom_left: value.clone(),
            bottom_right: value,
        }
    }
}

#[derive(PartialEq, Clone, Copy, Debug)]
pub struct ComputedValues {
    pub(crate) display: DisplayType,
    pub(crate) text_size: LogicalLength,
    pub(crate) text_color: Color,
    pub(crate) background_color: Color,
    pub(crate) border: Edges<Border>,
    pub(crate) corner_radius: Corners<LogicalLength>,
}

impl Default for ComputedValues {
    fn default() -> Self {
        ComputedValues {
            display: DisplayType::Block(BlockValues::default()),
            text_size: LogicalLength::new(16.0),
            text_color: Color::black(),
            background_color: Color::clear(),
            corner_radius: Corners::new_all_same(LogicalLength::new(0.0)),
            border: Edges::new_all_same(Border {
                width: LogicalLength::new(0.0),
                style: BorderStyle::None,
                color: Color::clear(),
            }),
        }
    }
}

pub trait NodeSelect {
    fn has_type(&self, ty: TypeId) -> bool;
    fn has_state(&self, state: ElementState) -> bool;
}

impl dyn NodeSelect {
    pub fn is_type<Ty: 'static>(&self) -> bool {
        self.has_type(TypeId::of::<Ty>())
    }
}

pub type Selector = fn(&dyn NodeSelect) -> bool;
pub type ApplyFunc = fn(&mut ComputedValues);
pub type GetAttributes = fn() -> Vec<(&'static str, String)>;

pub struct Attributes {
    pub apply: ApplyFunc,
    pub get_attributes: GetAttributes,
}

impl std::fmt::Debug for Attributes {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "Attributes {{ ... }}")
    }
}

pub struct SubStyle {
    pub selector: Selector,
    pub attributes: Attributes,
}

impl std::fmt::Debug for SubStyle {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "SubStyle {{ ... }}")
    }
}

/// Affects the presentation of elements that are chosen based on the
/// selector. See `style!` for how you define this.
#[derive(Debug)]
pub struct StyleData {
    pub attributes: Attributes,
    pub sub_styles: &'static [SubStyle],
    pub name: &'static str,
    pub file: &'static str,
    pub line: u32,
}

#[derive(Copy, Clone, Debug)]
pub struct Style(pub &'static StyleData);

impl Style {
    pub fn name(self) -> &'static str {
        self.0.name
    }

    pub fn file(self) -> (&'static str, u32) {
        (self.0.file, self.0.line)
    }
}

impl PartialEq for Style {
    fn eq(&self, other: &Style) -> bool {
        std::ptr::eq(self.0 as *const StyleData, other.0 as *const StyleData)
    }
}
