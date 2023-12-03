use super::graph::Graph;


//TODO should i even have this, maybe i can use the Primitive type directly
#[derive(Debug, Clone)]
pub enum ConstantValue {
    Number(f64),
    OneDimArray(Vec<f64>),
    TwoDimArray(Vec<Vec<f64>>),
    Graph(Graph),
    String(String),
}
impl ConstantValue {
    pub fn to_string(&self) -> String {
        match self {
            Self::Number(n) => n.to_string(),
            Self::OneDimArray(v) => format!("{:?}", v),
            Self::TwoDimArray(v) => {
                let result = v.iter().map(|row| format!("{:?}", row)).collect::<Vec<_>>();
                format!("[\n{}\n]", result.join(",\n"))
            }
            Self::Graph(g) => {
                format!("Graph {{\n{}\n}}", g.to_string())
            }
            Self::String(s) => format!("\"{}\"", s),
        }
    }
}

#[derive(Debug)]
pub struct Constant {
    pub name: String,
    pub value: ConstantValue,
}
impl Constant {
    pub fn new(name: String, value: ConstantValue) -> Self {
        Self { name, value }
    }
    pub fn to_string(&self) -> String {
        format!("{} = {}", self.name, self.value.to_string())
    }
}
