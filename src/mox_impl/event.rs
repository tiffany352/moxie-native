use crate::dom::events::ClickEvent;
use std::marker::PhantomData;

pub fn on_click() -> PhantomData<ClickEvent> {
    PhantomData
}
