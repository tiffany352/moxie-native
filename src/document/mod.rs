use crate::dom::element::{DynamicNode, ElementStates};
use crate::dom::input::{InputEvent, State};
use crate::dom::node::{AnyNode, NodeRef};
use crate::dom::{Node, Window};
use crate::layout::{LayoutEngine, LayoutTreeNode, LogicalSize};
use crate::style::ComputedValues;
use crate::util::equal_rc::EqualRc;
use moxie::runtime::Runtime;
use std::collections::HashMap;

struct NodeState {
    node: AnyNode,
    states: ElementStates,
    computed_values: Option<ComputedValues>,
    live: bool,
}

pub(crate) struct DocumentState {
    states: HashMap<u64, NodeState>,
    pub window: Node<Window>,
    pub content_size: LogicalSize,
    hovered_node: Option<u64>,
    pressed_node: Option<u64>,
}

impl DocumentState {
    pub fn walk_children(&mut self, node: NodeRef) {
        let entry = self.states.entry(node.id()).or_insert_with(|| NodeState {
            node: node.to_owned(),
            states: ElementStates::default(),
            computed_values: None,
            live: false,
        });
        entry.live = true;

        for child in node.children() {
            if let DynamicNode::Node(node) = child {
                self.walk_children(node)
            }
        }
    }

    pub fn computed_values(&self, id: u64) -> &ComputedValues {
        self.states
            .get(&id)
            .unwrap()
            .computed_values
            .as_ref()
            .unwrap()
    }

    pub fn node_states(&self, id: u64) -> ElementStates {
        self.states.get(&id).unwrap().states
    }

    fn set_root(&mut self, window: Node<Window>) {
        self.window = window.clone();

        self.walk_children((&window).into());
        self.states
            .retain(|_id, state| std::mem::replace(&mut state.live, false));
    }

    pub fn set_size(&mut self, size: LogicalSize) {
        self.content_size = size;
    }

    pub fn mouse_move(&mut self, hovered: Option<u64>) -> bool {
        if self.pressed_node.is_some() {
            return false;
        }

        if hovered != self.hovered_node {
            if let Some(hovered) = self.hovered_node {
                if let Some(state) = self.states.get_mut(&hovered) {
                    state.states = state
                        .node
                        .process(state.states, &InputEvent::Hovered { state: State::End });
                }
            }

            self.hovered_node = hovered;

            if let Some(hovered) = self.hovered_node {
                if let Some(state) = self.states.get_mut(&hovered) {
                    state.states = state.node.process(
                        state.states,
                        &InputEvent::Hovered {
                            state: State::Begin,
                        },
                    );
                }
            }

            true
        } else {
            false
        }
    }

    pub fn mouse_button1(&mut self, pressed: bool) -> bool {
        if let Some(node) = self.pressed_node {
            if let Some(state) = self.states.get_mut(&node) {
                if !pressed {
                    self.pressed_node = None;
                }
                state.states = state.node.process(
                    state.states,
                    &InputEvent::MouseLeft {
                        state: if pressed { State::Begin } else { State::End },
                    },
                );
                return true;
            }
        }

        if let Some(hovered) = self.hovered_node {
            if let Some(state) = self.states.get_mut(&hovered) {
                if pressed {
                    self.pressed_node = Some(hovered);
                }
                state.states = state.node.process(
                    state.states,
                    &InputEvent::MouseLeft {
                        state: if pressed { State::Begin } else { State::End },
                    },
                );
                return true;
            }
        }

        false
    }

    pub fn close_requested(&mut self) -> bool {
        if let Some(state) = self.states.get_mut(&self.window.id()) {
            state.states = state
                .node
                .process(state.states, &InputEvent::CloseRequested);
            return true;
        }
        false
    }
}

pub struct Document {
    state: DocumentState,
    style_runtime: Runtime,
    layout_engine: LayoutEngine,
}

impl Document {
    pub fn new(window: Node<Window>, content_size: LogicalSize) -> Document {
        let mut doc = Document {
            state: DocumentState {
                window,
                content_size,
                states: HashMap::new(),
                hovered_node: None,
                pressed_node: None,
            },
            style_runtime: Runtime::new(),
            layout_engine: LayoutEngine::new(),
        };
        doc.state.walk_children((&doc.state.window.clone()).into());
        doc
    }

    pub fn computed_values(&self, id: u64) -> &ComputedValues {
        self.state.computed_values(id)
    }

    pub fn set_root(&mut self, window: Node<Window>) {
        self.state.set_root(window);
    }

    pub fn set_size(&mut self, size: LogicalSize) {
        self.state.set_size(size);
    }

    pub fn get_layout(&mut self) -> EqualRc<LayoutTreeNode> {
        let state = &mut self.state;
        let window = state.window.clone();
        let size = state.content_size;
        self.style_runtime.run_once(move || {
            illicit::Layer::new().offer(size).enter(move || {
                state.update_style((&window).into(), None);
            })
        });
        self.layout_engine.layout(&mut self.state)
    }

    pub fn mouse_move(&mut self, hovered: Option<u64>) -> bool {
        self.state.mouse_move(hovered)
    }

    pub fn mouse_button1(&mut self, pressed: bool) -> bool {
        self.state.mouse_button1(pressed)
    }

    pub fn close_requested(&mut self) -> bool {
        self.state.close_requested()
    }
}

mod styling;
