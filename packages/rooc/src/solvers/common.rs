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
///
/// Only states in which a usable assignment exists are representable. As in
/// `good_lp`, an infeasible or unbounded model is reported through
/// [`SolverError::Infeasible`] and [`SolverError::Unbounded`] instead of a
/// status, because neither case yields values to read.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum SolutionStatus {
    /// A proven optimal solution.
    #[default]
    Optimal,
    /// A feasible solution whose optimality was not proven (for example, a time
    /// limit was reached before the search completed).
    Feasible,
}

/// Why a solve stopped.
///
/// This is orthogonal to [`SolutionStatus`]: it explains what ended the search,
/// while the status says whether the resulting assignment is proven optimal.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum TerminationReason {
    /// The search completed the optimality proof.
    #[default]
    ProvenOptimal,
    /// The configured relative MIP gap was reached before the exact proof.
    MipGap,
    /// The wall-clock budget was exhausted.
    TimeLimit,
    /// The branch-and-bound node budget was exhausted.
    NodeLimit,
    /// The iteration budget of an iterative method was exhausted. Reported by
    /// the tableau simplex, which counts pivots rather than nodes.
    IterationLimit,
}

impl Display for TerminationReason {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let text = match self {
            TerminationReason::ProvenOptimal => "optimality was proven",
            TerminationReason::MipGap => "the MIP gap was reached",
            TerminationReason::TimeLimit => "the time limit was reached",
            TerminationReason::NodeLimit => "the node limit was reached",
            TerminationReason::IterationLimit => "the iteration limit was reached",
        };
        write!(f, "{}", text)
    }
}

/// A solve that stopped at a limit before any usable assignment was found.
///
/// It deliberately carries no variable values: unlike a [`LpSolution`], there is
/// no validated assignment to read. The bound and gap are reported when the
/// backend tracks them, so callers can still show search progress.
#[derive(Debug, Clone)]
pub struct InterruptedSolve {
    reason: TerminationReason,
    best_bound: Option<f64>,
    gap: Option<f64>,
}

impl InterruptedSolve {
    /// Creates an interrupted solve that reports no bound information.
    pub fn new(reason: TerminationReason) -> Self {
        Self {
            reason,
            best_bound: None,
            gap: None,
        }
    }

    /// Sets the best objective bound proven before stopping, returning the value
    /// for chaining.
    pub fn with_best_bound(mut self, best_bound: Option<f64>) -> Self {
        self.best_bound = best_bound;
        self
    }

    /// Sets the relative gap reached before stopping, returning the value for
    /// chaining.
    pub fn with_gap(mut self, gap: Option<f64>) -> Self {
        self.gap = gap;
        self
    }

    /// Returns why the search stopped.
    pub fn termination_reason(&self) -> TerminationReason {
        self.reason
    }

    /// Returns the best objective bound proven before stopping, when the backend
    /// reports one. It is expressed on the same scale as
    /// [`LpSolution::value`], including the model's objective offset.
    pub fn best_bound(&self) -> Option<f64> {
        self.best_bound
    }

    /// Returns the relative gap reached before stopping, when the backend
    /// reports one. See [`LpSolution::gap`] for how it is measured.
    pub fn gap(&self) -> Option<f64> {
        self.gap
    }
}

impl Display for InterruptedSolve {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "No solution was found ({})", self.reason)
    }
}

impl std::error::Error for InterruptedSolve {}

/// The result of a solve that did not fail.
///
/// A solve that ran to completion produces a [`SolveOutcome::Solution`]. Hitting
/// a limit is not an error: if an assignment was already found it is returned as
/// a solution whose [`SolutionStatus`] is [`SolutionStatus::Feasible`], and only
/// when no assignment exists yet does the outcome become
/// [`SolveOutcome::Interrupted`].
///
/// The type parameter is the backend's own solution type, so custom solvers
/// plugged into the builder keep their solution shape.
#[derive(Debug, Clone)]
pub enum SolveOutcome<S> {
    /// A usable assignment, optimal or feasible.
    Solution(S),
    /// A limit stopped the search before any usable assignment was found.
    Interrupted(InterruptedSolve),
}

impl<S> SolveOutcome<S> {
    /// Borrows the solution, or `None` when the solve was interrupted.
    pub fn solution(&self) -> Option<&S> {
        match self {
            SolveOutcome::Solution(solution) => Some(solution),
            SolveOutcome::Interrupted(_) => None,
        }
    }

    /// Consumes the outcome and returns its solution.
    ///
    /// The [`InterruptedSolve`] is returned as the error so it can still be
    /// inspected; it implements [`std::error::Error`], so `?` composes.
    pub fn into_solution(self) -> Result<S, InterruptedSolve> {
        match self {
            SolveOutcome::Solution(solution) => Ok(solution),
            SolveOutcome::Interrupted(interrupted) => Err(interrupted),
        }
    }

    /// Returns whether a usable assignment is available.
    pub fn has_solution(&self) -> bool {
        matches!(self, SolveOutcome::Solution(_))
    }

    /// Applies `f` to the contained solution, preserving an interruption.
    pub fn map<U, F: FnOnce(S) -> U>(self, f: F) -> SolveOutcome<U> {
        match self {
            SolveOutcome::Solution(solution) => SolveOutcome::Solution(f(solution)),
            SolveOutcome::Interrupted(interrupted) => SolveOutcome::Interrupted(interrupted),
        }
    }
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
    /// Why the search that produced this solution stopped. Not serialized, for
    /// the same reason as `status`.
    #[serde(skip)]
    termination_reason: TerminationReason,
    /// Best objective bound proven by the search, when the backend reports one.
    #[serde(skip)]
    best_bound: Option<f64>,
    /// Relative gap between this solution and the best bound, when the backend
    /// reports one.
    #[serde(skip)]
    gap: Option<f64>,
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
        // A solution that stopped at a limit names that limit. Gating on the
        // reason rather than the status keeps "proven optimal" from being
        // restated as a parenthetical, which would read as nonsense next to a
        // `Feasible` status.
        write!(f, "Status: {:?}", self.status)?;
        if self.termination_reason != TerminationReason::ProvenOptimal {
            write!(f, " ({})", self.termination_reason)?;
        }
        write!(f, "\nObjective value: {}\n\n", format_float(self.value))?;
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
            termination_reason: TerminationReason::ProvenOptimal,
            best_bound: None,
            gap: None,
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

    /// Returns why the search that produced this solution stopped.
    ///
    /// A [`SolutionStatus::Optimal`] solution always reports
    /// [`TerminationReason::ProvenOptimal`]; a [`SolutionStatus::Feasible`] one
    /// reports the limit that ended the search.
    pub fn termination_reason(&self) -> TerminationReason {
        self.termination_reason
    }

    /// Sets the termination reason, returning the solution for chaining.
    pub fn with_termination_reason(mut self, reason: TerminationReason) -> Self {
        self.termination_reason = reason;
        self
    }

    /// Returns the best objective bound proven by the search, when the backend
    /// reports one. Backends that expose no bound information (every `good_lp`
    /// backend) return `None`.
    ///
    /// It is on the same scale as [`LpSolution::value`], including the model's
    /// objective offset, so the two are directly comparable.
    pub fn best_bound(&self) -> Option<f64> {
        self.best_bound
    }

    /// Sets the best objective bound, returning the solution for chaining.
    pub fn with_best_bound(mut self, best_bound: Option<f64>) -> Self {
        self.best_bound = best_bound;
        self
    }

    /// Returns the relative gap between this solution and the best bound, when
    /// the backend reports one.
    ///
    /// This is the quantity a configured MIP gap is checked against. It is
    /// measured on the solver's own objective and therefore excludes the model's
    /// constant objective offset, so it is not simply
    /// `(value - best_bound) / value` when the offset is non-zero.
    pub fn gap(&self) -> Option<f64> {
        self.gap
    }

    /// Sets the relative gap, returning the solution for chaining.
    pub fn with_gap(mut self, gap: Option<f64>) -> Self {
        self.gap = gap;
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

/// Accessors available when the outcome carries the crate's own solution type.
/// They read through to the solution or to the interruption, whichever is
/// present, so callers can report progress without matching first.
impl<T: Clone + Serialize + DeserializeOwned + Copy + Display> SolveOutcome<LpSolution<T>> {
    /// Returns why the search stopped, whether or not a solution was found.
    pub fn termination_reason(&self) -> TerminationReason {
        match self {
            SolveOutcome::Solution(solution) => solution.termination_reason(),
            SolveOutcome::Interrupted(interrupted) => interrupted.termination_reason(),
        }
    }

    /// Returns the status of the contained solution, or `None` when the solve
    /// was interrupted before finding one.
    pub fn status(&self) -> Option<SolutionStatus> {
        self.solution().map(LpSolution::status)
    }

    /// Returns whether this outcome holds a solution with proven optimality.
    pub fn is_optimal(&self) -> bool {
        matches!(self.status(), Some(SolutionStatus::Optimal))
    }

    /// Returns the best objective bound, from the solution or the interruption.
    pub fn best_bound(&self) -> Option<f64> {
        match self {
            SolveOutcome::Solution(solution) => solution.best_bound(),
            SolveOutcome::Interrupted(interrupted) => interrupted.best_bound(),
        }
    }

    /// Returns the relative gap, from the solution or the interruption.
    pub fn gap(&self) -> Option<f64> {
        match self {
            SolveOutcome::Solution(solution) => solution.gap(),
            SolveOutcome::Interrupted(interrupted) => interrupted.gap(),
        }
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
