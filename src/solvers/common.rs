use crate::parser::model_transformer::transformer_context::DomainVariable;
use serde::Serialize;
use crate::math::math_enums::VariableType;

#[derive(Debug)]
pub enum IntegerBinarySolverError {
    InvalidDomain{
        expected: Vec<VariableType>,
        got: Vec<(String, DomainVariable)>
    },
    TooLarge { name: String, value: f64 },
    DidNotSolve,
}

impl std::fmt::Display for IntegerBinarySolverError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            IntegerBinarySolverError::InvalidDomain{expected, got} => {
                let vars = got
                    .iter()
                    .map(|(name, domain)| format!("    {}: {}", name, domain.get_type()))
                    .collect::<Vec<_>>()
                    .join("\n");
                write!(
                    f,
                    "Invalid domain, the following variables are not {}: \n{}",
                    expected.iter().map(|t| t.to_string()).collect::<Vec<_>>().join(" or "),
                    vars
                )
            }
            IntegerBinarySolverError::TooLarge { name, value } => {
                write!(f, "The value of variable {} is too large: {}", name, value)
            }
            IntegerBinarySolverError::DidNotSolve => {
                write!(f, "The problem was able to be solved")
            }
        }
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct Assignment<T: Clone + Serialize + Copy> {
    pub name: String,
    pub value: T,
}

#[derive(Debug, Clone, Serialize)]
pub struct IntegerBinaryLpSolution<T: Clone + Serialize + Copy> {
    assignment: Vec<Assignment<T>>,
    value: f64,
}

impl<T: Clone + Serialize + Copy> IntegerBinaryLpSolution<T> {
    pub fn new(assignment: Vec<Assignment<T>>, value: f64) -> Self {
        Self { assignment, value }
    }

    pub fn get_assignment(&self) -> &Vec<Assignment<T>> {
        &self.assignment
    }
    pub fn get_assignment_values(&self) -> Vec<T> {
        self.assignment.iter().map(|a| a.value).collect()
    }
    pub fn get_value(&self) -> f64 {
        self.value
    }
}
