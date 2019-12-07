use super::node::LocalNodeStorage;
use super::style_engine::StyleEngine;
use crate::dom::{input::ElementState, input::InputEvent, node::NodeRef, Node, Window};
use crate::layout::{LayoutEngine, LayoutTreeNode, LogicalPoint, LogicalSize, RenderData};
use crate::runtime::{Message, RuntimeWaker};
use crate::style::ComputedValues;
use crate::util::equal_rc::EqualRc;
use enumset::EnumSet;
use euclid::{point2, Rect};
use std::collections::HashMap;
use std::rc::Rc;
use std::sync::Arc;

pub struct Document {
    window: Node<Window>,
    content_size: LogicalSize,
    node_states: HashMap<usize, EnumSet<ElementState>>,
    style_engine: StyleEngine,
    layout_engine: LayoutEngine,
    local_nodes: Rc<LocalNodeStorage>,
    waker: Arc<RuntimeWaker>,
}

impl Document {
    pub fn new(
        window: Node<Window>,
        content_size: LogicalSize,
        waker: Arc<RuntimeWaker>,
    ) -> Document {
        Document {
            window,
            content_size,
            waker,
            node_states: HashMap::new(),
            style_engine: StyleEngine::new(),
            layout_engine: LayoutEngine::new(),
            local_nodes: Rc::new(LocalNodeStorage::new()),
        }
    }

    pub fn set_window(&mut self, window: Node<Window>) {
        self.window = window;
    }

    pub fn set_size(&mut self, size: LogicalSize) {
        self.content_size = size;
    }

    pub fn process_child(
        &mut self,
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

            if node.element().interactive() {
                let do_process = match event.get_position() {
                    Some((x, y)) => rect.contains(point2(x, y)),
                    None => true,
                };

                if do_process {
                    let states = self.node_states.entry(node.id()).or_default();
                    let (new_states, message) = node.element().process(*states, event);
                    if *states != new_states {
                        *states = new_states;
                    }
                    if let Some(message) = message {
                        self.waker.send_message(Message::InvokeHandler {
                            node_id: node.id(),
                            payload: message,
                        });
                    }
                    return true;
                }
            }
        }

        false
    }

    pub fn process(&mut self, event: &InputEvent) {
        let layout = self.get_layout();
        self.process_child(event, point2(0.0, 0.0), &layout);
    }

    pub fn get_layout(&mut self) -> EqualRc<LayoutTreeNode> {
        self.style_engine.update(
            self.window.clone(),
            self.content_size,
            self.local_nodes.clone(),
        );
        self.layout_engine.layout(
            self.window.clone(),
            self.content_size,
            self.local_nodes.clone(),
        )
    }

    pub fn get_values(&mut self, node: NodeRef) -> ComputedValues {
        self.local_nodes.values(node.id())
    }
}
