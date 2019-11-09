mod attributes;
mod elements;

pub use attributes::*;
pub use elements::*;
use std::cell::RefCell;

struct ElementData {
    parent: Option<Node>,
    children: Vec<Node>,
    element: Element,
    attributes: Vec<Attribute>,
}

struct TextData {
    parent: Option<Node>,
    text: String,
}

enum NodeData {
    Element(ElementData),
    Text(TextData),
}

pub struct NodeStorage {
    nodes: RefCell<Vec<NodeData>>,
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub struct Node(u32);

impl NodeStorage {
    pub fn new() -> NodeStorage {
        NodeStorage {
            nodes: RefCell::new(vec![NodeData::Element(ElementData {
                parent: None,
                children: vec![],
                element: Element::Root,
                attributes: vec![],
            })]),
        }
    }

    pub fn root() -> Node {
        Node(0)
    }

    pub fn create_text_node(&self, text: impl Into<String>) -> Node {
        let mut nodes = self.nodes.borrow_mut();
        let node = Node(nodes.len() as u32);
        nodes.push(NodeData::Text(TextData {
            parent: None,
            text: text.into(),
        }));
        node
    }

    pub fn create_element(&self, ty: &str) -> Node {
        let mut nodes = self.nodes.borrow_mut();
        let node = Node(nodes.len() as u32);
        nodes.push(NodeData::Element(ElementData {
            parent: None,
            children: vec![],
            element: Element::from_str(ty),
            attributes: vec![],
        }));
        node
    }

    pub fn set_attribute(&self, node: Node, key: &str, value: &str) {
        let mut nodes = self.nodes.borrow_mut();
        let node = nodes.get_mut(node.0 as usize).unwrap();
        if let NodeData::Element(element) = node {
            element.attributes.retain(|attr| attr.is_key(key));
            element.attributes.push(Attribute::from_str(key, value));
        }
    }

    pub fn remove_attribute(&self, node: Node, key: &str) {
        let mut nodes = self.nodes.borrow_mut();
        let node = nodes.get_mut(node.0 as usize).unwrap();
        if let NodeData::Element(element) = node {
            element.attributes.retain(|attr| attr.is_key(key));
        }
    }

    pub fn clear_children(&self, node: Node) {
        let mut nodes = self.nodes.borrow_mut();
        let node = nodes.get_mut(node.0 as usize).unwrap();
        if let NodeData::Element(element) = node {
            element.children.clear();
        }
    }

    pub fn add_child(&self, parent: Node, child: Node) {
        let mut nodes = self.nodes.borrow_mut();
        let parent = nodes.get_mut(parent.0 as usize).unwrap();
        if let NodeData::Element(element) = parent {
            element.children.push(child);
        }
    }

    pub fn pretty_print_xml(&self, node: Node) -> String {
        use std::fmt::Write;
        let mut nodes = self.nodes.borrow();
        let node = nodes.get(node.0 as usize).unwrap();
        let mut out = String::new();
        match node {
            NodeData::Element(element) => {
                if element.attributes.len() > 0 {
                    write!(out, "<{}", element.element).unwrap();
                    for attribute in element.attributes.iter() {
                        write!(
                            out,
                            " {}=\"{}\"",
                            attribute.get_key(),
                            attribute.get_value()
                        ).unwrap();
                    }
                    writeln!(out, ">").unwrap();
                } else {
                    writeln!(out, "<{}>", element.element).unwrap();
                }
                for child in element.children.iter() {
                    for line in self.pretty_print_xml(*child).lines() {
                        writeln!(out, "  {}", line).unwrap();
                    }
                }
                write!(out, "</{}>", element.element).unwrap();
            }
            NodeData::Text(text) => {
                write!(out, "\"{}\"", text.text).unwrap();
            }
        }
        out
    }
}
