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
    pub fn to_string(&self) -> String {
        format!("{} = {}", self.name, self.value.to_string())
    }
}
