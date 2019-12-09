use crate::dom::element::{Element, ElementStates, HasEvent};
use crate::dom::input::{InputEvent, State};
use crate::dom::{AttrStyle, ClickEvent, Node, Span, View};
use crate::style::Style;
use crate::util::event_handler::EventHandler;

/// Corresponds to <button>. This element can be hovered and pressed,
/// resulting in corresponding events.
#[derive(Default, Clone, Debug, PartialEq)]
pub struct Button {
    style: Option<Style>,
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
        style: AttrStyle,
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

    const ELEMENT_NAME: &'static str = "button";

    fn interactive(&self) -> bool {
        true
    }

    fn process(
        &self,
        states: Self::States,
        handlers: &mut Self::Handlers,
        event: &InputEvent,
    ) -> (bool, Self::States) {
        match event {
            InputEvent::Hovered {
                state: State::Begin,
            } => (
                true,
                ButtonStates {
                    hovered: true,
                    ..states
                },
            ),
            InputEvent::Hovered { state: State::End } => (
                true,
                ButtonStates {
                    hovered: false,
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

    fn style(&self) -> Option<Style> {
        self.style
    }
}
