use crate::dom::input::InputEvent;
use crate::dom::{Node, Window};
use crate::layout::{LayoutEngine, LayoutTreeNode, LogicalPoint, LogicalSize, RenderData};
use crate::style::StyleEngine;
use crate::util::equal_rc::EqualRc;
use euclid::{point2, Rect};

pub struct Document {
    window: Node<Window>,
    content_size: LogicalSize,
    layout_engine: LayoutEngine,
    style_engine: StyleEngine,
}

impl Document {
    pub fn new(window: Node<Window>, content_size: LogicalSize) -> Document {
        Document {
            window,
            content_size,
            layout_engine: LayoutEngine::new(),
            style_engine: StyleEngine::new(),
        }
    }

    pub fn set_root(&mut self, window: Node<Window>) {
        self.window = window;
    }

    pub fn set_size(&mut self, size: LogicalSize) {
        self.content_size = size;
    }

    pub fn get_layout(&mut self) -> EqualRc<LayoutTreeNode> {
        self.style_engine
            .update(self.window.clone(), self.content_size);
        self.layout_engine
            .layout(self.window.clone(), self.content_size)
    }

    pub fn process_child(
        &self,
        event: &InputEvent,
        position: LogicalPoint,
        layout: &EqualRc<LayoutTreeNode>,
    ) -> bool {
        let rect = Rect::new(position, layout.size);

        if let RenderData::Node(ref node) = layout.render {
            for layout in &layout.children {
                if self.process_child(
                    event,
                    position + layout.position.to_vector(),
                    &layout.layout,
                ) {
                    return true;
                }
            }

            let do_process = match event.get_position() {
                Some((x, y)) => rect.contains(point2(x, y)),
                None => true,
            };

            if do_process {
                if node.process(event) {
                    return true;
                }
            }
        }

        false
    }

    pub fn process(&mut self, event: &InputEvent) -> bool {
        let root_layout = self.get_layout();

        for layout in &root_layout.children {
            if self.process_child(event, layout.position, &layout.layout) {
                return true;
            }
        }

        false
    }
}
