use crate::dom::{element::children, element::NodeChild, Node, Window};
use crate::layout::{LogicalLength, LogicalSideOffsets, LogicalSize};
use crate::Color;
use moxie::embed::Runtime;
use std::borrow::Cow;

/// Represents a position or size that can be specified in multiple
/// units, which are resolved during styling.
#[derive(Default, Clone, PartialEq)]
pub struct Value {
    pub pixels: f32,
    pub ems: f32,
    pub view_width: f32,
    pub view_height: f32,
}

struct ValueContext {
    pixels_per_em: f32,
    viewport: LogicalSize,
}

impl Value {
    fn resolve(&self, ctx: &ValueContext) -> LogicalLength {
        let pixels = self.pixels
            + self.ems * ctx.pixels_per_em
            + self.view_width * ctx.viewport.width
            + self.view_height * ctx.viewport.height;
        LogicalLength::new(pixels)
    }
}

/// Decides how a given element should be laid out.
#[derive(Clone, PartialEq, Copy)]
pub enum Display {
    /// Treat each child as a box and place them in a linear list.
    Block,
    /// Lay out elements with text wrapping.
    Inline,
}

/// Specifies which direction layout should be performed in.
#[derive(Clone, PartialEq, Copy)]
pub enum Direction {
    Vertical,
    Horizontal,
}

#[derive(Default, Clone, PartialEq)]
pub struct CommonAttributes {
    pub display: Option<Display>,
    pub direction: Option<Direction>,
    pub text_size: Option<Value>,
    pub text_color: Option<Color>,
    pub font_family: Option<Cow<'static, str>>,
    pub font_weight: Option<u32>,
    pub background_color: Option<Color>,
    pub border_radius: Option<Value>,
    pub padding: Option<Value>,
    pub margin: Option<Value>,
    pub width: Option<Value>,
    pub height: Option<Value>,
}

impl CommonAttributes {
    #[topo::from_env(viewport_size: &LogicalSize)]
    fn apply(&self, values: &mut ComputedValues) {
        let ctx = ValueContext {
            pixels_per_em: 16.0, // todo
            viewport: *viewport_size,
        };
        if let Some(display) = self.display {
            match display {
                Display::Block => values.display = DisplayType::Block(BlockValues::default()),
                Display::Inline => values.display = DisplayType::Inline(InlineValues::default()),
            }
        }
        if let Some(direction) = self.direction {
            if let DisplayType::Block(ref mut block) = values.display {
                block.direction = direction;
            }
        }
        if let Some(ref text_size) = self.text_size {
            values.text_size = text_size.resolve(&ctx);
        }
        if let Some(ref padding) = self.padding {
            if let DisplayType::Block(ref mut block) = values.display {
                block.padding = LogicalSideOffsets::from_length_all_same(padding.resolve(&ctx));
            }
        }
        if let Some(ref margin) = self.margin {
            if let DisplayType::Block(ref mut block) = values.display {
                block.margin = LogicalSideOffsets::from_length_all_same(margin.resolve(&ctx));
            }
        }
        if let Some(ref width) = self.width {
            if let DisplayType::Block(ref mut block) = values.display {
                block.width = Some(width.resolve(&ctx));
            }
        }
        if let Some(ref height) = self.height {
            if let DisplayType::Block(ref mut block) = values.display {
                block.height = Some(height.resolve(&ctx));
            }
        }
        if let Some(ref border_radius) = self.border_radius {
            values.border_radius = border_radius.resolve(&ctx);
        }
        if let Some(text_color) = self.text_color {
            values.text_color = text_color;
        }
        if let Some(background_color) = self.background_color {
            values.background_color = background_color;
        }
    }
}

#[derive(Default, PartialEq, Clone, Copy)]
pub struct InlineValues {}

#[derive(PartialEq, Clone, Copy)]
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

#[derive(PartialEq, Clone, Copy)]
pub enum DisplayType {
    Inline(InlineValues),
    Block(BlockValues),
}

#[derive(PartialEq, Clone, Copy)]
pub struct ComputedValues {
    pub display: DisplayType,
    pub text_size: LogicalLength,
    pub text_color: Color,
    pub background_color: Color,
    pub border_radius: LogicalLength,
}

impl Default for ComputedValues {
    fn default() -> Self {
        ComputedValues {
            display: DisplayType::Block(BlockValues::default()),
            text_size: LogicalLength::new(16.0),
            text_color: Color::black(),
            background_color: Color::clear(),
            border_radius: LogicalLength::new(0.0),
        }
    }
}

pub struct SubStyle {
    pub selector: fn(&dyn NodeChild) -> bool,
    pub attributes: CommonAttributes,
}

/// Affects the presentation of elements that are chosen based on the
/// selector. See `style!` for how you define this.
pub struct StyleData {
    pub attributes: CommonAttributes,
    pub sub_styles: &'static [SubStyle],
}

#[derive(Copy, Clone)]
pub struct Style(pub &'static StyleData);

impl PartialEq for Style {
    fn eq(&self, other: &Style) -> bool {
        std::ptr::eq(self.0 as *const StyleData, other.0 as *const StyleData)
    }
}

/// Used to annotate the node tree with computed values from styling.
pub struct StyleEngine {
    runtime: Runtime<fn(), ()>,
}

impl StyleEngine {
    pub fn new() -> StyleEngine {
        StyleEngine {
            runtime: Runtime::new(StyleEngine::run_styling),
        }
    }

    fn update_style(node: &dyn NodeChild) {
        let mut computed = node.create_computed_values();

        let style = node.style();
        if let Some(Style(style)) = style {
            style.attributes.apply(&mut computed);
            for sub_style in style.sub_styles {
                if (sub_style.selector)(node) {
                    sub_style.attributes.apply(&mut computed);
                }
            }
        }

        if let Ok(values) = node.computed_values() {
            values.set(Some(computed));
        }

        for child in children(node) {
            Self::update_style(child);
        }
    }

    #[topo::from_env(node: &Node<Window>)]
    fn run_styling() {
        Self::update_style(node);
    }

    /// Update the node tree with computed values.
    pub fn update(&mut self, node: Node<Window>, size: LogicalSize) {
        topo::call!(
            { self.runtime.run_once() },
            env! {
                Node<Window> => node,
                LogicalSize => size,
            }
        )
    }
}

pub const DEFAULT_ATTRIBUTES: CommonAttributes = CommonAttributes {
    display: None,
    direction: None,
    text_size: None,
    text_color: None,
    font_family: None,
    font_weight: None,
    background_color: None,
    border_radius: None,
    padding: None,
    margin: None,
    width: None,
    height: None,
};
