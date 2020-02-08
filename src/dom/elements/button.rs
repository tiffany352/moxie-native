use crate::dom::element::{Element, ElementState, ElementStates, HasEvent};
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

impl Element for Button {
    type Child = ButtonChild;
    type Handlers = ButtonHandlers;

    const ELEMENT_NAME: &'static str = "button";

    fn interactive(&self) -> bool {
        true
    }

    fn process(
        &self,
        states: ElementStates,
        handlers: &mut Self::Handlers,
        event: &InputEvent,
    ) -> (bool, ElementStates) {
        match event {
            InputEvent::Hovered {
                state: State::Begin,
            } => (true, states | ElementState::Hover),
            InputEvent::Hovered { state: State::End } => {
                (true, states.difference(ElementState::Hover.into()))
            }
            InputEvent::MouseLeft {
                state: State::Begin,
                ..
            } => (true, states | ElementState::Press),
            InputEvent::MouseLeft {
                state: State::End, ..
            } if states.contains(ElementState::Press) => {
                handlers.on_click.invoke(&ClickEvent);
                (true, states.difference(ElementState::Press.into()))
            }
            _ => (false, states),
        }
    }

    fn style(&self) -> Option<Style> {
        self.style
    }
}
