use core::fmt;

use super::primitive::Primitive;

#[derive(Debug)]
pub struct Constant {
    pub name: String,
    pub value: Primitive,
}
impl Constant {
    pub fn new(name: String, value: Primitive) -> Self {
        Self { name, value }
    }
}

impl fmt::Display for Constant {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{} = {}", self.name, self.value)
    }
}