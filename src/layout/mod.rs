use crate::dom::{element::children as get_children, Node, NodeChild, Window};
use euclid::{point2, size2, Length, Point2D, SideOffsets2D, Size2D};
use font_kit::family_name::FamilyName;
use font_kit::properties::Properties;
use font_kit::source::SystemSource;
use moxie::embed::Runtime;
use moxie::*;
use skribo::{FontCollection, FontFamily, LayoutSession, TextStyle};
use std::ptr;
use std::rc::Rc;

pub struct LogicalPixel;
pub type LogicalPoint = Point2D<f32, LogicalPixel>;
pub type LogicalSize = Size2D<f32, LogicalPixel>;
pub type LogicalLength = Length<f32, LogicalPixel>;
pub type LogicalSideOffsets = SideOffsets2D<f32, LogicalPixel>;

#[derive(PartialEq)]
pub enum LayoutType {
    List,
    Inline,
}

#[derive(PartialEq)]
pub struct LayoutOptions {
    pub padding: LogicalSideOffsets,
    pub width: Option<LogicalLength>,
    pub height: Option<LogicalLength>,
    pub text: Option<String>,
    pub layout_ty: LayoutType,
}

impl Default for LayoutOptions {
    fn default() -> Self {
        LayoutOptions {
            padding: LogicalSideOffsets::new_all_same(0.0f32),
            width: None,
            height: None,
            text: None,
            layout_ty: LayoutType::List,
        }
    }
}

pub struct LayoutChild {
    pub position: LogicalPoint,
    pub layout: Rc<LayoutTreeNode>,
}

pub struct LayoutTreeNode {
    pub size: LogicalSize,
    pub children: Vec<LayoutChild>,
}

struct LayoutInputs {
    opts: LayoutOptions,
    max_size: LogicalSize,
    children: Vec<Rc<LayoutTreeNode>>,
}

impl PartialEq for LayoutInputs {
    fn eq(&self, other: &LayoutInputs) -> bool {
        if self.opts != other.opts {
            return false;
        }
        if self.max_size != other.max_size {
            return false;
        }
        if self.children.len() != other.children.len() {
            return false;
        }
        for (a, b) in self.children.iter().zip(other.children.iter()) {
            if !ptr::eq(a, b) {
                return false;
            }
        }
        true
    }
}

pub struct LayoutEngine {
    runtime: Runtime<fn() -> Rc<LayoutTreeNode>, Rc<LayoutTreeNode>>,
}

impl LayoutEngine {
    pub fn new() -> LayoutEngine {
        LayoutEngine {
            runtime: Runtime::new(LayoutEngine::run_layout),
        }
    }

    fn calc_max_size(opts: &LayoutOptions, parent_size: LogicalSize) -> LogicalSize {
        let mut outer = parent_size;
        if let Some(width) = opts.width {
            outer.width = width.get();
        }
        if let Some(height) = opts.height {
            outer.height = height.get();
        }
        outer - size2(opts.padding.horizontal(), opts.padding.vertical())
    }

    #[topo::from_env(collection: &Rc<FontCollection>)]
    fn calc_layout(input: &LayoutInputs) -> Rc<LayoutTreeNode> {
        let opts = &input.opts;
        let children = &input.children;
        let max_size = input.max_size;

        let mut child_positions = vec![];
        child_positions.reserve(children.len());
        let mut min_size = match opts.layout_ty {
            LayoutType::List => {
                let mut width = 0.0f32;
                let mut height = 0.0f32;
                for child in children {
                    let size = child.size;
                    width = width.max(size.width);
                    let size = child.size;
                    child_positions.push(LayoutChild {
                        position: point2(opts.padding.left, height + opts.padding.top),
                        layout: child.clone(),
                    });
                    height += size.height;
                }
                size2(width, height)
            }
            LayoutType::Inline => {
                let max_size = max_size - size2(opts.padding.horizontal(), opts.padding.vertical());
                let mut x = 0.0f32;
                let mut height = 0.0f32;
                let mut line_height = 0.0f32;
                let mut longest_line = 0.0f32;
                for child in children {
                    let size = child.size;
                    if x + size.width > max_size.width {
                        height += line_height;
                        longest_line = longest_line.max(x);
                        x = 0.0;
                        line_height = 0.0;
                    }
                    child_positions.push(LayoutChild {
                        position: point2(opts.padding.left + x, opts.padding.top + height),
                        layout: child.clone(),
                    });
                    x += size.width;
                    line_height = line_height.max(size.height);
                }
                size2(longest_line.max(x), height.max(line_height))
            }
        };

        if let Some(ref text) = opts.text {
            let mut session = LayoutSession::create(text, &TextStyle { size: 32.0 }, collection);

            let mut width = 0.0;
            for run in session.iter_substr(0..text.len()) {
                let font = run.font();
                let metrics = font.font.metrics();
                let units_per_px = metrics.units_per_em / 32;
                for glyph in run.glyphs() {
                    width = glyph.offset.x
                        + font.font.advance(glyph.glyph_id).unwrap().x / units_per_px as f32;
                }
            }

            min_size = size2(width, 32.0);
        }

        let mut outer = min_size + size2(opts.padding.horizontal(), opts.padding.vertical());
        if let Some(width) = opts.width {
            outer.width = width.get();
        }
        if let Some(height) = opts.height {
            outer.height = height.get();
        }
        Rc::new(LayoutTreeNode {
            size: outer,
            children: child_positions,
        })
    }

    fn layout_child(node: &dyn NodeChild, parent_max_size: LogicalSize) -> Rc<LayoutTreeNode> {
        topo::call!({
            let opts = node.create_layout_opts();

            let max_size = Self::calc_max_size(&opts, parent_max_size);
            let mut children = vec![];
            for child in get_children(node) {
                children.push(Self::layout_child(child, max_size));
            }

            moxie::memo!(
                LayoutInputs {
                    children,
                    opts,
                    max_size
                },
                Self::calc_layout
            )
        })
    }

    #[topo::from_env(node: &Node<Window>, size: &LogicalSize)]
    fn run_layout() -> Rc<LayoutTreeNode> {
        let collection = once!(|| {
            let mut collection = FontCollection::new();
            let source = SystemSource::new();
            let font = source
                .select_best_match(&[FamilyName::SansSerif], &Properties::new())
                .unwrap()
                .load()
                .unwrap();
            collection.add_family(FontFamily::new_from_font(font));

            Rc::new(collection)
        });

        topo::call!(
            {
                let mut child_nodes = vec![];

                for child in node.children() {
                    child_nodes.push(LayoutChild {
                        position: point2(0.0, 0.0),
                        layout: Self::layout_child(child, *size),
                    });
                }

                Rc::new(LayoutTreeNode {
                    size: *size,
                    children: child_nodes,
                })
            },
            env! {
                Rc<FontCollection> => collection,
            }
        )
    }

    pub fn layout(&mut self, node: Node<Window>, size: LogicalSize) -> Rc<LayoutTreeNode> {
        topo::call!(
            { self.runtime.run_once() },
            env! {
                Node<Window> => node,
                LogicalSize => size,
            }
        )
    }
}
