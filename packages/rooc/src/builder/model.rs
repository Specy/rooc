//! The model builder, its constraints, and the build error type.

use super::expr::{Expr, Var, to_exp};
use super::solution::BuilderSolution;
use super::solvers::Solver;
use crate::InputSpan;
use crate::math::{Comparison, OptimizationType, VariableType};
use crate::parser::model_transformer::transformer_context::DomainVariable;
use crate::parser::model_transformer::{Constraint, Model, Objective};
use crate::solvers::SolverError;
use crate::transformers::linear_model::LinearModel;
use crate::transformers::linearizer::{LinearizationError, Linearizer};
use indexmap::IndexMap;

/// Index-based constraint used internally in the ModelBuilder.
#[derive(Debug, Clone)]
pub struct BuilderConstraint {
    pub name: String,
    pub lhs: Expr,
    pub constraint_type: Comparison,
    pub rhs: Expr,
    pub is_logic_assertion: bool,
}

impl BuilderConstraint {
    pub fn new(lhs: Expr, constraint_type: Comparison, rhs: Expr, name: String) -> Self {
        Self {
            name,
            lhs,
            constraint_type,
            rhs,
            is_logic_assertion: false,
        }
    }

    pub fn new_logic_assertion(lhs: Expr, name: String) -> Self {
        Self {
            name,
            lhs,
            constraint_type: Comparison::Equal,
            rhs: Expr::Number(1.0),
            is_logic_assertion: true,
        }
    }

    pub fn to_constraint(&self, names: &[String]) -> Constraint {
        if self.is_logic_assertion {
            Constraint::new_logic_assertion(to_exp(&self.lhs, names), self.name.clone())
        } else {
            Constraint::new(to_exp(&self.lhs, names), self.constraint_type, to_exp(&self.rhs, names), self.name.clone())
        }
    }
}

/// Builder for an optimization model.
///
/// Declare variables with [`add_var`](ModelBuilder::add_var) (or the
/// [`vars!`](crate::vars) macro), then add constraints with
/// [`with`](ModelBuilder::with) / [`with_all`](ModelBuilder::with_all) and set
/// the objective with [`maximize`](ModelBuilder::maximize),
/// [`minimize`](ModelBuilder::minimize) or [`satisfy`](ModelBuilder::satisfy),
/// in any order. Finish by calling [`solve_with`](ModelBuilder::solve_with).
#[derive(Clone)]
pub struct ModelBuilder {
    variable_names: Vec<String>,
    domain: IndexMap<String, DomainVariable>,
    constraints: Vec<BuilderConstraint>,
    objective: Option<(OptimizationType, Expr)>,
}

impl Default for ModelBuilder {
    fn default() -> Self {
        Self::new()
    }
}

impl ModelBuilder {
    pub fn new() -> Self {
        Self {
            variable_names: Vec::new(),
            domain: IndexMap::new(),
            constraints: Vec::new(),
            objective: None,
        }
    }

    /// Declares a new variable and returns a copyable handle to it.
    ///
    /// # Panics
    /// Panics if a variable with the same name was already declared. Names must
    /// be unique because the model is compiled to a name-based representation,
    /// so two variables sharing a name would otherwise silently collapse into
    /// one and produce wrong results.
    pub fn add_var(&mut self, name: impl Into<String>, var_type: VariableType) -> Var {
        let name_str = name.into();
        if self.domain.contains_key(&name_str) {
            panic!("a variable named \"{name_str}\" already exists; variable names must be unique");
        }
        let idx = self.variable_names.len();
        self.variable_names.push(name_str.clone());
        self.domain
            .insert(name_str, DomainVariable::new(var_type, InputSpan::default()));
        Var { index: idx }
    }

    /// Declares an indexed family of `count` variables named
    /// `{name}_0 .. {name}_{count - 1}` and returns their handles as a `Vec`.
    ///
    /// This is what the [`vars!`](crate::vars) array form (`x[n]: ...`) expands
    /// to; index the returned vector (`x[i]`) to use a member in an expression.
    ///
    /// # Panics
    /// Panics if any generated name collides with an existing variable.
    pub fn add_vars(&mut self, name: &str, count: usize, var_type: VariableType) -> Vec<Var> {
        (0..count)
            .map(|i| self.add_var(format!("{name}_{i}"), var_type))
            .collect()
    }

    /// Adds a constraint to the model, returning the builder for chaining.
    ///
    /// Constraints may be added before or after the objective is set.
    pub fn with(mut self, constraint: BuilderConstraint) -> Self {
        self.constraints.push(constraint);
        self
    }

    /// Adds every constraint yielded by an iterator, returning the builder for
    /// chaining.
    pub fn with_all(
        mut self,
        constraints: impl IntoIterator<Item = BuilderConstraint>,
    ) -> Self {
        self.constraints.extend(constraints);
        self
    }

    /// Sets the objective to maximize `expr`, returning the builder for chaining.
    ///
    /// May be called at any point and overrides any objective set earlier.
    pub fn maximize(mut self, expr: impl Into<Expr>) -> Self {
        self.objective = Some((OptimizationType::Max, expr.into()));
        self
    }

    /// Sets the objective to minimize `expr`, returning the builder for chaining.
    ///
    /// May be called at any point and overrides any objective set earlier.
    pub fn minimize(mut self, expr: impl Into<Expr>) -> Self {
        self.objective = Some((OptimizationType::Min, expr.into()));
        self
    }

    /// Sets a feasibility objective: find any point satisfying the constraints.
    pub fn satisfy(mut self) -> Self {
        self.objective = Some((OptimizationType::Satisfy, Expr::Number(0.0)));
        self
    }

    /// Converts the builder into a name-based [`Model`], the same representation
    /// the textual front-end compiles to. Use this as an escape hatch into the
    /// rest of the pipeline. If no objective was set, a feasibility (satisfy)
    /// objective is used.
    pub fn into_model(self) -> Model {
        let ModelBuilder {
            variable_names,
            mut domain,
            constraints,
            objective,
        } = self;
        // Every declared variable is marked as used so that it survives
        // linearization and its handle always resolves in the solution, even if
        // it never appears in a constraint or the objective.
        for var in domain.values_mut() {
            var.increment_usage();
        }
        let constraints: Vec<Constraint> = constraints
            .iter()
            .map(|c| c.to_constraint(&variable_names))
            .collect();
        let (objective_type, objective_expr) =
            objective.unwrap_or((OptimizationType::Satisfy, Expr::Number(0.0)));
        let objective = Objective::new(objective_type, to_exp(&objective_expr, &variable_names));
        Model::new(objective, constraints, domain)
    }

    /// Linearizes the model into a [`LinearModel`], ready for any solver that
    /// follows the crate's `Fn(&LinearModel) -> Result<_, _>` convention.
    pub fn linearize(self) -> Result<LinearModel, LinearizationError> {
        Linearizer::linearize(self.into_model())
    }

    /// Solves the model with the given [`Solver`], returning a solution whose
    /// values can be read back through the [`Var`] handles minted by the builder.
    ///
    /// Any type implementing [`Solver`] can be passed, including user-defined
    /// solvers, so new back-ends need no changes to the builder.
    pub fn solve_with<S: Solver>(self, solver: S) -> Result<BuilderSolution<S>, BuilderError> {
        let variable_names = self.variable_names.clone();
        // Keep a copy of the model so the solution can be edited and re-solved
        // by solvers that opt into `Reoptimizable`.
        let model = self.clone();
        let linearized = self.linearize()?;
        let solution = solver.solve(&linearized)?;
        Ok(BuilderSolution::new(solution, variable_names, model, solver))
    }
}

/// Error returned when solving a model built with [`ModelBuilder`].
#[derive(Debug)]
pub enum BuilderError {
    /// The model could not be linearized.
    Linearization(LinearizationError),
    /// The solver failed on the linearized model.
    Solver(SolverError),
}

impl std::fmt::Display for BuilderError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            BuilderError::Linearization(e) => write!(f, "{}", e),
            BuilderError::Solver(e) => write!(f, "{}", e),
        }
    }
}

impl std::error::Error for BuilderError {}

impl From<LinearizationError> for BuilderError {
    fn from(e: LinearizationError) -> Self {
        BuilderError::Linearization(e)
    }
}

impl From<SolverError> for BuilderError {
    fn from(e: SolverError) -> Self {
        BuilderError::Solver(e)
    }
}

