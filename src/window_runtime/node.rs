use crate::style::ComputedValues;
use std::cell::{RefCell, RefMut};
use std::collections::hash_map::HashMap;
use std::fmt;

pub struct LocalNodeStorage {
    values: RefCell<HashMap<usize, ComputedValues>>,
}

impl fmt::Debug for LocalNodeStorage {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        write!(fmt, "LocalNodeStorage {{ ... }}")
    }
}

impl LocalNodeStorage {
    pub fn new() -> Self {
        LocalNodeStorage {
            values: RefCell::new(HashMap::new()),
        }
    }

    pub fn values(&self, id: usize) -> ComputedValues {
        let mut map = self.values.borrow_mut();
        map.entry(id).or_default().clone()
    }

    pub fn values_mut(&self, id: usize) -> RefMut<ComputedValues> {
        {
            let mut map = self.values.borrow_mut();
            map.entry(id).or_default();
        }
        RefMut::map(self.values.borrow_mut(), |map| map.get_mut(&id).unwrap())
    }
}
