use super::element::Event;

/// The element associated with this event was activated by the user.
pub struct ClickEvent;

impl Event for ClickEvent {}

/// Fired when the text contents of a textbox changes.
pub struct TextEvent {
    pub text: String,
    pub cursor_pos: usize,
}

impl Event for TextEvent {}
