/*
use std::collections::HashMap;
use std::env::var;
use crate::math::VariableType;
use crate::solvers::{LpSolution, SolverError};
use crate::transformers::{LinearConstraint, LinearModel};
use good_lp::{ProblemVariables, SolverModel, Variable, VariableDefinition, Expression};
use indexmap::IndexMap;

pub fn solve_milp_model(lp: &LinearModel) -> Result<LpSolution<f64>, SolverError> {
    let domain = lp.get_domain();

    let mut variables = ProblemVariables::new();
    let mut created_vars: IndexMap<String, Variable> = IndexMap::new();
    for (name, var) in domain.iter() {
        let def = VariableDefinition::new().name(name);
        let def = match var.get_type() {
            VariableType::Boolean => {
                def.binary()
            }
            VariableType::Real => {
                def.min(f32::MIN).max(f32::MAX)
            }
            VariableType::NonNegativeReal => {
                def.min(0.0).max(f32::MAX)
            }
            VariableType::IntegerRange(min, max) => {
                def.min(*min as f32).max(*max as f32).integer()
            }
        };
        let var = variables.add(def);
        created_vars.insert(name.clone(), var);
    }
    let vars = lp.get_variables();
    for constraint in lp.get_constraints(){
        let mut good_lp_constraint = Expression::with_capacity(vars.len());
        for (i,c) in constraint.get_coefficients().iter().enumerate() {
            let name = &vars[i];
            let existing = created_vars.get(name).unwrap().clone();
            good_lp_constraint = good_lp_constraint + (*c) * existing;
        }
    }
    
    panic!()
}

 */