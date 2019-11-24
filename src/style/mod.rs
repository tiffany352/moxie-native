use crate::dom::{element::children, Node, NodeChild, Window};
use crate::layout::{LogicalLength, LogicalSideOffsets, LogicalSize};
use crate::Color;
use moxie::embed::Runtime;
use std::any::TypeId;
use std::borrow::Cow;

#[derive(Clone, PartialEq)]
pub enum Selector {
    ElementType(TypeId),
    ClassName(&'static str),
    HasParent(&'static Selector),
    HasAncestor(&'static Selector),
    IsFirstChild,
    IsLastChild,
    IsEvenChild,
    IsOddChild,
}

impl Selector {
    pub fn select(&self, node: &dyn NodeChild) -> bool {
        match self {
            Selector::ElementType(type_id) => node.type_id() == *type_id,
            Selector::ClassName(class) => node
                .class_name()
                .map(|value| value == *class)
                .unwrap_or(false),
            _ => unimplemented!(),
        }
    }
}

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

#[derive(Clone, PartialEq, Copy)]
pub enum Display {
    Block,
    Inline,
}

#[derive(Default, Clone, PartialEq)]
pub struct CommonAttributes {
    pub display: Option<Display>,
    pub text_size: Option<Value>,
    pub text_color: Option<Color>,
    pub font_family: Option<Cow<'static, str>>,
    pub font_weight: Option<u32>,
    pub background_color: Option<Color>,
    pub padding: Option<Value>,
    pub width: Option<Value>,
    pub height: Option<Value>,
}

#[derive(Default, PartialEq, Clone, Copy)]
pub struct InlineValues {}

#[derive(PartialEq, Clone, Copy)]
pub struct BlockValues {
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
}

impl Default for ComputedValues {
    fn default() -> Self {
        ComputedValues {
            display: DisplayType::Block(BlockValues::default()),
            text_size: LogicalLength::new(16.0),
            text_color: Color::black(),
            background_color: Color::clear(),
        }
    }
}

#[derive(Clone, PartialEq)]
pub struct Style {
    pub selector: Selector,
    pub attributes: CommonAttributes,
}

impl Style {
    #[topo::from_env(viewport_size: &LogicalSize)]
    fn apply(&self, values: &mut ComputedValues) {
        let ctx = ValueContext {
            pixels_per_em: 16.0, // todo
            viewport: *viewport_size,
        };
        if let Some(display) = self.attributes.display {
            match display {
                Display::Block => values.display = DisplayType::Block(BlockValues::default()),
                Display::Inline => values.display = DisplayType::Inline(InlineValues::default()),
            }
        }
        if let Some(ref text_size) = self.attributes.text_size {
            values.text_size = text_size.resolve(&ctx);
        }
        if let Some(ref padding) = self.attributes.padding {
            if let DisplayType::Block(ref mut block) = values.display {
                block.padding = LogicalSideOffsets::from_length_all_same(padding.resolve(&ctx));
            }
        }
        if let Some(ref width) = self.attributes.width {
            if let DisplayType::Block(ref mut block) = values.display {
                block.width = Some(width.resolve(&ctx));
            }
        }
        if let Some(ref height) = self.attributes.height {
            if let DisplayType::Block(ref mut block) = values.display {
                block.height = Some(height.resolve(&ctx));
            }
        }
        if let Some(text_color) = self.attributes.text_color {
            values.text_color = text_color;
        }
        if let Some(background_color) = self.attributes.background_color {
            values.background_color = background_color;
        }
    }
}

struct StyleChain<'a> {
    styles: &'a [&'static Style],
    parent: Option<&'a StyleChain<'a>>,
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

    fn apply_style(node: &dyn NodeChild, chain: &StyleChain, values: &mut ComputedValues) {
        if let Some(parent) = chain.parent {
            Self::apply_style(node, parent, values);
        }
        for style in chain.styles {
            if style.selector.select(node) {
                style.apply(values);
            }
        }
    }

    fn update_style(node: &dyn NodeChild, chain: Option<&StyleChain>) {
        let chain = StyleChain {
            parent: chain,
            styles: node.styles(),
        };

        let mut computed = node.create_computed_values();
        Self::apply_style(node, &chain, &mut computed);
        if let Ok(values) = node.computed_values() {
            values.set(Some(computed));
        }

        for child in children(node) {
            Self::update_style(child, Some(&chain));
        }
    }

    #[topo::from_env(node: &Node<Window>)]
    fn run_styling() {
        Self::update_style(node, None);
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

#[macro_export]
macro_rules! style_selector {
    (class_name == $value:expr) => {
        $crate::style::Selector::ClassName($value)
    };
    (element == $element:ty) => {
        $crate::style::Selector::ElementType(::std::any::TypeId::of::<$element>())
    };
}

pub const DEFAULT_ATTRIBUTES: CommonAttributes = CommonAttributes {
    display: None,
    text_size: None,
    text_color: None,
    font_family: None,
    font_weight: None,
    background_color: None,
    padding: None,
    width: None,
    height: None,
};

#[macro_export]
macro_rules! style {
    ( ( $($selector:tt)+ ) => { $( $name:ident : $value:expr ),* $(,)* } ) => {
        & $crate::style::Style {
            selector: style_selector!($($selector)+),
            attributes: $crate::style::CommonAttributes {
                $(
                    $name : Some($value)
                ),*
                , .. $crate::style::DEFAULT_ATTRIBUTES
            }
        }
    };
}

#[macro_export]
macro_rules! styles {
    ( $( ( $($selector:tt)+ ) => { $( $name:ident : $value:expr ),* $(,)* } ),* ) => {
        &'static [
            $(
                style!(($($selector)+) => { $( $name : $value ),+ })
            ),*
        ]
    }
}
