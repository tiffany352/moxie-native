use crate::dom::element::DynamicNode;
use crate::dom::input::{InputEvent, State};
use crate::dom::node::{NodeRef, PersistentRef};
use crate::dom::{Node, Window};
use crate::layout::{LayoutEngine, LayoutTreeNode, LogicalSize};
use crate::style::StyleEngine;
use crate::util::equal_rc::EqualRc;
use crate::util::outer_join::{outer_join_filter, Joined};
use std::collections::HashMap;

pub struct Document {
    window: Node<Window>,
    content_size: LogicalSize,
    layout_engine: LayoutEngine,
    style_engine: StyleEngine,
    hovered_node: Option<PersistentRef>,
    pressed_node: Option<PersistentRef>,
    nodes_by_id: HashMap<u64, PersistentRef>,
}

impl Document {
    pub fn new(window: Node<Window>, content_size: LogicalSize) -> Document {
        let mut nodes_by_id = HashMap::new();
        Self::add_node((&window).into(), &mut nodes_by_id);
        Document {
            window,
            content_size,
            nodes_by_id,
            layout_engine: LayoutEngine::new(),
            style_engine: StyleEngine::new(),
            hovered_node: None,
            pressed_node: None,
        }
    }

    fn add_node(new: NodeRef, map: &mut HashMap<u64, PersistentRef>) {
        map.insert(new.id(), new.persistent());

        for child in new.children() {
            if let DynamicNode::Node(child) = child {
                Self::add_node(child, map);
            }
        }
    }

    fn remove_node(old: NodeRef, map: &mut HashMap<u64, PersistentRef>) {
        map.remove(&old.id());

        for child in old.children() {
            if let DynamicNode::Node(child) = child {
                Self::remove_node(child, map);
            }
        }
    }

    fn walk_live(new: NodeRef, old: NodeRef, map: &mut HashMap<u64, PersistentRef>) {
        if new != old {
            if new.id() != old.id() {
                map.remove(&old.id());
                map.insert(new.id(), new.persistent());
            }

            let new_children = new.children().map(|child| child.node());
            let old_children = old.children().map(|child| child.node());

            for item in outer_join_filter(new_children, old_children) {
                match item {
                    Joined::Both(new, old) => Self::walk_live(new, old, map),
                    Joined::Left(new) => Self::add_node(new, map),
                    Joined::Right(old) => Self::remove_node(old, map),
                }
            }
        }
    }

    pub fn set_root(&mut self, window: Node<Window>) {
        let old = std::mem::replace(&mut self.window, window);

        Self::walk_live((&self.window).into(), (&old).into(), &mut self.nodes_by_id);
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

    pub fn mouse_move(&mut self, hovered: Option<u64>) -> bool {
        if self.pressed_node.is_some() {
            return false;
        }

        let new_hovered = hovered.and_then(|hovered| self.nodes_by_id.get(&hovered));

        if new_hovered != self.hovered_node.as_ref() {
            if let Some(ref hovered) = self.hovered_node {
                if let Some(owner) = hovered.owner() {
                    owner.process(&InputEvent::Hovered { state: State::End });
                }
            }

            self.hovered_node = new_hovered.cloned();

            if let Some(ref hovered) = self.hovered_node {
                if let Some(owner) = hovered.owner() {
                    owner.process(&InputEvent::Hovered {
                        state: State::Begin,
                    });
                }
            }

            true
        } else {
            false
        }
    }

    pub fn mouse_button1(&mut self, pressed: bool) -> bool {
        if let Some(ref node) = self.pressed_node {
            if let Some(owner) = node.owner() {
                if !pressed {
                    self.pressed_node = None;
                }
                return owner.process(&InputEvent::MouseLeft {
                    state: if pressed { State::Begin } else { State::End },
                });
            }
        }

        if let Some(ref hovered) = self.hovered_node {
            if let Some(owner) = hovered.owner() {
                if pressed {
                    self.pressed_node = Some(hovered.clone());
                }
                return owner.process(&InputEvent::MouseLeft {
                    state: if pressed { State::Begin } else { State::End },
                });
            }
        }

        false
    }
}
