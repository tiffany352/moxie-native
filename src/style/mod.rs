use crate::dom::{Element, Node, Window};
use crate::layout::{LogicalLength, LogicalSideOffsets, LogicalSize};
use crate::Color;
use moxie::embed::Runtime;
use std::any::{Any, TypeId};
use std::borrow::Cow;

#[derive(Clone, PartialEq)]
pub enum Selector {
    ElementType(TypeId),
    ClassName(Cow<'static, str>),
    HasParent(Box<Selector>),
    HasAncestor(Box<Selector>),
    IsFirstChild,
    IsLastChild,
    IsEvenChild,
    IsOddChild,
}

impl Selector {
    pub fn select<Elt>(&self, node: &Node<Elt>) -> bool
    where
        Elt: Element,
    {
        match self {
            Selector::ElementType(type_id) => node.element().type_id() == *type_id,
            Selector::ClassName(class) => node
                .element()
                .class_name()
                .map(|value| value == class)
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

#[derive(Default, Clone, PartialEq)]
pub struct CommonAttributes {
    pub text_size: Option<Value>,
    pub text_color: Option<Color>,
    pub font_family: Option<Cow<'static, str>>,
    pub font_weight: Option<u32>,
    pub background_color: Option<Color>,
}

pub struct InlineValues {}

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

pub enum DisplayType {
    Inline(InlineValues),
    Block(BlockValues),
}

pub struct ComputedValues {
    pub display: DisplayType,
    pub text_size: LogicalLength,
    pub text_color: Color,
    pub background_color: Color,
}

#[derive(Clone, PartialEq)]
pub struct Style {
    pub selector: Selector,
    pub attributes: CommonAttributes,
}

impl Style {
    #[topo::from_env(viewport_size: &LogicalSize)]
    fn apply<Elt>(&self, _node: &Node<Elt>, values: &mut ComputedValues)
    where
        Elt: Element,
    {
        let ctx = ValueContext {
            pixels_per_em: 16.0, // todo
            viewport: *viewport_size,
        };
        if let Some(ref text_size) = self.attributes.text_size {
            values.text_size = text_size.resolve(&ctx);
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

    fn apply_style<Elt>(node: &Node<Elt>, chain: &StyleChain, values: &mut ComputedValues)
    where
        Elt: Element,
    {
        if let Some(parent) = chain.parent {
            Self::apply_style(node, parent, values);
        }
        for style in chain.styles {
            if style.selector.select(node) {
                style.apply(node, values);
            }
        }
    }

    fn update_style<Elt>(node: &Node<Elt>, chain: &StyleChain)
    where
        Elt: Element,
    {
        let mut computed = ComputedValues {
            display: DisplayType::Block(BlockValues {
                margin: LogicalSideOffsets::new_all_same(0.0),
                padding: LogicalSideOffsets::new_all_same(0.0),
                width: None,
                height: None,
                min_width: None,
                min_height: None,
                max_width: None,
                max_height: None,
            }),
            text_size: LogicalLength::new(16.0),
            text_color: Color::black(),
            background_color: Color::clear(),
        };
        Self::apply_style(node, chain, &mut computed);
        node.computed_values().set(Some(computed));
    }

    #[topo::from_env(node: &Node<Window>)]
    fn run_styling() {
        let chain = StyleChain {
            styles: node.element().styles(),
            parent: None,
        };
        Self::update_style(node, &chain);

        for child in node.children() {
            Self::update_style(
                child,
                &StyleChain {
                    styles: child.element().styles(),
                    parent: Some(&chain),
                },
            );
        }
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
