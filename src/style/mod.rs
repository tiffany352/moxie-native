use crate::dom::node::NodeRef;
use crate::layout::{LogicalLength, LogicalSideOffsets};
use crate::Color;

mod attributes;

pub use attributes::*;

/// Specifies which direction layout should be performed in.
#[derive(Clone, PartialEq, Copy, Debug)]
pub enum Direction {
    Vertical,
    Horizontal,
}

#[derive(Default, PartialEq, Clone, Copy, Debug)]
pub struct InlineValues {}

#[derive(PartialEq, Clone, Copy, Debug)]
pub struct BlockValues {
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
pub enum DisplayType {
    Inline(InlineValues),
    Block(BlockValues),
}

#[derive(PartialEq, Clone, Copy, Debug)]
pub struct ComputedValues {
    pub display: DisplayType,
    pub text_size: LogicalLength,
    pub text_color: Color,
    pub background_color: Color,
    pub border_radius: LogicalLength,
    pub border_thickness: LogicalSideOffsets,
    pub border_color: Color,
}

impl Default for ComputedValues {
    fn default() -> Self {
        ComputedValues {
            display: DisplayType::Block(BlockValues::default()),
            text_size: LogicalLength::new(16.0),
            text_color: Color::black(),
            background_color: Color::clear(),
            border_radius: LogicalLength::new(0.0),
            border_thickness: LogicalSideOffsets::new_all_same(0.0),
            border_color: Color::clear(),
        }
    }
}

pub struct SubStyle {
    pub selector: fn(NodeRef) -> bool,
    pub attributes: CommonAttributes,
}

impl std::fmt::Debug for SubStyle {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        f.debug_struct("SubStyle")
            .field("selector", &"<fn(NodeRef) -> bool>")
            .field("attributes", &self.attributes)
            .finish()
    }
}

/// Affects the presentation of elements that are chosen based on the
/// selector. See `style!` for how you define this.
#[derive(Debug)]
pub struct StyleData {
    pub attributes: CommonAttributes,
    pub sub_styles: &'static [SubStyle],
    pub name: &'static str,
    pub file: &'static str,
    pub line: u32,
}

#[derive(Copy, Clone, Debug)]
pub struct Style(pub &'static StyleData);

impl Style {
    pub fn name(&self) -> &'static str {
        self.0.name
    }

    pub fn file(&self) -> (&'static str, u32) {
        (self.0.file, self.0.line)
    }
}

impl PartialEq for Style {
    fn eq(&self, other: &Style) -> bool {
        std::ptr::eq(self.0 as *const StyleData, other.0 as *const StyleData)
    }
}
