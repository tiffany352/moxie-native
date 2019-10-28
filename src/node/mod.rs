mod attributes;

use std::cell::RefCell;
use std::fmt;
use std::ptr;
use std::rc::{Rc, Weak};

pub use attributes::*;

#[derive(Clone)]
pub enum Element {
    Root,
    Window,
    View,
    Unknown(String),
}

impl Element {
    fn from_str(name: &str) -> Element {
        match name {
            "view" => Element::View,
            "window" => Element::Window,
            _ => Element::Unknown(name.to_owned()),
        }
    }
}

impl fmt::Display for Element {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Element::Root => write!(fmt, "root"),
            Element::Window => write!(fmt, "window"),
            Element::View => write!(fmt, "view"),
            Element::Unknown(name) => write!(fmt, "{}", name),
        }
    }
}

struct ElementData {
    parent: RefCell<Option<Weak<NodeData>>>,
    children: RefCell<Vec<Rc<NodeData>>>,
    element: Element,
    attributes: RefCell<Vec<Attribute>>,
}

struct TextData {
    parent: RefCell<Option<Weak<NodeData>>>,
    text: String,
}

enum NodeData {
    Element(ElementData),
    Text(TextData),
}

impl NodeData {
    fn get_parent(&self) -> Option<Rc<NodeData>> {
        match self {
            NodeData::Element(element) => element
                .parent
                .borrow()
                .as_ref()
                .and_then(|weak| weak.upgrade()),
            NodeData::Text(text) => text
                .parent
                .borrow()
                .as_ref()
                .and_then(|weak| weak.upgrade()),
        }
    }
}

impl fmt::Debug for NodeData {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        match self {
            NodeData::Element(element) => {
                let attributes = element.attributes.borrow();
                if attributes.len() > 0 {
                    write!(fmt, "<{}", element.element)?;
                    for attribute in attributes.iter() {
                        write!(
                            fmt,
                            "  {}=\"{}\"",
                            attribute.get_key(),
                            attribute.get_value()
                        )?;
                    }
                    writeln!(fmt, ">")?;
                } else {
                    writeln!(fmt, "<{}>", element.element)?;
                }
                let children = element.children.borrow();
                for child in children.iter() {
                    for line in format!("{:#?}", child).lines() {
                        writeln!(fmt, "  {}", line)?;
                    }
                }
                write!(fmt, "</{}>", element.element)
            }
            NodeData::Text(text) => write!(fmt, "\"{}\"", text.text),
        }
    }
}

type NodePtr = Rc<NodeData>;

#[derive(Clone)]
pub struct Node {
    ptr: NodePtr,
}

impl fmt::Debug for Node {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        self.ptr.fmt(fmt)
    }
}

impl PartialEq for Node {
    fn eq(&self, other: &Self) -> bool {
        ptr::eq(&*self.ptr, &*other.ptr)
    }
}

impl Node {
    pub fn create_text_node<T>(&self, text: T) -> Node
    where
        T: Into<String>,
    {
        let child = Node {
            ptr: Rc::new(NodeData::Text(TextData {
                parent: RefCell::new(Some(Rc::downgrade(&self.ptr))),
                text: text.into(),
            })),
        };
        self.append_child(&child);
        child
    }

    pub fn create_element(&self, ty: &str) -> Node {
        let child = Node {
            ptr: Rc::new(NodeData::Element(ElementData {
                parent: RefCell::new(Some(Rc::downgrade(&self.ptr))),
                children: RefCell::new(vec![]),
                element: Element::from_str(ty),
                attributes: RefCell::new(vec![]),
            })),
        };
        self.append_child(&child);
        child
    }

    pub fn create_root() -> Node {
        Node {
            ptr: Rc::new(NodeData::Element(ElementData {
                parent: RefCell::new(None),
                children: RefCell::new(vec![]),
                element: Element::Root,
                attributes: RefCell::new(vec![]),
            })),
        }
    }

    pub fn get_text(&self) -> Option<String> {
        if let NodeData::Text(ref text) = *self.ptr {
            return Some(text.text.clone());
        }
        None
    }

    pub fn get_element(&self) -> Option<Element> {
        if let NodeData::Element(ref element) = *self.ptr {
            return Some(element.element.clone());
        }
        None
    }

    pub fn get_attribute<ValueTy>(&self) -> Option<ValueTy>
    where
        ValueTy: AttributeType + Clone,
    {
        if let NodeData::Element(ref element) = *self.ptr {
            let attributes = element.attributes.borrow();
            for attribute in attributes.iter() {
                if let Some(value) = ValueTy::from_attribute(attribute) {
                    return Some(value.clone());
                }
            }
        }
        None
    }

    pub fn set_attribute(&self, key: &str, value: &str) {
        if let NodeData::Element(ref element) = *self.ptr {
            let mut attributes = element.attributes.borrow_mut();
            attributes.retain(|attr| attr.is_key(key));
            attributes.push(Attribute::from_str(key, value));
        }
    }

    pub fn remove_attribute(&self, key: &str) {
        if let NodeData::Element(ref element) = *self.ptr {
            let mut attributes = element.attributes.borrow_mut();
            attributes.retain(|attr| attr.is_key(key));
        }
    }

    pub fn first_child(&self) -> Option<Node> {
        match &*self.ptr {
            NodeData::Element(element) => element
                .children
                .borrow()
                .first()
                .map(|ptr| Node { ptr: ptr.clone() }),
            NodeData::Text(_) => None,
        }
    }

    pub fn last_child(&self) -> Option<Node> {
        match &*self.ptr {
            NodeData::Element(element) => element
                .children
                .borrow()
                .last()
                .map(|ptr| Node { ptr: ptr.clone() }),
            NodeData::Text(_) => None,
        }
    }

    pub fn next_sibling(&self) -> Option<Node> {
        if let Some(ref parent) = self.ptr.get_parent() {
            if let NodeData::Element(ref parent) = **parent {
                let children = parent.children.borrow();
                let mut last_was_me = false;
                for child in children.iter() {
                    if ptr::eq(&**child, &*self.ptr) {
                        last_was_me = true
                    } else if last_was_me {
                        return Some(Node { ptr: child.clone() });
                    }
                }
            }
        }
        None
    }

    pub fn replace_child(&self, new_child: &Node, existing: &Node) {
        if let NodeData::Element(ref element) = *self.ptr {
            let mut children = element.children.borrow_mut();
            for child in children.iter_mut() {
                if ptr::eq(&**child, &*existing.ptr) {
                    *child = new_child.ptr.clone();
                }
            }
        }
    }

    pub fn append_child(&self, new_child: &Node) {
        if let Some(parent) = new_child.ptr.get_parent() {
            Node { ptr: parent }.remove_child(new_child);
        }
        if let NodeData::Element(ref element) = *self.ptr {
            let mut children = element.children.borrow_mut();
            children.retain(|child| !ptr::eq(&**child, &*new_child.ptr));
            children.push(new_child.ptr.clone());
        }
    }

    pub fn remove_child(&self, to_remove: &Node) {
        if let NodeData::Element(ref element) = *self.ptr {
            let mut children = element.children.borrow_mut();
            children.retain(|child| !ptr::eq(&**child, &*to_remove.ptr))
        }
    }

    pub fn clear_children(&self) {
        if let NodeData::Element(ref element) = *self.ptr {
            element.children.replace(vec![]);
        }
    }
}
