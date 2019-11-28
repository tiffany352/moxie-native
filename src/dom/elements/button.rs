use crate::dom::element::{Element, ElementStates, HasEvent};
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

#[derive(Default, Clone, Copy, PartialEq)]
pub struct ButtonStates {
    hovered: bool,
    pressed: bool,
}

impl ElementStates for ButtonStates {
    fn has_state(&self, name: &str) -> bool {
        match name {
            "hover" => self.hovered,
            "press" => self.pressed,
            _ => false,
        }
    }
}

impl Element for Button {
    type Child = ButtonChild;
    type Handlers = ButtonHandlers;
    type States = ButtonStates;

    fn process(
        &self,
        states: Self::States,
        handlers: &mut Self::Handlers,
        event: &InputEvent,
    ) -> (bool, Self::States) {
        match event {
            InputEvent::MouseMove { .. } => (
                true,
                ButtonStates {
                    hovered: true,
                    ..states
                },
            ),
            InputEvent::MouseLeft {
                state: State::Begin,
                ..
            } => (
                true,
                ButtonStates {
                    pressed: true,
                    ..states
                },
            ),
            InputEvent::MouseLeft {
                state: State::End, ..
            } if states.pressed => {
                handlers.on_click.invoke(&ClickEvent);
                (
                    true,
                    ButtonStates {
                        pressed: false,
                        ..states
                    },
                )
            }
            _ => (false, states),
        }
    }

    fn class_name(&self) -> Option<&str> {
        self.class_name.as_ref().map(|cow| cow.as_ref())
    }

    fn styles(&self) -> &[&'static Style] {
        self.styles.as_ref()
    }
}
