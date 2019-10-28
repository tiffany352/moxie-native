pub trait AttributeType {
    fn is_key(&self, key: &str) -> bool;
    fn get_key(&self) -> &str;
    fn get_value(&self) -> String;
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

    fn get_key(&self) -> &str {
        &*self.key
    }

    fn get_value(&self) -> String {
        self.value.clone()
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

    pub fn get_key(&self) -> &str {
        match self {
            Attribute::Unknown(attr) => attr.get_key(),
        }
    }

    pub fn get_value(&self) -> String {
        match self {
            Attribute::Unknown(attr) => attr.get_value(),
        }
    }
}
