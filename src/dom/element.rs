use std::borrow::Cow;

pub trait Element: Default + Clone + PartialEq + 'static {
    type Child: Clone + PartialEq + 'static;

    fn set_attribute(&mut self, key: &str, value: Option<Cow<'static, str>>);
}
