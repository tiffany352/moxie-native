use crate::dom::element::{Element, ElementStates, HasEvent};
use crate::dom::events::CloseRequestedEvent;
use crate::dom::input::InputEvent;
use crate::dom::{AttrStyle, AttrTitle, Node, View};
use crate::style::Style;
use crate::util::event_handler::EventHandler;
use crate::Runtime;

/// Corresponds to <window>. This is the top-level container for UI and
/// corresponds to an OS window.
#[derive(Clone, Debug, PartialEq)]
pub struct Window {
    style: Option<Style>,
    pub title: String,
}

impl Default for Window {
    fn default() -> Self {
        Window {
            style: None,
            title: "Untitled Window".to_owned(),
        }
    }
}

element_attributes! {
    Window {
        style: AttrStyle,
        title: AttrTitle,
    }
}

element_handlers! {
    WindowHandlers for Window {
        /// Handle the window closing. If an on_close handler isn't
        /// specified, the default behavior is to call
        /// `Runtime::shutdown()` which will stop the event loop and
        /// cause the application to exit.
        on_close: CloseRequestedEvent,
    }
}

impl Element for Window {
    type Child = Node<View>;
    type Handlers = WindowHandlers;

    const ELEMENT_NAME: &'static str = "window";

    fn style(&self) -> Option<Style> {
        self.style
    }

    fn attributes(&self) -> Vec<(&'static str, String)> {
        vec![("title", format!("{:?}", self.title))]
    }

    fn process(
        &self,
        states: ElementStates,
        handlers: &mut Self::Handlers,
        event: &InputEvent,
    ) -> (bool, ElementStates) {
        match event {
            InputEvent::CloseRequested => {
                if handlers.on_close.present() {
                    handlers.on_close.invoke(&CloseRequestedEvent {});
                } else {
                    Runtime::shutdown()
                }
                (true, states)
            }
            _ => (false, states),
        }
    }
}
