use core::fmt;
use std::str::FromStr;

#[allow(unused_imports)]
use crate::prelude::*;
use serde::Serialize;

use crate::enum_with_variants_to_string;
use crate::parser::il::il_exp::PreExp;
use crate::parser::il::iterable_set::IterableSet;
use crate::traits::ToLatex;
use crate::utils::InputSpan;

enum_with_variants_to_string! {
    pub enum BlockScopedFunctionKind derives[Debug, Clone] with_wasm {
        Sum,
        Prod,
        Min,
        Max,
        Avg,
        All,
        Any,
        Xor,
    }
}
impl BlockScopedFunctionKind {
    /// Returns true if the function is a logic one, requiring boolean bodies.
    pub fn is_logic(&self) -> bool {
        match self {
            Self::All | Self::Any | Self::Xor => true,
            Self::Sum | Self::Prod | Self::Min | Self::Max | Self::Avg => false,
        }
    }
}
impl fmt::Display for BlockScopedFunctionKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let s = match self {
            Self::Sum => "sum".to_string(),
            Self::Prod => "prod".to_string(),
            Self::Min => "min".to_string(),
            Self::Max => "max".to_string(),
            Self::Avg => "avg".to_string(),
            Self::All => "all".to_string(),
            Self::Any => "any".to_string(),
            Self::Xor => "xor".to_string(),
        };
        f.write_str(&s)
    }
}

impl ToLatex for BlockScopedFunctionKind {
    fn to_latex(&self) -> String {
        match self {
            Self::Sum => "\\sum".to_string(),
            Self::Prod => "\\prod".to_string(),
            Self::Min => "\\min".to_string(),
            Self::Max => "\\max".to_string(),
            Self::Avg => "avg".to_string(),
            Self::All => "\\bigwedge".to_string(),
            Self::Any => "\\bigvee".to_string(),
            Self::Xor => "\\bigoplus".to_string(),
        }
    }
}

impl FromStr for BlockScopedFunctionKind {
    type Err = ();
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "sum" => Ok(Self::Sum),
            "prod" => Ok(Self::Prod),
            "min" => Ok(Self::Min),
            "max" => Ok(Self::Max),
            "avg" => Ok(Self::Avg),
            "all" | "conjunction" => Ok(Self::All),
            "any" | "disjunction" => Ok(Self::Any),
            "xor" | "exclusive_disjunction" => Ok(Self::Xor),
            _ => Err(()),
        }
    }
}

enum_with_variants_to_string! {
    pub enum BlockFunctionKind derives[Debug, Clone] with_wasm {
        Min,
        Max,
        Avg,
        Abs,
        All,
        Any,
        Xor,
    }
}

impl BlockFunctionKind {
    /// Returns true if the function is a logic one, requiring boolean arguments.
    pub fn is_logic(&self) -> bool {
        match self {
            Self::All | Self::Any | Self::Xor => true,
            Self::Min | Self::Max | Self::Avg | Self::Abs => false,
        }
    }

    /// Returns the exact number of arguments required by this function, when
    /// the function has a fixed arity.
    pub fn exact_arity(&self) -> Option<usize> {
        match self {
            Self::Abs => Some(1),
            Self::Min | Self::Max | Self::Avg | Self::All | Self::Any | Self::Xor => None,
        }
    }
}

impl ToLatex for BlockFunctionKind {
    fn to_latex(&self) -> String {
        match self {
            Self::Min => "\\min".to_string(),
            Self::Max => "\\max".to_string(),
            Self::Avg => "avg".to_string(),
            Self::Abs => "\\operatorname{abs}".to_string(),
            Self::All => "\\bigwedge".to_string(),
            Self::Any => "\\bigvee".to_string(),
            Self::Xor => "\\bigoplus".to_string(),
        }
    }
}

impl FromStr for BlockFunctionKind {
    type Err = ();
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "min" => Ok(Self::Min),
            "max" => Ok(Self::Max),
            "avg" => Ok(Self::Avg),
            "abs" => Ok(Self::Abs),
            "all" | "conjunction" => Ok(Self::All),
            "any" | "disjunction" => Ok(Self::Any),
            "xor" | "exclusive_disjunction" => Ok(Self::Xor),
            _ => Err(()),
        }
    }
}

impl fmt::Display for BlockFunctionKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let s = match self {
            Self::Min => "min".to_string(),
            Self::Max => "max".to_string(),
            Self::Avg => "avg".to_string(),
            Self::Abs => "abs".to_string(),
            Self::All => "all".to_string(),
            Self::Any => "any".to_string(),
            Self::Xor => "xor".to_string(),
        };
        f.write_str(&s)
    }
}

/// A function that operates over a set of values with iteration variables.
/// This represents functions like sum, product, etc. that iterate over a set.
#[derive(Debug, Serialize, Clone)]
pub struct BlockScopedFunction {
    /// The type of block scoped function (sum, product, etc)
    pub kind: BlockScopedFunctionKind,
    /// The iteration variables and their domains
    pub iters: Vec<IterableSet>,
    /// The expression to evaluate for each iteration
    pub exp: Box<PreExp>,
}

impl ToLatex for BlockScopedFunction {
    fn to_latex(&self) -> String {
        match self.kind {
            BlockScopedFunctionKind::Sum | BlockScopedFunctionKind::Prod => {
                let name = self.kind.to_latex();
                let iters = self
                    .iters
                    .iter()
                    .map(|i| format!("{}_{{{}}}", name, i.to_latex()))
                    .collect::<Vec<String>>()
                    .join("");
                format!("{}{}", iters, self.exp.to_latex())
            }
            _ => {
                let iters = self
                    .iters
                    .iter()
                    .map(|i| i.to_latex())
                    .collect::<Vec<String>>()
                    .join(",\\");
                format!(
                    "{}_{{{}}} \\left\\{{ {} \\right\\}}",
                    self.kind.to_latex(),
                    iters,
                    self.exp.to_latex()
                )
            }
        }
    }
}

#[cfg_attr(target_arch = "wasm32", wasm_bindgen(typescript_custom_section))]
#[allow(non_upper_case_globals)]
#[cfg(target_arch = "wasm32")]
const IBlockScopedFunction: &'static str = r#"
export type SerializedBlockScopedFunctionKind = { type: keyof typeof BlockScopedFunctionKind }
export type SerializedBlockScopedFunction = {
    kind: SerializedBlockScopedFunctionKind,
    iters: SerializedIterableSet[],
    exp: SerializedPreExp,
}
"#;

impl BlockScopedFunction {
    /// Creates a new BlockScopedFunction.
    ///
    /// # Arguments
    /// * `kind` - The type of block scoped function
    /// * `iters` - Vector of iteration variables and their domains
    /// * `exp` - The expression to evaluate for each iteration
    pub fn new(kind: BlockScopedFunctionKind, iters: Vec<IterableSet>, exp: Box<PreExp>) -> Self {
        Self { kind, iters, exp }
    }

    /// Returns the span (source location) of the function body expression
    pub fn body_span(&self) -> InputSpan {
        self.exp.span().clone()
    }
}

impl fmt::Display for BlockScopedFunction {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let name = self.kind.to_string();
        write!(
            f,
            "{}({}) {{ {} }}",
            name,
            self.iters
                .iter()
                .map(|i| i.to_string())
                .collect::<Vec<String>>()
                .join(", "),
            self.exp
        )
    }
}

/// A function that operates on a fixed set of expressions.
/// This represents functions like min, max, avg that take a set of arguments
/// and compute a single result.
#[derive(Debug, Serialize, Clone)]
pub struct BlockFunction {
    /// The type of block function (min, max, avg)
    pub kind: BlockFunctionKind,
    /// The expressions to evaluate
    pub exps: Vec<PreExp>,
}

#[cfg_attr(target_arch = "wasm32", wasm_bindgen(typescript_custom_section))]
#[allow(non_upper_case_globals)]
#[cfg(target_arch = "wasm32")]
const IBlockFunction: &'static str = r#"
export type SerializedBlockFunctionKind = { type: keyof typeof BlockFunctionKind }
export type SerializedBlockFunction = {
    kind: SerializedBlockFunctionKind,
    exps: SerializedPreExp[],
}
"#;

impl BlockFunction {
    /// Creates a new BlockFunction.
    ///
    /// # Arguments
    /// * `kind` - The type of block function
    /// * `exps` - Vector of expressions to evaluate
    pub fn new(kind: BlockFunctionKind, exps: Vec<PreExp>) -> Self {
        Self { kind, exps }
    }
}

impl ToLatex for BlockFunction {
    fn to_latex(&self) -> String {
        match self.kind {
            BlockFunctionKind::Abs => match self.exps.as_slice() {
                [exp] => format!("\\left|{}\\right|", exp.to_latex()),
                _ => format!(
                    "abs\\left\\{{{}\\right\\}}",
                    self.exps
                        .iter()
                        .map(|e| e.to_latex())
                        .collect::<Vec<String>>()
                        .join(", ")
                ),
            },
            BlockFunctionKind::Min
            | BlockFunctionKind::Max
            | BlockFunctionKind::Avg
            | BlockFunctionKind::All
            | BlockFunctionKind::Any
            | BlockFunctionKind::Xor => format!(
                "{}\\left\\{{{}\\right\\}}",
                self.kind,
                self.exps
                    .iter()
                    .map(|e| e.to_latex())
                    .collect::<Vec<String>>()
                    .join(", ")
            ),
        }
    }
}

impl fmt::Display for BlockFunction {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let name = self.kind.to_string();
        write!(
            f,
            "{} {{ {} }}",
            name,
            self.exps
                .iter()
                .map(|e| e.to_string())
                .collect::<Vec<String>>()
                .join(", ")
        )
    }
}

#[cfg(test)]
mod tests {
    use super::BlockFunctionKind;

    #[test]
    fn absolute_value_is_registered_as_a_block_function() {
        assert_eq!(
            "abs".parse::<BlockFunctionKind>().unwrap().to_string(),
            "abs"
        );
        assert!(BlockFunctionKind::kinds_to_string().contains(&"abs".to_string()));
    }
}
