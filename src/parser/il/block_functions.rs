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
            _ => Err(()),
        }
    }
}

enum_with_variants_to_string! {
    pub enum BlockFunctionKind derives[Debug, Clone] with_wasm {
        Min,
        Max,
        Avg,
    }
}

impl ToLatex for BlockFunctionKind {
    fn to_latex(&self) -> String {
        match self {
            Self::Min => "\\min".to_string(),
            Self::Max => "\\max".to_string(),
            Self::Avg => "avg".to_string(),
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
        };
        f.write_str(&s)
    }
}

#[derive(Debug, Serialize, Clone)]
pub struct BlockScopedFunction {
    pub kind: BlockScopedFunctionKind,
    pub iters: Vec<IterableSet>,
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
export type SerializedBlockScopedFunction = {
    kind: BlockScopedFunctionKind,
    iters: SerializedIterableSet[],
    exp: SerializedPreExp,
}
"#;

impl BlockScopedFunction {
    pub fn new(kind: BlockScopedFunctionKind, iters: Vec<IterableSet>, exp: Box<PreExp>) -> Self {
        Self { kind, iters, exp }
    }
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

#[derive(Debug, Serialize, Clone)]
pub struct BlockFunction {
    pub kind: BlockFunctionKind,
    pub exps: Vec<PreExp>,
}

#[cfg_attr(target_arch = "wasm32", wasm_bindgen(typescript_custom_section))]
#[allow(non_upper_case_globals)]
#[cfg(target_arch = "wasm32")]
const IBlockFunction: &'static str = r#"
export type SerializedBlockFunction = {
    kind: BlockFunctionKind,
    exps: SerializedPreExp[],
}
"#;

impl BlockFunction {
    pub fn new(kind: BlockFunctionKind, exps: Vec<PreExp>) -> Self {
        Self { kind, exps }
    }
}

impl ToLatex for BlockFunction {
    fn to_latex(&self) -> String {
        let name = self.kind.to_string();
        format!(
            "{}\\left\\{{{}\\right\\}}",
            name,
            self.exps
                .iter()
                .map(|e| e.to_latex())
                .collect::<Vec<String>>()
                .join(", ")
        )
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
