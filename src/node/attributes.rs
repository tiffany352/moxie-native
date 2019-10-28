pub trait AttributeType {
    fn is_key(&self, key: &str) -> bool;
    fn from_attribute(attr: &Attribute) -> Option<&Self>;
}

pub struct UnknownAttribute {
    pub key: String,
    pub value: String,
}

impl AttributeType for UnknownAttribute {
    fn is_key(&self, key: &str) -> bool {
        self.key == key
    }

    fn from_attribute(attr: &Attribute) -> Option<&Self> {
        match attr {
            Attribute::Unknown(ref attr) => Some(attr),
            _ => None,
        }
    }
}

pub enum Attribute {
    Unknown(UnknownAttribute),
}

impl Attribute {
    pub fn from_str(key: &str, value: &str) -> Self {
        Attribute::Unknown(UnknownAttribute {
            key: key.to_owned(),
            value: value.to_owned(),
        })
    }

    pub fn is_key(&self, key_str: &str) -> bool {
        match self {
            Attribute::Unknown(attr) => attr.is_key(key_str),
        }
    }
}
