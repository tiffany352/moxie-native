use std::ops::Deref;
use std::rc::Rc;

pub struct EqualRc<T>(Rc<T>);

impl<T> EqualRc<T> {
    pub fn new(value: T) -> Self {
        EqualRc(Rc::new(value))
    }
}

impl<T> From<Rc<T>> for EqualRc<T> {
    fn from(rc: Rc<T>) -> Self {
        EqualRc(rc)
    }
}

impl<T> Into<Rc<T>> for EqualRc<T> {
    fn into(self) -> Rc<T> {
        self.0
    }
}

impl<T> Deref for EqualRc<T> {
    type Target = T;

    fn deref(&self) -> &T {
        &*self.0
    }
}

impl<T> PartialEq for EqualRc<T> {
    fn eq(&self, other: &Self) -> bool {
        Rc::ptr_eq(&self.0, &other.0)
    }
}

impl<T> Clone for EqualRc<T> {
    fn clone(&self) -> Self {
        EqualRc(self.0.clone())
    }
}
