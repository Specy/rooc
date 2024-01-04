use core::fmt;

use serde::Serialize;
use wasm_bindgen::prelude::wasm_bindgen;

use super::primitive::Primitive;

#[derive(Debug, Serialize)]
pub struct Constant {
    pub name: String,
    pub value: Primitive,
}
#[wasm_bindgen(typescript_custom_section)]
const IConstant: &'static str = r#"
export type SerializedConstant = {
    name: string,
    value: SerializedPrimitive
}
"#;
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
