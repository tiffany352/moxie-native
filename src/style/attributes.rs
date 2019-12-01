use super::{ComputedValues, Direction, DisplayType};
use crate::layout::{LogicalLength, LogicalSize};
use crate::Color;
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

#[derive(Clone, PartialEq)]
pub struct SideOffsets {
    pub left: Option<Value>,
    pub right: Option<Value>,
    pub top: Option<Value>,
    pub bottom: Option<Value>,
}

#[derive(Clone, PartialEq)]
pub struct CommonAttributes {
    pub display: Option<Display>,
    pub direction: Option<Direction>,
    pub text_size: Option<Value>,
    pub text_color: Option<Color>,
    pub font_family: Option<Cow<'static, str>>,
    pub font_weight: Option<u32>,
    pub background_color: Option<Color>,
    pub border_radius: Option<Value>,
    pub padding: SideOffsets,
    pub margin: SideOffsets,
    pub width: Option<Value>,
    pub height: Option<Value>,
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
    padding: SideOffsets {
        left: None,
        right: None,
        top: None,
        bottom: None,
    },
    margin: SideOffsets {
        left: None,
        right: None,
        top: None,
        bottom: None,
    },
    width: None,
    height: None,
};

impl Default for CommonAttributes {
    fn default() -> Self {
        DEFAULT_ATTRIBUTES
    }
}

impl CommonAttributes {
    #[topo::from_env(viewport_size: &LogicalSize)]
    pub(super) fn apply(&self, values: &mut ComputedValues) {
        let ctx = ValueContext {
            pixels_per_em: 16.0, // todo
            viewport: *viewport_size,
        };
        if let Some(display) = self.display {
            match display {
                Display::Block => values.display = DisplayType::Block(Default::default()),
                Display::Inline => values.display = DisplayType::Inline(Default::default()),
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
        if let Some(ref padding) = self.padding.left {
            if let DisplayType::Block(ref mut block) = values.display {
                block.padding.left = padding.resolve(&ctx).get();
            }
        }
        if let Some(ref padding) = self.padding.right {
            if let DisplayType::Block(ref mut block) = values.display {
                block.padding.right = padding.resolve(&ctx).get();
            }
        }
        if let Some(ref padding) = self.padding.top {
            if let DisplayType::Block(ref mut block) = values.display {
                block.padding.top = padding.resolve(&ctx).get();
            }
        }
        if let Some(ref padding) = self.padding.bottom {
            if let DisplayType::Block(ref mut block) = values.display {
                block.padding.bottom = padding.resolve(&ctx).get();
            }
        }
        if let Some(ref margin) = self.margin.left {
            if let DisplayType::Block(ref mut block) = values.display {
                block.margin.left = margin.resolve(&ctx).get();
            }
        }
        if let Some(ref margin) = self.margin.right {
            if let DisplayType::Block(ref mut block) = values.display {
                block.margin.right = margin.resolve(&ctx).get();
            }
        }
        if let Some(ref margin) = self.margin.top {
            if let DisplayType::Block(ref mut block) = values.display {
                block.margin.top = margin.resolve(&ctx).get();
            }
        }
        if let Some(ref margin) = self.margin.bottom {
            if let DisplayType::Block(ref mut block) = values.display {
                block.margin.bottom = margin.resolve(&ctx).get();
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
