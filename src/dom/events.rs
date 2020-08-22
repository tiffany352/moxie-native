use super::element::Event;

/// The element associated with this event was activated by the user.
pub struct ClickEvent;
impl Event for ClickEvent {}

/// Fired when the user requests a window to be closed (such as by
/// pressing the close button, or pressing alt+f4).
pub struct CloseRequestedEvent {}
impl Event for CloseRequestedEvent {}
