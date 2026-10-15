use crate::domain_declaration::format_domain;
use crate::math::{VariableType, float_lt};
use crate::parser::model_transformer::DomainVariable;
/// A module for representing and manipulating linear programming models.
#[allow(unused_imports)]
use crate::prelude::*;
use crate::solvers::SolverError;
use crate::transformers::standard_linear_model::{StandardLinearModel, format_var};
use crate::utils::InputSpan;
use crate::{
    math::{Comparison, OptimizationType},
    transformers::standardizer::to_standard_form,
};
use indexmap::IndexMap;
use num_traits::Zero;
use serde::{Deserialize, Serialize};
use std::fmt::Display;

/// Represents a linear constraint in the form: coefficients * variables comparison_operator rhs
///
/// Only the nonzero coefficients are stored, as the indexes of their variables
/// and their values: most variables of a model don't appear in most of its
/// constraints, so a coefficient for every variable would be mostly zeros.
/// An index is the position of the variable in [`LinearModel::variables`], and
/// `values[k]` is the coefficient of the variable at `indices[k]`; see
/// [`Self::terms`] to read them back as variable names.
///
/// For example: 2x + 3z <= 5, in a model with the variables x, y and z, would
/// be represented as:
/// - indices: [0, 2]
/// - values: [2.0, 3.0]
/// - constraint_type: LessOrEqual
/// - rhs: 5.0
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(try_from = "SparseConstraintData")]
#[cfg_attr(target_arch = "wasm32", wasm_bindgen)]
pub struct LinearConstraint {
    name: String,
    /// The indexes of the variables with a nonzero coefficient, increasing
    indices: Vec<usize>,
    /// The nonzero coefficients, `values[k]` belongs to the variable `indices[k]`
    values: Vec<f64>,
    rhs: f64,
    constraint_type: Comparison,
}

/// A [`LinearConstraint`] as it is deserialized, the coefficients may come
/// from JavaScript so they are normalized before being used.
#[derive(Deserialize)]
struct SparseConstraintData {
    name: String,
    indices: Vec<usize>,
    values: Vec<f64>,
    rhs: f64,
    constraint_type: Comparison,
}

impl TryFrom<SparseConstraintData> for LinearConstraint {
    type Error = String;
    fn try_from(data: SparseConstraintData) -> Result<Self, Self::Error> {
        if data.indices.len() != data.values.len() {
            return Err(format!(
                "Constraint \"{}\" has {} indices but {} values",
                data.name,
                data.indices.len(),
                data.values.len()
            ));
        }
        let terms = data.indices.into_iter().zip(data.values);
        Ok(
            LinearConstraint::new_sparse(terms, data.constraint_type, data.rhs)
                .with_name(&data.name),
        )
    }
}

#[cfg_attr(target_arch = "wasm32", wasm_bindgen(typescript_custom_section))]
#[allow(non_upper_case_globals)]
#[cfg(target_arch = "wasm32")]
const ILinearConstraint: &'static str = r#"

/**
 * Only the nonzero coefficients are listed: values[k] is the coefficient
 * of the variable model.variables[indices[k]] of the SerializedLinearModel
 * the constraint belongs to.
 */
export type SerializedLinearConstraint = {
    rhs: number
    indices: number[]
    values: number[]
    constraint_type: SerializedComparison
    name: string
}

export type SerializedComparison = {
    type: "LessOrEqual" | "GreaterOrEqual" | "Equal" | "Less" | "Greater"
}
"#;

impl LinearConstraint {
    /// Creates a new linear constraint from the coefficient of every variable,
    /// zeros included. Only the nonzero coefficients are kept.
    ///
    /// # Arguments
    /// * `coefficients` - Vector of coefficients for each variable
    /// * `constraint_type` - Type of comparison operator (e.g., <=, =, >=)
    /// * `rhs` - Right-hand side constant value
    pub fn new(coefficients: Vec<f64>, constraint_type: Comparison, rhs: f64) -> LinearConstraint {
        Self::new_sparse(coefficients.into_iter().enumerate(), constraint_type, rhs)
    }

    /// Creates a new linear constraint with a name, see [`Self::new`].
    pub fn new_with_name(
        coefficients: Vec<f64>,
        constraint_type: Comparison,
        rhs: f64,
        name: String,
    ) -> LinearConstraint {
        let mut constraint = Self::new(coefficients, constraint_type, rhs);
        constraint.name = name;
        constraint
    }

    /// Creates a new linear constraint from its nonzero coefficients, given as
    /// `(variable index, coefficient)` pairs in any order. The coefficients of
    /// the same variable are added together, and zeros are dropped.
    ///
    /// # Example
    /// ```rust
    /// use rooc::{Comparison, LinearConstraint};
    ///
    /// // 2x + 3z <= 5, in a model with the variables x, y and z
    /// let constraint = LinearConstraint::new_sparse([(2, 3.0), (0, 2.0)], Comparison::LessOrEqual, 5.0);
    /// assert_eq!(constraint.indices(), &[0, 2]);
    /// assert_eq!(constraint.to_dense(3), vec![2.0, 0.0, 3.0]);
    /// ```
    pub fn new_sparse(
        coefficients: impl IntoIterator<Item = (usize, f64)>,
        constraint_type: Comparison,
        rhs: f64,
    ) -> LinearConstraint {
        let mut terms = coefficients.into_iter().collect::<Vec<(usize, f64)>>();
        terms.sort_by_key(|(index, _)| *index);
        let mut merged: Vec<(usize, f64)> = Vec::with_capacity(terms.len());
        for (index, value) in terms {
            match merged.last_mut() {
                Some((last, sum)) if *last == index => *sum += value,
                _ => merged.push((index, value)),
            }
        }
        let (indices, values) = merged
            .into_iter()
            .filter(|(_, value)| *value != 0.0)
            .unzip();
        LinearConstraint {
            name: String::new(),
            indices,
            values,
            rhs,
            constraint_type,
        }
    }

    /// Sets the name of the constraint.
    pub fn with_name(mut self, name: &str) -> Self {
        self.name = name.to_string();
        self
    }

    /// Returns the name of the constraint.
    pub fn name(&self) -> String {
        self.name.clone()
    }

    /// Returns the indexes of the variables with a nonzero coefficient, in
    /// increasing order. An index is the position of the variable in
    /// [`LinearModel::variables`].
    pub fn indices(&self) -> &[usize] {
        &self.indices
    }

    /// Returns the nonzero coefficients, `values()[k]` is the coefficient of
    /// the variable at `indices()[k]`.
    pub fn values(&self) -> &[f64] {
        &self.values
    }

    /// Returns the nonzero coefficients as `(variable index, coefficient)`
    /// pairs, by increasing variable index.
    ///
    /// # Example
    /// ```rust
    /// use rooc::{Comparison, LinearModel, OptimizationType, VariableType};
    ///
    /// let mut model = LinearModel::new();
    /// model.add_variable("x", VariableType::non_negative_real());
    /// model.add_variable("y", VariableType::non_negative_real());
    /// model.add_variable("z", VariableType::non_negative_real());
    /// model.set_objective(vec![1.0, 1.0, 1.0], OptimizationType::Min);
    /// model.add_constraint(vec![2.0, 0.0, 3.0], Comparison::LessOrEqual, 5.0);
    ///
    /// // the index of a term is the position of its variable in the model
    /// let constraint = &model.constraints()[0];
    /// let terms = constraint
    ///     .terms()
    ///     .map(|(index, coefficient)| format!("{coefficient}{}", model.variables()[index]))
    ///     .collect::<Vec<_>>();
    /// assert_eq!(terms, ["2x", "3z"]);
    /// ```
    pub fn terms(&self) -> impl Iterator<Item = (usize, f64)> + '_ {
        self.indices
            .iter()
            .copied()
            .zip(self.values.iter().copied())
    }

    /// Returns the coefficient of the variable at `index`, zero when the
    /// variable does not appear in the constraint.
    pub fn coefficient(&self, index: usize) -> f64 {
        match self.indices.binary_search(&index) {
            Ok(position) => self.values[position],
            Err(_) => 0.0,
        }
    }

    /// Returns the coefficient of every variable, zeros included, in a model
    /// with `variables` variables.
    ///
    /// # Panics
    /// If the constraint uses a variable whose index is not below `variables`
    pub fn to_dense(&self, variables: usize) -> Vec<f64> {
        let mut coefficients = vec![0.0; variables];
        for (index, value) in self.terms() {
            coefficients[index] = value;
        }
        coefficients
    }

    /// Removes the variables at the specified indices, the variables after
    /// them move down to fill their place.
    ///
    /// # Arguments
    /// * `indices` - Slice of indices to remove
    pub fn remove_coefficients_by_index(&mut self, indices: &[usize]) {
        let mut removed = indices.to_vec();
        removed.sort_unstable();
        removed.dedup();
        let mut kept = 0;
        for position in 0..self.indices.len() {
            let index = self.indices[position];
            //the insertion point counts the removed variables before this one
            if let Err(shift) = removed.binary_search(&index) {
                self.indices[kept] = index - shift;
                self.values[kept] = self.values[position];
                kept += 1;
            }
        }
        self.indices.truncate(kept);
        self.values.truncate(kept);
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
    /// A tuple containing (indices, values, comparison operator, right-hand side),
    /// where `values[k]` is the coefficient of the variable at `indices[k]`
    pub fn into_parts(self) -> (Vec<usize>, Vec<f64>, Comparison, f64) {
        (self.indices, self.values, self.constraint_type, self.rhs)
    }
}

#[cfg_attr(target_arch = "wasm32", wasm_bindgen)]
#[cfg(target_arch = "wasm32")]
impl LinearConstraint {
    pub fn wasm_get_indices(&self) -> Vec<usize> {
        self.indices.clone()
    }
    pub fn wasm_get_values(&self) -> Vec<f64> {
        self.values.clone()
    }
    pub fn wasm_get_coefficients(&self, variables: usize) -> Vec<f64> {
        self.to_dense(variables)
    }
    pub fn wasm_get_rhs(&self) -> f64 {
        self.rhs
    }
    pub fn wasm_get_constraint_type(&self) -> Comparison {
        self.constraint_type
    }
}

impl Display for LinearConstraint {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let coefficients = self
            .terms()
            .enumerate()
            .map(|(position, (i, c))| format_var(&format!("x{}", i + 1), c, position == 0))
            .collect::<Vec<String>>()
            .join(" ");
        let lhs = if coefficients.is_empty() {
            "0".to_string()
        } else {
            coefficients
        };
        let rhs = if self.rhs.is_zero() {
            "0".to_string()
        } else {
            self.rhs.to_string()
        };
        write!(f, "{} {} {}", lhs, self.constraint_type, rhs)
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
/// model.add_variable("x", VariableType::non_negative_real());
/// model.add_variable("y", VariableType::non_negative_real());
///
/// // Set objective function: minimize x + 2y
/// model.set_objective(vec![1.0, 2.0], OptimizationType::Min);
///
/// // Add constraint: x + y <= 10
/// model.add_constraint(vec![1.0, 1.0], Comparison::LessOrEqual, 10.0);
/// ```
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(target_arch = "wasm32", wasm_bindgen)]
pub struct LinearModel {
    variables: Vec<String>,
    domain: IndexMap<String, DomainVariable>,
    objective_offset: f64,
    optimization_type: OptimizationType,
    objective: Vec<f64>,
    constraints: Vec<LinearConstraint>,
}
#[cfg_attr(target_arch = "wasm32", wasm_bindgen(typescript_custom_section))]
#[allow(non_upper_case_globals)]
#[cfg(target_arch = "wasm32")]
const ISerializedLinearModel: &'static str = r#"

export type SerializedLinearModel = {
    variables: string[]
    domain: Record<string, DomainVariable>
    objective_offset: number
    optimization_type: SerializedOptimizationType
    objective: number[]
    constraints: SerializedLinearConstraint[]
}

export type SerializedOptimizationType = {
    type: "Min" | "Max" | "Satisfy"
}
"#;

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
        //constraints only store their nonzero coefficients, so only the
        //objective has to grow
        self.objective.resize(self.variables.len(), 0.0);
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
        coefficients: Vec<f64>,
        constraint_type: Comparison,
        rhs: f64,
    ) {
        self.add_named_constraint(coefficients, constraint_type, rhs, "");
    }

    /// Adds a new constraint to the model.
    ///
    /// # Arguments
    /// * `coefficients` - Vector of coefficients for the constraint
    /// * `constraint_type` - Type of comparison operator
    /// * `rhs` - Right-hand side value
    /// * `name` - Name of the constraint
    ///
    /// # Returns
    /// * `Ok(())` if successful
    /// * `Err(LinearModelError)` if too many coefficients provided
    /// # Panics
    /// If there are more coefficient than how many variables there are
    pub fn add_named_constraint(
        &mut self,
        coefficients: Vec<f64>,
        constraint_type: Comparison,
        rhs: f64,
        name: &str,
    ) {
        if coefficients.len() > self.variables.len() {
            panic!(
                "Coefficients have {} variables while only {} were defined",
                coefficients.len(),
                self.variables.len()
            );
        }
        self.constraints.push(LinearConstraint::new_with_name(
            coefficients,
            constraint_type,
            rhs,
            name.to_string(),
        ));
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

    pub fn calc_objective(&self, values: &Vec<f64>) -> f64 {
        if values.len() != self.objective.len() {
            panic!(
                "Objective has {} coefficients while {} values were provided",
                self.objective.len(),
                values.len()
            );
        }
        self.objective
            .iter()
            .zip(values)
            .map(|(c, v)| c * v)
            .sum::<f64>()
            + self.objective_offset
    }

    pub fn calc_constraints(&self, values: &[f64]) -> Vec<(String, f64)> {
        if values.len() != self.variables.len() {
            panic!(
                "The model has {} variables while {} values were provided",
                self.variables.len(),
                values.len()
            );
        }
        self.constraints
            .iter()
            .map(|c| {
                let value = c.terms().map(|(i, c)| c * values[i]).sum::<f64>();
                (c.name(), value)
            })
            .collect()
    }

    /// Renders the model in CPLEX LP format.
    ///
    /// This is an explicit export, called deliberately; it is not the type's
    /// `Display`. Boolean variables are written in a `Binary` section, integers
    /// in `General`, and any non-default continuous bounds in `Bounds`.
    pub fn to_lp_format(&self) -> String {
        let vars = &self.variables;
        let mut out = String::new();

        let direction = match self.optimization_type {
            OptimizationType::Max => "Maximize",
            OptimizationType::Min | OptimizationType::Satisfy => "Minimize",
        };
        out.push_str(direction);
        out.push('\n');
        let mut objective = lp_terms(self.objective.iter().copied().enumerate(), vars);
        if !self.objective_offset.is_zero() {
            let sign = if self.objective_offset < 0.0 {
                "-"
            } else {
                "+"
            };
            objective.push_str(&format!(
                " {} {}",
                sign,
                lp_num(self.objective_offset.abs())
            ));
        }
        out.push_str(&format!(" obj: {}\n", objective));

        out.push_str("Subject To\n");
        for (i, c) in self.constraints.iter().enumerate() {
            let name = if c.name().is_empty() {
                format!("c{}", i + 1)
            } else {
                c.name()
            };
            let op = match c.constraint_type() {
                Comparison::LessOrEqual | Comparison::Less => "<=",
                Comparison::GreaterOrEqual | Comparison::Greater => ">=",
                Comparison::Equal => "=",
            };
            out.push_str(&format!(
                " {}: {} {} {}\n",
                name,
                lp_terms(c.terms(), vars),
                op,
                lp_num(c.rhs())
            ));
        }

        let mut bounds: Vec<String> = Vec::new();
        let mut binaries: Vec<String> = Vec::new();
        let mut generals: Vec<String> = Vec::new();
        for (name, dv) in self.domain.iter() {
            match dv.get_type() {
                VariableType::Boolean => binaries.push(name.clone()),
                VariableType::IntegerRange(min, max) => {
                    generals.push(name.clone());
                    bounds.push(format!(" {} <= {} <= {}", min, name, max));
                }
                VariableType::NonNegativeReal(min, max) => {
                    if !(*min == 0.0 && *max == f64::INFINITY) {
                        bounds.push(format!(
                            " {} <= {} <= {}",
                            lp_bound(*min),
                            name,
                            lp_bound(*max)
                        ));
                    }
                }
                VariableType::Real(min, max) => {
                    if *min == f64::NEG_INFINITY && *max == f64::INFINITY {
                        bounds.push(format!(" {} free", name));
                    } else {
                        bounds.push(format!(
                            " {} <= {} <= {}",
                            lp_bound(*min),
                            name,
                            lp_bound(*max)
                        ));
                    }
                }
            }
        }
        if !bounds.is_empty() {
            out.push_str("Bounds\n");
            for b in &bounds {
                out.push_str(b);
                out.push('\n');
            }
        }
        if !binaries.is_empty() {
            out.push_str("Binary\n");
            out.push_str(&format!(" {}\n", binaries.join(" ")));
        }
        if !generals.is_empty() {
            out.push_str("General\n");
            out.push_str(&format!(" {}\n", generals.join(" ")));
        }
        out.push_str("End\n");
        out
    }
}

/// Formats a number for LP output. `Display` already renders `2.0` as `2` and
/// `2.5` as `2.5`.
fn lp_num(n: f64) -> String {
    format!("{}", n)
}

/// Formats a bound, mapping infinities to LP's `infinity` keyword.
fn lp_bound(n: f64) -> String {
    if n == f64::INFINITY {
        "+infinity".to_string()
    } else if n == f64::NEG_INFINITY {
        "-infinity".to_string()
    } else {
        lp_num(n)
    }
}

/// Formats a linear combination `sum(coeff_i * var_i)` from `(variable index,
/// coefficient)` pairs, omitting zero terms and unit coefficients (so `1 x`
/// renders as `x`).
fn lp_terms(terms: impl Iterator<Item = (usize, f64)>, vars: &[String]) -> String {
    let mut s = String::new();
    for (i, c) in terms {
        if c.is_zero() {
            continue;
        }
        let v = &vars[i];
        let magnitude = c.abs();
        let coeff = if magnitude == 1.0 {
            String::new()
        } else {
            format!("{} ", lp_num(magnitude))
        };
        if s.is_empty() {
            if c < 0.0 {
                s.push_str(&format!("- {}{}", coeff, v));
            } else {
                s.push_str(&format!("{}{}", coeff, v));
            }
        } else {
            let sign = if c < 0.0 { "-" } else { "+" };
            s.push_str(&format!(" {} {}{}", sign, coeff, v));
        }
    }
    if s.is_empty() {
        s.push('0');
    }
    s
}

impl Display for LinearModel {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let constraints = self.constraints.iter().map(|c| {
            let coefficients = c
                .terms()
                .enumerate()
                .map(|(position, (i, c))| format_var(&self.variables[i], c, position == 0))
                .collect::<Vec<String>>()
                .join(" ");
            let lhs = if coefficients.is_empty() {
                "0".to_string()
            } else {
                coefficients
            };
            let rhs = if c.rhs.is_zero() {
                "0".to_string()
            } else {
                c.rhs.to_string()
            };
            let name = if c.name.is_empty() {
                "".to_string()
            } else {
                format!("{}: ", c.name)
            };
            format!("    {name}{} {} {}", lhs, c.constraint_type, rhs)
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
        let objective = if objective.is_empty() {
            "0".to_string()
        } else {
            objective
        };
        let offset = if self.objective_offset.is_zero() {
            "".to_string()
        } else if float_lt(self.objective_offset, 0.0) {
            format!(" - {}", self.objective_offset.abs())
        } else {
            format!(" + {}", self.objective_offset)
        };
        let objective = format!("{}{}", objective, offset);
        let domain: String = if !self.domain.is_empty() {
            format!(
                "\ndefine\n    {}",
                format_domain(&self.domain)
                    .split("\n")
                    .collect::<Vec<_>>()
                    .join("\n    ")
            )
        } else {
            "".to_string()
        };
        let objective = match self.optimization_type {
            //a satisfaction problem has no objective expression
            OptimizationType::Satisfy => self.optimization_type.to_string(),
            OptimizationType::Min | OptimizationType::Max => {
                format!("{} {}", self.optimization_type, objective)
            }
        };
        write!(f, "{}\ns.t.\n{}{}", objective, constraints, domain)
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

pub fn make_constraints_map_from_assignment(
    model: &LinearModel,
    assignment: &[f64],
) -> IndexMap<String, f64> {
    let constraints = model.calc_constraints(assignment);
    // Remove constraints that are not user-defined, marked as "__"
    constraints
        .into_iter()
        .filter(|c| !c.0.starts_with("__"))
        .collect()
}

#[cfg(test)]
mod tests {
    use super::LinearConstraint;
    use crate::math::Comparison;
    use serde_json::json;

    #[test]
    fn dense_coefficients_keep_only_the_nonzero_ones() {
        let constraint =
            LinearConstraint::new(vec![0.0, 2.0, 0.0, -1.0], Comparison::LessOrEqual, 3.0);
        assert_eq!(constraint.indices(), &[1, 3]);
        assert_eq!(constraint.values(), &[2.0, -1.0]);
        assert_eq!(constraint.coefficient(1), 2.0);
        assert_eq!(constraint.coefficient(2), 0.0);
        assert_eq!(constraint.to_dense(5), vec![0.0, 2.0, 0.0, -1.0, 0.0]);
    }

    #[test]
    fn sparse_coefficients_are_sorted_and_merged() {
        let terms = [(4, 1.0), (1, 2.0), (4, 2.0), (2, 1.0), (2, -1.0)];
        let constraint = LinearConstraint::new_sparse(terms, Comparison::Equal, 0.0);
        // the two coefficients of variable 2 cancel out
        assert_eq!(constraint.indices(), &[1, 4]);
        assert_eq!(constraint.values(), &[2.0, 3.0]);
    }

    #[test]
    fn removing_variables_moves_the_following_ones_down() {
        let mut constraint =
            LinearConstraint::new(vec![1.0, 2.0, 3.0, 4.0, 5.0], Comparison::Equal, 0.0);
        constraint.remove_coefficients_by_index(&[3, 1]);
        assert_eq!(constraint.to_dense(3), vec![1.0, 3.0, 5.0]);
    }

    #[test]
    fn serialized_constraints_list_only_the_nonzero_coefficients() {
        let constraint =
            LinearConstraint::new(vec![0.0, 2.0, 0.0], Comparison::LessOrEqual, 3.0).with_name("c");
        let serialized = serde_json::to_value(&constraint).unwrap();
        assert_eq!(
            serialized,
            json!({
                "name": "c",
                "indices": [1],
                "values": [2.0],
                "rhs": 3.0,
                "constraint_type": { "type": "LessOrEqual" }
            })
        );
    }

    #[test]
    fn deserialized_constraints_are_normalized() {
        let constraint: LinearConstraint = serde_json::from_value(json!({
            "name": "c",
            "indices": [3, 0, 3],
            "values": [1.0, 0.0, 2.0],
            "rhs": 1.0,
            "constraint_type": { "type": "Equal" }
        }))
        .unwrap();
        assert_eq!(constraint.indices(), &[3]);
        assert_eq!(constraint.values(), &[3.0]);

        let mismatched = serde_json::from_value::<LinearConstraint>(json!({
            "name": "c",
            "indices": [0, 1],
            "values": [1.0],
            "rhs": 1.0,
            "constraint_type": { "type": "Equal" }
        }));
        assert!(mismatched.is_err());
    }
}
