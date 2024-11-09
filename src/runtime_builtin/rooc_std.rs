use crate::functions::ZipArrays;
use crate::parser::il::PreExp;
use crate::runtime_builtin::functions::NumericRange;
use crate::runtime_builtin::functions::{
    EdgesOfGraphFn, NeighbourOfNodeFn, NeighboursOfNodeInGraphFn, NodesOfGraphFn,
};
use crate::runtime_builtin::functions::{EnumerateArray, LenOfIterableFn};
use crate::runtime_builtin::functions::{FunctionCall, RoocFunction};
use crate::traits::ToLatex;
use crate::{Constant, Primitive};
use indexmap::IndexMap;

use super::{ArrayDifference, ArrayIntersection, ArrayUnion};

pub fn make_std() -> IndexMap<String, Box<dyn RoocFunction>> {
    let mut m: IndexMap<String, Box<dyn RoocFunction>> = IndexMap::new();
    m.insert("edges".to_string(), Box::new(EdgesOfGraphFn {}));
    m.insert("len".to_string(), Box::new(LenOfIterableFn {}));
    m.insert("nodes".to_string(), Box::new(NodesOfGraphFn {}));
    m.insert("neigh_edges".to_string(), Box::new(NeighbourOfNodeFn {}));
    m.insert(
        "neigh_edges_of".to_string(),
        Box::new(NeighboursOfNodeInGraphFn {}),
    );
    m.insert("enumerate".to_string(), Box::new(EnumerateArray {}));
    m.insert("range".to_string(), Box::new(NumericRange {}));
    m.insert("zip".to_string(), Box::new(ZipArrays {}));
    m.insert("difference".to_string(), Box::new(ArrayDifference {}));
    m.insert("union".to_string(), Box::new(ArrayUnion {}));
    m.insert("intersection".to_string(), Box::new(ArrayIntersection {}));
    m
}

pub fn make_std_constants() -> Vec<Constant> {
    vec![
        Constant::from_primitive("Infinity", Primitive::Number(f64::INFINITY)),
        Constant::from_primitive("MinusInfinity", Primitive::Number(f64::NEG_INFINITY)),
        Constant::from_primitive("PI", Primitive::Number(std::f64::consts::PI)),
    ]
}

pub fn std_fn_to_latex(fun: &FunctionCall) -> Option<String> {
    match fun.name.as_str() {
        "range" => {
            if let [ref from, ref to, known_inclusive] = &fun.args[..] {
                let known_inclusive = match known_inclusive {
                    PreExp::Primitive(p) => p.as_boolean().ok(),
                    _ => None,
                };
                if let Some(inclusive) = known_inclusive {
                    let range = if inclusive {
                        "\\dots\\text{=}"
                    } else {
                        "\\dots"
                    };
                    let from = if from.is_leaf() {
                        from.to_latex()
                    } else {
                        format!("({})", from.to_latex())
                    };
                    let to = if to.is_leaf() {
                        to.to_latex()
                    } else {
                        format!("({})", to.to_latex())
                    };
                    Some(format!("\\left\\{{{},{},{}\\right\\}}", from, range, to))
                } else {
                    None
                }
            } else {
                None
            }
        }
        "difference" => {
            if let [ref first, ref second] = &fun.args[..] {
                Some(format!(
                    "\\left\\{{{}\\setminus {}\\right\\}}",
                    first.to_latex(),
                    second.to_latex()
                ))
            } else {
                None
            }
        }
        "union" => {
            if let [ref first, ref second] = &fun.args[..] {
                Some(format!(
                    "\\left\\{{{}\\cup {}\\right\\}}",
                    first.to_latex(),
                    second.to_latex()
                ))
            } else {
                None
            }
        }
        "intersection" => {
            if let [ref first, ref second] = &fun.args[..] {
                Some(format!(
                    "\\left\\{{{}\\cap {}\\right\\}}",
                    first.to_latex(),
                    second.to_latex()
                ))
            } else {
                None
            }
        }
        _ => None,
    }
}

pub fn std_fn_to_string(fun: &FunctionCall) -> Option<String> {
    match fun.name.as_str() {
        "range" => {
            if let [ref from, ref to, known_inclusive] = &fun.args[..] {
                let known_inclusive = match known_inclusive {
                    PreExp::Primitive(p) => p.as_boolean().ok(),
                    _ => None,
                };
                if let Some(inclusive) = known_inclusive {
                    let range = if inclusive { "..=" } else { ".." };
                    let from = if from.is_leaf() {
                        from.to_string()
                    } else {
                        format!("({})", from)
                    };
                    let to = if to.is_leaf() {
                        to.to_string()
                    } else {
                        format!("({})", to)
                    };
                    Some(format!("{}{}{}", from, range, to))
                } else {
                    None
                }
            } else {
                None
            }
        }
        _ => None,
    }
}
