use crate::{
    math::math_enums::{Comparison, OptimizationType},
    transformers::standardizer::to_standard_form,
};
use crate::transformers::standard_linear_model::StandardLinearModel;

pub struct Constraint {
    coefficients: Vec<f64>,
    rhs: f64,
    constraint_type: Comparison,
}

impl Constraint {
    pub fn new(coefficients: Vec<f64>, constraint_type: Comparison, rhs: f64) -> Constraint {
        Constraint {
            coefficients,
            rhs,
            constraint_type,
        }
    }
    pub fn get_coefficients(&self) -> &Vec<f64> {
        &self.coefficients
    }
    pub fn get_rhs(&self) -> f64 {
        self.rhs
    }
    pub fn get_constraint_type(&self) -> &Comparison {
        &self.constraint_type
    }
    pub fn ensure_size(&mut self, size: usize) {
        self.coefficients.resize(size, 0.0);
    }
}

pub struct LinearModel {
    variables: Vec<String>,
    objective_offset: f64,
    optimization_type: OptimizationType,
    objective: Vec<f64>,
    constraints: Vec<Constraint>,
}

impl LinearModel {
    pub fn new(
        objective: Vec<f64>,
        optimization_type: OptimizationType,
        objective_offset: f64,
        constraints: Vec<Constraint>,
        variables: Vec<String>,
    ) -> LinearModel {
        LinearModel {
            objective,
            constraints,
            optimization_type,
            variables,
            objective_offset,
        }
    }

    pub fn get_optimization_type(&self) -> &OptimizationType {
        &self.optimization_type
    }
    pub fn into_standard_form(self) -> Result<StandardLinearModel, ()> {
        to_standard_form(&self)
    }
    pub fn get_objective(&self) -> &Vec<f64> {
        &self.objective
    }
    pub fn get_constraints(&self) -> &Vec<Constraint> {
        &self.constraints
    }
    pub fn get_variables(&self) -> &Vec<String> {
        &self.variables
    }
    pub fn get_objective_offset(&self) -> f64 {
        self.objective_offset
    }
}
