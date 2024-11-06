/// A module for representing and manipulating linear programming models.
#[allow(unused_imports)]
use crate::prelude::*;
use indexmap::IndexMap;
use num_traits::Zero;
use std::fmt::Display;

use crate::math::{float_lt, VariableType};
use crate::parser::model_transformer::DomainVariable;
use crate::solvers::SolverError;
use crate::transformers::standard_linear_model::{format_var, StandardLinearModel};
use crate::utils::{remove_many, InputSpan};
use crate::{
    math::{Comparison, OptimizationType},
    transformers::standardizer::to_standard_form,
};

/// Represents a linear constraint in the form: coefficients * variables comparison_operator rhs
///
/// For example: 2x + 3y <= 5 would be represented as:
/// - coefficients: [2.0, 3.0]
/// - constraint_type: LessOrEqual
/// - rhs: 5.0
#[derive(Debug, Clone)]
#[cfg_attr(target_arch = "wasm32", wasm_bindgen)]
pub struct LinearConstraint {
    coefficients: Vec<f64>,
    rhs: f64,
    constraint_type: Comparison,
}

impl LinearConstraint {
    /// Creates a new linear constraint.
    ///
    /// # Arguments
    /// * `coefficients` - Vector of coefficients for each variable
    /// * `constraint_type` - Type of comparison operator (e.g., <=, =, >=)
    /// * `rhs` - Right-hand side constant value
    pub fn new(coefficients: Vec<f64>, constraint_type: Comparison, rhs: f64) -> LinearConstraint {
        LinearConstraint {
            coefficients,
            rhs,
            constraint_type,
        }
    }

    /// Returns a reference to the coefficient vector.
    pub fn coefficients(&self) -> &Vec<f64> {
        &self.coefficients
    }

    /// Returns a mutable reference to the coefficient vector.
    pub fn coefficients_mut(&mut self) -> &mut Vec<f64> {
        &mut self.coefficients
    }

    /// Removes coefficients at the specified indices.
    ///
    /// # Arguments
    /// * `indices` - Slice of indices to remove
    pub fn remove_coefficients_by_index(&mut self, indices: &[usize]) {
        remove_many(&mut self.coefficients, indices);
    }

    /// Returns the right-hand side value of the constraint.
    pub fn rhs(&self) -> f64 {
        self.rhs
    }

    /// Returns the comparison operator type of the constraint.
    pub fn constraint_type(&self) -> &Comparison {
        &self.constraint_type
    }

    /// Decomposes the constraint into its constituent parts.
    ///
    /// # Returns
    /// A tuple containing (coefficients, comparison operator, right-hand side)
    pub fn into_parts(self) -> (Vec<f64>, Comparison, f64) {
        (self.coefficients, self.constraint_type, self.rhs)
    }

    /// Ensures the coefficient vector has the specified size by padding with zeros if necessary.
    ///
    /// # Arguments
    /// * `size` - The desired size of the coefficient vector
    pub fn ensure_size(&mut self, size: usize) {
        self.coefficients.resize(size, 0.0);
    }
}

#[cfg_attr(target_arch = "wasm32", wasm_bindgen)]
#[cfg(target_arch = "wasm32")]
impl LinearConstraint {
    pub fn wasm_get_coefficients(&self) -> Vec<f64> {
        self.coefficients.clone()
    }
    pub fn wasm_get_rhs(&self) -> f64 {
        self.rhs
    }
    pub fn wasm_get_constraint_type(&self) -> Comparison {
        self.constraint_type
    }
}

/// Represents a complete linear programming model including variables, constraints, and objective function.
///
/// # Example
/// ```rust
/// use rooc::{OptimizationType, VariableType, Comparison, LinearModel};
/// use rooc::model_transformer::DomainVariable;
/// let mut model = LinearModel::new();
///
/// // Add variables
/// model.add_variable("x", VariableType::NonNegativeReal);
/// model.add_variable("y", VariableType::NonNegativeReal);
///
/// // Set objective function: minimize x + 2y
/// model.set_objective(vec![1.0, 2.0], OptimizationType::Min);
///
/// // Add constraint: x + y <= 10
/// model.add_constraint(vec![1.0, 1.0], Comparison::LessOrEqual, 10.0);
/// ```
#[derive(Debug, Clone)]
#[cfg_attr(target_arch = "wasm32", wasm_bindgen)]
pub struct LinearModel {
    variables: Vec<String>,
    domain: IndexMap<String, DomainVariable>,
    objective_offset: f64,
    optimization_type: OptimizationType,
    objective: Vec<f64>,
    constraints: Vec<LinearConstraint>,
}

impl Default for LinearModel {
    fn default() -> Self {
        LinearModel {
            variables: vec![],
            domain: IndexMap::new(),
            objective_offset: 0.0,
            optimization_type: OptimizationType::Min,
            objective: vec![],
            constraints: vec![],
        }
    }
}

/// Errors that can occur when manipulating a LinearModel.
#[derive(Debug)]
pub enum LinearModelError {
    /// Indicates that the number of coefficients provided exceeds the number of variables in the model.
    TooManyCoefficients,
}

impl LinearModel {
    /// Creates a new LinearModel from its constituent parts.
    ///
    /// # Arguments
    /// * `objective` - Vector of objective function coefficients
    /// * `optimization_type` - Whether to minimize or maximize
    /// * `objective_offset` - Constant term in objective function
    /// * `constraints` - Vector of linear constraints
    /// * `variables` - Vector of variable names
    /// * `domain` - Map of variable domains
    pub fn new_from_parts(
        objective: Vec<f64>,
        optimization_type: OptimizationType,
        objective_offset: f64,
        constraints: Vec<LinearConstraint>,
        variables: Vec<String>,
        domain: IndexMap<String, DomainVariable>,
    ) -> LinearModel {
        LinearModel {
            objective,
            constraints,
            optimization_type,
            variables,
            objective_offset,
            domain,
        }
    }

    /// Creates a new empty LinearModel.
    pub fn new() -> LinearModel {
        LinearModel::default()
    }

    /// Decomposes the model into its constituent parts.
    ///
    /// # Returns
    /// A tuple containing (objective coefficients, optimization type, objective offset,
    /// constraints, variables, domain)
    #[allow(clippy::type_complexity)]
    pub fn into_parts(
        self,
    ) -> (
        Vec<f64>,
        OptimizationType,
        f64,
        Vec<LinearConstraint>,
        Vec<String>,
        IndexMap<String, DomainVariable>,
    ) {
        (
            self.objective,
            self.optimization_type,
            self.objective_offset,
            self.constraints,
            self.variables,
            self.domain,
        )
    }

    /// Ensures all vectors in the model have consistent sizes.
    fn ensure_sizes(&mut self) {
        self.constraints
            .iter_mut()
            .for_each(|c| c.ensure_size(self.variables.len()));
        self.objective.resize(self.variables.len(), 0.0);
    }

    /// Adds a new variable to the model.
    ///
    /// # Arguments
    /// * `name` - Name of the variable
    /// * `domain` - Type/domain of the variable (e.g., Boolean, Integer, etc.)
    pub fn add_variable(&mut self, name: &str, domain: VariableType) {
        self.variables.push(name.to_string());
        self.domain.insert(
            name.to_string(),
            DomainVariable::new(domain, InputSpan::default()),
        );
        self.ensure_sizes();
    }

    /// Adds a new constraint to the model.
    ///
    /// # Arguments
    /// * `coefficients` - Vector of coefficients for the constraint
    /// * `constraint_type` - Type of comparison operator
    /// * `rhs` - Right-hand side value
    ///
    /// # Returns
    /// * `Ok(())` if successful
    /// * `Err(LinearModelError)` if too many coefficients provided
    /// # Panics
    /// If there are more coefficient than how many variables there are
    pub fn add_constraint(
        &mut self,
        mut coefficients: Vec<f64>,
        constraint_type: Comparison,
        rhs: f64,
    ) {
        if coefficients.len() > self.variables.len() {
            panic!(
                "Coefficients have {} variables while only {} were defined",
                coefficients.len(),
                self.variables.len()
            );
        }
        coefficients.resize(self.variables.len(), 0.0);
        self.constraints
            .push(LinearConstraint::new(coefficients, constraint_type, rhs));
    }

    /// Sets the objective function of the model.
    ///
    /// # Arguments
    /// * `objective` - Vector of objective function coefficients
    /// * `optimization_type` - Whether to minimize or maximize
    ///
    /// # Returns
    /// * `Ok(())` if successful
    /// * `Err(LinearModelError)` if too many coefficients provided
    /// # Panics
    /// If there are more coefficient than how many variables there are
    pub fn set_objective(&mut self, mut objective: Vec<f64>, optimization_type: OptimizationType) {
        if objective.len() > self.variables.len() {
            panic!(
                "Coefficients have {} variables while only {} were defined",
                objective.len(),
                self.variables.len()
            );
        }
        objective.resize(self.variables.len(), 0.0);
        self.objective = objective;
        self.optimization_type = optimization_type;
    }

    /// Returns the optimization type (minimize/maximize).
    pub fn optimization_type(&self) -> &OptimizationType {
        &self.optimization_type
    }

    /// Converts the model to standard form.
    pub fn into_standard_form(self) -> Result<StandardLinearModel, SolverError> {
        to_standard_form(self)
    }

    /// Returns a reference to the objective function coefficients.
    pub fn objective(&self) -> &Vec<f64> {
        &self.objective
    }

    /// Returns a reference to the model's constraints.
    pub fn constraints(&self) -> &Vec<LinearConstraint> {
        &self.constraints
    }

    /// Returns a reference to the variable names.
    pub fn variables(&self) -> &Vec<String> {
        &self.variables
    }

    /// Returns the constant term in the objective function.
    pub fn objective_offset(&self) -> f64 {
        self.objective_offset
    }

    /// Returns a reference to the variable domains.
    pub fn domain(&self) -> &IndexMap<String, DomainVariable> {
        &self.domain
    }
}

impl Display for LinearModel {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let constraints = self.constraints.iter().map(|c| {
            let mut is_first = true;
            let coefficients = c
                .coefficients
                .iter()
                .enumerate()
                .flat_map(|(i, c)| {
                    if c.is_zero() {
                        None
                    } else {
                        let var = format_var(&self.variables[i], *c, is_first);
                        is_first = false;
                        Some(var)
                    }
                })
                .collect::<Vec<String>>()
                .join(" ");
            let rhs = if c.rhs.is_zero() {
                "0".to_string()
            } else {
                c.rhs.to_string()
            };
            format!("    {} {} {rhs}", coefficients, c.constraint_type,)
        });

        let constraints = constraints.collect::<Vec<String>>().join("\n");
        let mut is_first = true;
        let objective = self
            .objective
            .iter()
            .enumerate()
            .flat_map(|(i, c)| {
                if c.is_zero() {
                    None
                } else {
                    let var = format_var(&self.variables[i], *c, is_first);
                    is_first = false;
                    Some(var)
                }
            })
            .collect::<Vec<String>>()
            .join(" ");
        let offset = if self.objective_offset.is_zero() {
            "".to_string()
        } else if float_lt(self.objective_offset, 0.0) {
            format!(" - {}", self.objective_offset.abs())
        } else {
            format!(" + {}", self.objective_offset)
        };
        let objective = format!("{}{}", objective, offset);
        write!(
            f,
            "{} {}\ns.t.\n{}",
            self.optimization_type, objective, constraints
        )
    }
}

#[cfg_attr(target_arch = "wasm32", wasm_bindgen)]
#[cfg(target_arch = "wasm32")]
impl LinearModel {
    pub fn wasm_get_objective(&self) -> Vec<f64> {
        self.objective.clone()
    }
    pub fn wasm_get_constraints(&self) -> Vec<LinearConstraint> {
        self.constraints.clone()
    }
    pub fn wasm_get_variables(&self) -> Vec<String> {
        self.variables.clone()
    }
    pub fn wasm_get_objective_offset(&self) -> f64 {
        self.objective_offset
    }
    pub fn wasm_get_optimization_type(&self) -> OptimizationType {
        self.optimization_type.clone()
    }

    pub fn wasm_to_string(&self) -> String {
        format!("{}", self)
    }
}
