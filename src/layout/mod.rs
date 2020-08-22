//! This module handles creating the layout tree, which includes
//! arranging elements and performing text layout.

use crate::document::DocumentState;
use crate::dom::node::AnyNode;
use crate::style::DisplayType;
use crate::util::equal_rc::EqualRc;
use euclid::{Length, Point2D, SideOffsets2D, Size2D};
use font_kit::family_name::FamilyName;
use font_kit::properties::Properties;
use font_kit::source::SystemSource;
use moxie::runtime::Runtime;
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

pub enum RenderData {
    Text { text: LayoutText, parent: AnyNode },
    Node(AnyNode),
}

/// One node in the layout tree, which corresponds n:1 with DOM nodes.
pub struct LayoutTreeNode {
    /// The computed size of the node.
    pub size: LogicalSize,
    pub margin: LogicalSideOffsets,
    pub render: RenderData,
    pub children: Vec<LayoutChild>,
}

/// Used to build the layout tree, with internal caching for
/// performance.
pub struct LayoutEngine {
    runtime: Runtime,
    collection: EqualRc<FontCollection>,
}

impl LayoutEngine {
    pub fn new() -> LayoutEngine {
        let mut collection = FontCollection::new();
        let source = SystemSource::new();
        let font = source
            .select_best_match(&[FamilyName::SansSerif], &Properties::new())
            .unwrap()
            .load()
            .unwrap();
        collection.add_family(FontFamily::new_from_font(font));

        LayoutEngine {
            runtime: Runtime::new(),
            collection: EqualRc::new(collection),
        }
    }

    /// Perform a layout step based on the new DOM and content size, and
    /// return a fresh layout tree.
    pub(crate) fn layout(&mut self, state: &mut DocumentState) -> EqualRc<LayoutTreeNode> {
        illicit::Layer::new()
            .offer(self.collection.clone())
            .enter(move || {
                self.runtime.run_once(move || {
                    let node = state.window.clone();
                    let values = *state.computed_values(node.id());
                    match values.display {
                        DisplayType::Block(ref block) => block::layout_block(
                            state,
                            (&node).into(),
                            &values,
                            block,
                            state.content_size,
                        ),
                        DisplayType::Inline(_) => inline::layout_inline(
                            state,
                            (&node).into(),
                            &values,
                            state.content_size,
                        ),
                    }
                })
            })
    }
}
