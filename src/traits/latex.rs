use std::fmt::Debug;

pub trait ToLatex: Debug {
    fn to_latex(&self) -> String;
}

impl ToLatex for String {
    fn to_latex(&self) -> String {
        format!("\\text{{\"{}\"}}", escape_latex(self))
    }
}

impl ToLatex for f64 {
    fn to_latex(&self) -> String {
        format!("{}", self)
    }
}

impl ToLatex for i64 {
    fn to_latex(&self) -> String {
        format!("{}", self)
    }
}

impl ToLatex for u64 {
    fn to_latex(&self) -> String {
        format!("{}", self)
    }
}

impl ToLatex for bool {
    fn to_latex(&self) -> String {
        format!("{}", self)
    }
}

pub fn escape_latex(string: &str) -> String {
    let mut result = String::new();
    for c in string.chars() {
        match c {
            '_' => result.push_str("\\_"),
            '{' => result.push_str("\\{"),
            '}' => result.push_str("\\}"),
            '&' => result.push_str("\\&"),
            '%' => result.push_str("\\%"),
            '$' => result.push_str("\\$"),
            '#' => result.push_str("\\#"),
            '^' => result.push_str("\\^"),
            '~' => result.push_str("\\~"),
            '\\' => result.push_str("\\textbackslash{}"),
            _ => result.push(c),
        }
    }
    result
}
