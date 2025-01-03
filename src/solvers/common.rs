use crate::math::{Comparison, OptimizationType, VariableType};
use crate::parser::model_transformer::DomainVariable;
use copper::views::{Times, ViewExt};
use copper::{VarId, VarIdBinary};
use indexmap::IndexMap;
use num_traits::ToPrimitive;
use serde::Serialize;
#[allow(unused)]
use std::fmt::{write, Display, Formatter};

/// Represents errors that can occur during linear programming problem solving.
#[derive(Debug)]
pub enum SolverError {
    /// Variables in the problem domain have invalid types.
    /// - `expected`: List of valid variable types
    /// - `got`: List of variables with invalid types
    InvalidDomain {
        expected: Vec<VariableType>,
        got: Vec<(String, DomainVariable)>,
    },

    /// A variable's value exceeds the maximum allowed value.
    /// - `name`: Name of the variable
    /// - `value`: The value that was too large
    TooLarge { name: String, value: f64 },

    /// The solver failed to find a solution.
    DidNotSolve,

    /// The problem is unbounded (has no finite optimal solution).
    Unbounded,

    /// The problem has no feasible solution.
    Infisible,

    /// A general error with a custom message.
    Other(String),

    /// The solver reached its iteration limit before finding a solution.
    LimitReached,

    /// The optimization type is not supported by the solver.
    /// - `expected`: List of supported optimization types
    /// - `got`: The unsupported optimization type that was used
    UnimplementedOptimizationType {
        expected: Vec<OptimizationType>,
        got: OptimizationType,
    },

    /// The comparison operator is not supported by the solver.
    /// - `got`: The unsupported comparison operator
    /// - `expected`: List of supported comparison operators
    UnavailableComparison {
        got: Comparison,
        expected: Vec<Comparison>,
    },
}

impl std::fmt::Display for SolverError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SolverError::InvalidDomain { expected, got } => {
                let vars = got
                    .iter()
                    .map(|(name, domain)| format!("    {}: {}", name, domain.get_type()))
                    .collect::<Vec<_>>()
                    .join("\n");
                write!(
                    f,
                    "Invalid domain, the following variables are not {}: \n{}",
                    expected
                        .iter()
                        .map(|t| t.to_string())
                        .collect::<Vec<_>>()
                        .join(" or "),
                    vars
                )
            }
            SolverError::Unbounded => {
                write!(f, "The problem is unbounded")
            }
            SolverError::Other(s) => {
                write!(f, "{}", s)
            }
            SolverError::LimitReached => {
                write!(f, "The iteration limit was reached")
            }
            SolverError::UnavailableComparison { got, expected } => {
                write!(
                    f,
                    "The comparison \"{}\" is not available in this solver, expected one of {}",
                    got,
                    expected
                        .iter()
                        .map(|c| c.to_string())
                        .collect::<Vec<_>>()
                        .join(", ")
                )
            }
            SolverError::TooLarge { name, value } => {
                write!(f, "The value of variable {} is too large: {}", name, value)
            }
            SolverError::DidNotSolve => {
                write!(f, "The problem was unable to be solved, it might be infeasible")
            }
            SolverError::Infisible => {
                write!(f, "The problem is infeasible")
            }
            SolverError::UnimplementedOptimizationType { expected, got } => {
                write!(
                    f,
                    "Expected optimization type to be one of {:?} but got {:?}",
                    expected, got
                )
            }
        }
    }
}

/// Represents a variable assignment in a solution.
/// - `T`: The type of the variable's value
#[derive(Debug, Clone, Serialize)]
pub struct Assignment<T: Clone + Serialize + Copy + Display> {
    pub name: String,
    pub value: T,
}

impl<T: Clone + Serialize + Copy + Display> Display for Assignment<T> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} = {}", self.name, self.value)
    }
}

/// Represents a solution to a linear programming problem.
/// - `T`: The type of the variables' values
#[derive(Debug, Clone, Serialize)]
pub struct LpSolution<T: Clone + Serialize + Copy + Display> {
    assignment: Vec<Assignment<T>>,
    value: f64,
}

impl<T: Clone + Serialize + Copy + Display> Display for LpSolution<T> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "Optimal value: {}\n\n", self.value)?;
        write!(
            f,
            "Variables:\n{}",
            self.assignment
                .iter()
                .map(|a| a.to_string())
                .collect::<Vec<_>>()
                .join("\n")
        )
    }
}

impl<T: Clone + Serialize + Copy + Display> LpSolution<T> {
    /// Creates a new solution with the given assignments and objective value.
    ///
    /// # Arguments
    /// * `assignment` - Vector of variable assignments
    /// * `value` - The objective function value at this solution
    pub fn new(assignment: Vec<Assignment<T>>, value: f64) -> Self {
        Self { assignment, value }
    }

    /// Returns a reference to the vector of variable assignments.
    pub fn assignment(&self) -> &Vec<Assignment<T>> {
        &self.assignment
    }

    /// Returns a vector containing just the values of all assignments.
    pub fn assignment_values(&self) -> Vec<T> {
        self.assignment.iter().map(|a| a.value).collect()
    }

    /// Returns the objective function value of this solution.
    pub fn value(&self) -> f64 {
        self.value
    }
}

/// Finds variables in a domain that don't satisfy a validation condition.
///
/// # Arguments
/// * `domain` - Map of variable names to their domain definitions
/// * `validator` - Function that returns true if a variable type is valid
///
/// # Returns
/// Vector of (name, variable) pairs that failed validation
pub fn find_invalid_variables<F>(
    domain: &IndexMap<String, DomainVariable>,
    validator: F,
) -> Vec<(String, DomainVariable)>
where
    F: Fn(&VariableType) -> bool,
{
    domain
        .iter()
        .filter_map(|(name, var)| {
            let var_type = var.get_type();
            if !validator(var_type) {
                Some((name.clone(), var.clone()))
            } else {
                None
            }
        })
        .collect::<Vec<_>>()
}

/// Processes coefficients and variables to create Times expressions, filtering based on index.
///
/// # Arguments
/// * `coefficients` - Iterator of coefficient values
/// * `variables` - Iterator of variable IDs
/// * `filter_fn` - Function that returns true for indices to include
///
/// # Returns
/// Optional vector of Times expressions if all coefficients can be converted to i32
pub fn process_variables<'a, F>(
    coefficients: impl Iterator<Item = &'a f64>,
    variables: impl Iterator<Item = &'a VarId>,
    filter_fn: F,
) -> Option<Vec<Times<VarId>>>
where
    F: Fn(usize) -> bool,
{
    coefficients
        .enumerate()
        .filter(|(i, _c)| filter_fn(*i))
        .zip(variables)
        .map(|((_, c), v)| c.to_i32().map(|c| v.times(c)))
        .collect::<Option<Vec<_>>>()
}

/// Similar to process_variables but works with binary variables.
///
/// # Arguments
/// * `coefficients` - Iterator of coefficient values
/// * `variables` - Iterator of binary variable IDs
/// * `filter_fn` - Function that returns true for indices to include
///
/// # Returns
/// Optional vector of Times expressions if all coefficients can be converted to i32
pub fn process_variables_binary<'a, F>(
    coefficients: impl Iterator<Item = &'a f64>,
    variables: impl Iterator<Item = &'a VarIdBinary>,
    filter_fn: F,
) -> Option<Vec<Times<VarIdBinary>>>
where
    F: Fn(usize) -> bool,
{
    coefficients
        .enumerate()
        .filter(|(i, _c)| filter_fn(*i))
        .zip(variables)
        .map(|((_, c), v)| c.to_i32().map(|c| v.times(c)))
        .collect::<Option<Vec<_>>>()
}
