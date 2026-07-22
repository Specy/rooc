use crate::math::{Comparison, OptimizationType, VariableType};
use crate::parser::model_transformer::DomainVariable;
use indexmap::IndexMap;
use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};
#[allow(unused)]
use std::fmt::{Display, Formatter, write};

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
    Infeasible,

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
                write!(
                    f,
                    "The problem was unable to be solved, it might be infeasible"
                )
            }
            SolverError::Infeasible => {
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

impl std::error::Error for SolverError {}

/// Rounds an `f64` to 6 decimal places and trims trailing zeros so solver
/// output stays readable: `1.9999999999` prints as `2`, `0.00000000015` as `0`.
/// Non-finite values (`inf`, `NaN`) are passed through unchanged.
pub(crate) fn format_float(value: f64) -> String {
    if !value.is_finite() {
        return value.to_string();
    }
    let rounded = (value * 1_000_000.0).round() / 1_000_000.0;
    let rounded = if rounded == 0.0 { 0.0 } else { rounded }; // normalise -0.0
    let mut s = format!("{:.6}", rounded);
    if s.contains('.') {
        while s.ends_with('0') {
            s.pop();
        }
        if s.ends_with('.') {
            s.pop();
        }
    }
    s
}

/// How a solution value is rendered by a solution's `Display`. Implemented for
/// the value types solutions use (`f64` and `MILPValue`) so numeric output is
/// rounded consistently to 6 decimal places.
pub(crate) trait DisplayValue {
    fn display_value(&self) -> String;
}

impl DisplayValue for f64 {
    fn display_value(&self) -> String {
        format_float(*self)
    }
}

/// Represents a variable assignment in a solution.
/// - `T`: The type of the variable's value
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Assignment<T> {
    pub name: String,
    pub value: T,
}

impl<T: Clone + Serialize + Copy + DeserializeOwned + DisplayValue> Display for Assignment<T> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} = {}", self.name, self.value.display_value())
    }
}

/// The status of a solve, reported independently of the underlying solver.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum SolutionStatus {
    /// A proven optimal solution.
    #[default]
    Optimal,
    /// A feasible solution whose optimality was not proven (for example, a time
    /// limit was reached before the search completed).
    Feasible,
    /// The problem was proven infeasible.
    Infeasible,
    /// The problem is unbounded.
    Unbounded,
}

/// Represents a solution to a linear programming problem.
/// - `T`: The type of the variables' values
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LpSolution<T> {
    assignment: Vec<Assignment<T>>,
    assignment_by_name: IndexMap<String, T>,
    constraints: IndexMap<String, f64>,
    value: f64,
    /// Solve status. Not serialized: it is solver metadata, not part of the
    /// portable solution shape.
    #[serde(skip)]
    status: SolutionStatus,
    /// Optional solver-provided dual values. Not serialized: they are backend
    /// metadata, not part of the portable solution shape.
    #[serde(skip)]
    shadow_prices: IndexMap<String, f64>,
}

fn build_assignment_map<T: Copy>(assignment: &[Assignment<T>]) -> IndexMap<String, T> {
    let mut assignment_by_name = IndexMap::with_capacity(assignment.len());
    for item in assignment {
        assignment_by_name
            .entry(item.name.clone())
            .or_insert(item.value);
    }
    assignment_by_name
}

impl<T: Clone + Serialize + DeserializeOwned + Copy + DisplayValue> Display for LpSolution<T> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "Optimal value: {}\n\n", format_float(self.value))?;
        write!(
            f,
            "Variables:\n{}",
            self.assignment
                .iter()
                .map(|a| a.to_string())
                .collect::<Vec<_>>()
                .join("\n")
        )?;
        let constraints = self
            .constraints
            .iter()
            .map(|(name, value)| format!("{} = {}", name, format_float(*value)))
            .collect::<Vec<_>>()
            .join("\n");
        write!(f, "\n\nConstraints:\n{}", constraints)
    }
}

impl<T: Clone + Serialize + DeserializeOwned + Copy + Display> LpSolution<T> {
    /// Creates a new solution with the given assignments and objective value.
    ///
    /// # Arguments
    /// * `assignment` - Vector of variable assignments
    /// * `value` - The objective function value at this solution
    /// * `constraints` - Map of constraint names to their values at this solution
    pub fn new(
        assignment: Vec<Assignment<T>>,
        value: f64,
        constraints: IndexMap<String, f64>,
    ) -> Self {
        Self {
            assignment_by_name: build_assignment_map(&assignment),
            assignment,
            value,
            constraints,
            status: SolutionStatus::Optimal,
            shadow_prices: IndexMap::new(),
        }
    }

    /// Returns the solve status.
    pub fn status(&self) -> SolutionStatus {
        self.status
    }

    /// Sets the solve status, returning the solution for chaining.
    pub fn with_status(mut self, status: SolutionStatus) -> Self {
        self.status = status;
        self
    }

    /// Sets optional solver-provided shadow prices, returning the solution for
    /// chaining.
    pub fn with_shadow_prices(mut self, shadow_prices: IndexMap<String, f64>) -> Self {
        self.shadow_prices = shadow_prices;
        self
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

    /// Returns the constraint values of this solution.
    pub fn constraints(&self) -> &IndexMap<String, f64> {
        &self.constraints
    }

    /// Returns the solver-provided shadow prices, if any.
    pub fn shadow_prices(&self) -> &IndexMap<String, f64> {
        &self.shadow_prices
    }

    /// Returns the solved value of a variable by its name.
    pub fn value_of(&self, name: &str) -> Option<T> {
        self.assignment_by_name.get(name).copied()
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
