use crate::dom::element::{Element, HasEvent};
use crate::dom::input::{InputEvent, State};
use crate::dom::{AttrClassName, AttrStyles, ClickEvent, Node, Span, View};
use crate::style::Style;
use crate::util::event_handler::EventHandler;
use std::borrow::Cow;

/// Corresponds to <button>. This element can be hovered and pressed,
/// resulting in corresponding events.
#[derive(Default, Clone, PartialEq)]
pub struct Button {
    styles: Cow<'static, [&'static Style]>,
    class_name: Option<Cow<'static, str>>,
}

multiple_children! {
    enum ButtonChild {
        Button(Node<Button>),
        View(Node<View>),
        Span(Node<Span>),
    }
}

element_attributes! {
    Button {
        styles: AttrStyles,
        class_name: AttrClassName,
    }
}

element_handlers! {
    ButtonHandlers for Button {
        on_click: ClickEvent,
    }
}

impl Element for Button {
    type Child = ButtonChild;
    type Handlers = ButtonHandlers;

    fn process(&self, handlers: &mut Self::Handlers, event: &InputEvent) -> bool {
        match event {
            InputEvent::MouseLeft {
                state: State::Begin,
                ..
            } => {
                handlers.on_click.invoke(&ClickEvent);
                true
            }
            _ => false,
        }
    }

    fn class_name(&self) -> Option<&str> {
        self.class_name.as_ref().map(|cow| cow.as_ref())
    }

    fn styles(&self) -> &[&'static Style] {
        self.styles.as_ref()
    }
}
