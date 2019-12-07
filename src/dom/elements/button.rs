use crate::dom::element::{Element, HandlerList, HasEvent};
use crate::dom::input::{ElementState, InputEvent, State};
use crate::dom::{AttrStyle, ClickEvent, Node, Span, View};
use crate::style::Style;
use crate::util::event_handler::EventHandler;
use enumset::EnumSet;

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

pub enum Message {
    Click,
}

impl HandlerList for ButtonHandlers {
    type Message = Message;

    fn handle_message(&self, message: Message) {
        match message {
            Message::Click => {
                self.on_click.invoke(&ClickEvent);
            }
        }
    }
}

impl Element for Button {
    type Child = ButtonChild;
    type Handlers = ButtonHandlers;

    const ELEMENT_NAME: &'static str = "button";

    fn interactive(&self) -> bool {
        true
    }

    fn process(
        &self,
        states: EnumSet<ElementState>,
        event: &InputEvent,
    ) -> (EnumSet<ElementState>, Option<Message>) {
        match event {
            InputEvent::MouseMove { .. } => (states | ElementState::Hovered, None),
            InputEvent::MouseLeft {
                state: State::Begin,
                ..
            } => (states | ElementState::Pressed, None),
            InputEvent::MouseLeft {
                state: State::End, ..
            } if states.contains(ElementState::Pressed) => (
                states.difference(ElementState::Pressed.into()),
                Some(Message::Click),
            ),
            _ => (states, None),
        }
    }

    fn style(&self) -> Option<Style> {
        self.style
    }
}
