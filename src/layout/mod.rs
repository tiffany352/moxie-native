use crate::dom::{view::ViewChild, Element, Node, Span, View, Window};
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
pub struct LayoutOptions {
    pub padding: LogicalSideOffsets,
    pub width: Option<LogicalLength>,
    pub height: Option<LogicalLength>,
}

impl Default for LayoutOptions {
    fn default() -> Self {
        LayoutOptions {
            padding: LogicalSideOffsets::new_all_same(0.0f32),
            width: None,
            height: None,
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
    children: Vec<Rc<LayoutTreeNode>>,
}

impl PartialEq for LayoutInputs {
    fn eq(&self, other: &LayoutInputs) -> bool {
        if self.opts != other.opts {
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

    fn calc_layout(input: &LayoutInputs) -> Rc<LayoutTreeNode> {
        let opts = &input.opts;
        let children = &input.children;

        let mut width = 0.0f32;
        let mut height = 0.0f32;
        let mut child_positions = vec![];
        child_positions.reserve(children.len());
        for child in children {
            let size = child.size;
            width = width.max(size.width);
            child_positions.push(LayoutChild {
                position: point2(opts.padding.left, height + opts.padding.top),
                layout: child.clone(),
            });
            height += size.height;
        }
        let mut outer =
            size2(width, height) + size2(opts.padding.horizontal(), opts.padding.vertical());
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

    #[topo::from_env(collection: &Rc<FontCollection>)]
    fn layout_span(node: &Node<Span>, parent_max_size: LogicalSize) -> Rc<LayoutTreeNode> {
        topo::call!({
            let span = node.element();
            let opts = span.create_layout_opts();

            /*
            let max_size = Self::calc_max_size(&opts, parent_max_size);
            let inner_text = node.children().join("");
            let len = inner_text.len();
            let size = 20.0;
            let mut layout = LayoutSession::create(&inner_text, &TextStyle { size }, &**collection);

            for word in inner_text.split_whitespace() {
                let start = word.as_ptr() as usize - inner_text.as_ptr() as usize;
                let end = start + word.len();

                println!("word: {}", word);
                for run in layout.iter_substr(start..end) {
                    let font = run.font();
                    for glyph in run.glyphs() {
                        println!("offset {}", glyph.offset);
                    }
                }
            }

            let mut num_lines = 1;
            let mut x = 0.0;
            for run in layout.iter_substr(0..len) {
                let font = run.font();
                for glyph in run.glyphs() {
                    let id = glyph.glyph_id;
                    let x_advance = 20.0;
                    if x + x_advance > max_size.width {
                        num_lines += 1;
                        x = 0.0;
                    }
                    x += x_advance;
                }
            }

            let min_size = size2(max_size.width, num_lines as f32 * size + 100.0);*/

            let min_size = size2(300.0, 200.0);

            moxie::memo!(min_size, |min_size| {
                Rc::new(LayoutTreeNode {
                    size: *min_size,
                    children: vec![],
                })
            })
        })
    }

    fn layout_view(node: &Node<View>, parent_max_size: LogicalSize) -> Rc<LayoutTreeNode> {
        topo::call!({
            let view = node.element();
            let opts = view.create_layout_opts();

            let max_size = Self::calc_max_size(&opts, parent_max_size);
            let mut children = vec![];
            for child in node.children() {
                match child {
                    ViewChild::View(view) => children.push(Self::layout_view(view, max_size)),
                    ViewChild::Span(span) => children.push(Self::layout_span(span, max_size)),
                }
            }

            moxie::memo!(LayoutInputs { children, opts }, Self::calc_layout)
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
                        layout: Self::layout_view(child, *size),
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

    pub fn layout(&mut self, node: &Node<Window>, size: LogicalSize) -> Rc<LayoutTreeNode> {
        topo::call!(
            { self.runtime.run_once() },
            env! {
                Node<Window> => node.clone(),
                LogicalSize => size,
            }
        )
    }
}
