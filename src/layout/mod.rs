//! This module handles creating the layout tree, which includes
//! arranging elements and performing text layout.

use crate::dom::{Node, Window};
use crate::style::DisplayType;
use crate::util::equal_rc::EqualRc;
use euclid::{Length, Point2D, SideOffsets2D, Size2D};
use font_kit::family_name::FamilyName;
use font_kit::properties::Properties;
use font_kit::source::SystemSource;
use moxie::embed::Runtime;
use moxie::*;
use skribo::{FontCollection, FontFamily, FontRef};

mod block;
mod inline;
mod text;

pub struct LogicalPixel;
pub type LogicalPoint = Point2D<f32, LogicalPixel>;
pub type LogicalSize = Size2D<f32, LogicalPixel>;
pub type LogicalLength = Length<f32, LogicalPixel>;
pub type LogicalSideOffsets = SideOffsets2D<f32, LogicalPixel>;

/// Each edge of the layout tree contains information on the positions
/// of the child elements, since elements are positioned relative to
/// their parents, and the position is assigned by the parent.
pub struct LayoutChild {
    /// Child index of the DOM node this child is associated with.
    pub index: usize,
    pub position: LogicalPoint,
    pub layout: EqualRc<LayoutTreeNode>,
}

pub struct Glyph {
    pub index: u32,
    pub offset: LogicalPoint,
}

pub struct TextFragment {
    pub font: FontRef,
    pub glyphs: Vec<Glyph>,
}

/// Information passed to the renderer for rendering text.
pub struct LayoutText {
    pub fragments: Vec<TextFragment>,
    /// The text size of the text.
    pub size: f32,
}

/// One node in the layout tree, which corresponds n:1 with DOM nodes.
pub struct LayoutTreeNode {
    /// The computed size of the node.
    pub size: LogicalSize,
    pub margin: LogicalSideOffsets,
    pub render_text: Option<LayoutText>,
    pub children: Vec<LayoutChild>,
}

/// Used to build the layout tree, with internal caching for
/// performance.
pub struct LayoutEngine {
    runtime: Runtime<fn() -> EqualRc<LayoutTreeNode>, EqualRc<LayoutTreeNode>>,
}

impl LayoutEngine {
    pub fn new() -> LayoutEngine {
        LayoutEngine {
            runtime: Runtime::new(LayoutEngine::run_layout),
        }
    }

    #[topo::from_env(node: &Node<Window>, size: &LogicalSize)]
    fn run_layout() -> EqualRc<LayoutTreeNode> {
        let collection = once!(|| {
            let mut collection = FontCollection::new();
            let source = SystemSource::new();
            let font = source
                .select_best_match(&[FamilyName::SansSerif], &Properties::new())
                .unwrap()
                .load()
                .unwrap();
            collection.add_family(FontFamily::new_from_font(font));

            EqualRc::new(collection)
        });

        topo::call!(
            {
                let values = node.computed_values().get().unwrap();
                match values.display {
                    DisplayType::Block(ref block) => {
                        block::layout_block(&**node, &values, block, *size)
                    }
                    DisplayType::Inline(_) => inline::layout_inline(&**node, &values, *size),
                }
            },
            env! {
                EqualRc<FontCollection> => collection,
            }
        )
    }

    /// Perform a layout step based on the new DOM and content size, and
    /// return a fresh layout tree.
    pub fn layout(&mut self, node: Node<Window>, size: LogicalSize) -> EqualRc<LayoutTreeNode> {
        topo::call!(
            { self.runtime.run_once() },
            env! {
                Node<Window> => node,
                LogicalSize => size,
            }
        )
    }
}
