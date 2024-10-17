use num_traits::Zero;
use std::collections::HashMap;
use std::fmt::Display;

use wasm_bindgen::prelude::wasm_bindgen;

use crate::math::math_utils::float_lt;
use crate::parser::model_transformer::transformer_context::DomainVariable;
use crate::transformers::standard_linear_model::{format_var, StandardLinearModel};
use crate::{
    math::math_enums::{Comparison, OptimizationType},
    transformers::standardizer::to_standard_form,
};
use crate::utils::remove_many;

#[derive(Debug, Clone)]
#[wasm_bindgen]
pub struct LinearConstraint {
    coefficients: Vec<f64>,
    rhs: f64,
    constraint_type: Comparison,
}

impl LinearConstraint {
    pub fn new(coefficients: Vec<f64>, constraint_type: Comparison, rhs: f64) -> LinearConstraint {
        LinearConstraint {
            coefficients,
            rhs,
            constraint_type,
        }
    }
    pub fn get_coefficients(&self) -> &Vec<f64> {
        &self.coefficients
    }
    pub fn get_coefficients_mut(&mut self) -> &mut Vec<f64> {
        &mut self.coefficients
    }
    pub fn remove_coefficients_by_index(&mut self, indices: &[usize]) {
        remove_many(&mut self.coefficients, indices);
    }
    pub fn get_rhs(&self) -> f64 {
        self.rhs
    }
    pub fn get_constraint_type(&self) -> &Comparison {
        &self.constraint_type
    }
    pub fn into_parts(self) -> (Vec<f64>, Comparison, f64) {
        (self.coefficients, self.constraint_type, self.rhs)
    }
    pub fn ensure_size(&mut self, size: usize) {
        self.coefficients.resize(size, 0.0);
    }
}

#[wasm_bindgen]
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

#[derive(Debug, Clone)]
#[wasm_bindgen]
pub struct LinearModel {
    variables: Vec<String>,
    domain: HashMap<String, DomainVariable>,
    objective_offset: f64,
    optimization_type: OptimizationType,
    objective: Vec<f64>,
    constraints: Vec<LinearConstraint>,
}

impl LinearModel {
    pub fn new(
        objective: Vec<f64>,
        optimization_type: OptimizationType,
        objective_offset: f64,
        constraints: Vec<LinearConstraint>,
        variables: Vec<String>,
        domain: HashMap<String, DomainVariable>,
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

    pub fn into_parts(
        self,
    ) -> (
        Vec<f64>,
        OptimizationType,
        f64,
        Vec<LinearConstraint>,
        Vec<String>,
        HashMap<String, DomainVariable>,
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
    pub fn get_optimization_type(&self) -> &OptimizationType {
        &self.optimization_type
    }
    pub fn into_standard_form(self) -> Result<StandardLinearModel, ()> {
        to_standard_form(self)
    }
    pub fn get_objective(&self) -> &Vec<f64> {
        &self.objective
    }
    pub fn get_constraints(&self) -> &Vec<LinearConstraint> {
        &self.constraints
    }
    pub fn get_variables(&self) -> &Vec<String> {
        &self.variables
    }
    pub fn get_objective_offset(&self) -> f64 {
        self.objective_offset
    }
    pub fn get_domain(&self) -> &HashMap<String, DomainVariable> {
        &self.domain
    }

    pub fn into_dual(self) -> LinearModel {
        todo!()
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

#[wasm_bindgen]
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
