use crate::math::{Comparison, OptimizationType, VariableType};
use crate::solvers::{find_invalid_variables, Assignment, LpSolution, SolverError};
use crate::transformers::LinearModel;
use good_lp::clarabel;
use good_lp::solvers::ObjectiveDirection;
use good_lp::{
    Expression, ProblemVariables, ResolutionError, Solution, SolverModel, Variable,
    VariableDefinition,
};
use indexmap::IndexMap;

pub fn solve_real_lp_problem_clarabel(lp: &LinearModel) -> Result<LpSolution<f64>, SolverError> {
    let domain = lp.domain();
    let invalid_variables = find_invalid_variables(domain, |var| {
        matches!(var, VariableType::Real | VariableType::NonNegativeReal)
    });
    if !invalid_variables.is_empty() {
        return Err(SolverError::InvalidDomain {
            expected: vec![VariableType::Real, VariableType::NonNegativeReal],
            got: invalid_variables,
        });
    }
    let opt_type = match lp.optimization_type() {
        OptimizationType::Min => ObjectiveDirection::Minimisation,
        OptimizationType::Max => ObjectiveDirection::Maximisation,
        OptimizationType::Satisfy => ObjectiveDirection::Minimisation,
    };
    let mut variables = ProblemVariables::new();
    let mut created_vars: IndexMap<String, Variable> = IndexMap::new();
    for (name, var) in domain.iter() {
        let def = VariableDefinition::new().name(name);
        let def = match var.get_type() {
            VariableType::Real => def.min(f32::MIN).max(f32::MAX),
            VariableType::NonNegativeReal => def.min(0.0).max(f32::MAX),
            _ => panic!(),
        };
        let var = variables.add(def);
        created_vars.insert(name.clone(), var);
    }
    let vars = lp.variables();
    let obj_exp = match lp.optimization_type() {
        OptimizationType::Satisfy => 0.into(),
        OptimizationType::Max | OptimizationType::Min => vars.iter().zip(lp.objective()).fold(
            Expression::from(lp.objective_offset()),
            |acc, (name, coeff)| {
                let var = created_vars.get(name).unwrap();
                acc + (*coeff) * (*var)
            },
        ),
    };
    let objective = variables.optimise(opt_type, obj_exp.clone());
    let mut model = objective.using(clarabel);
    for constraint in lp.constraints() {
        let mut good_lp_constraint = Expression::with_capacity(vars.len());
        for (i, c) in constraint.coefficients().iter().enumerate() {
            let name = &vars[i];
            let existing = created_vars.get(name).unwrap().clone();
            good_lp_constraint += (*c) * existing;
        }
        let constraint = match constraint.constraint_type() {
            Comparison::LessOrEqual => good_lp_constraint.leq(constraint.rhs()),
            Comparison::GreaterOrEqual => good_lp_constraint.geq(constraint.rhs()),
            Comparison::Equal => good_lp_constraint.eq(constraint.rhs()),
            c => {
                return Err(SolverError::UnavailableComparison {
                    got: *c,
                    expected: vec![
                        Comparison::LessOrEqual,
                        Comparison::GreaterOrEqual,
                        Comparison::Equal,
                    ],
                })
            }
        };
        model = model.with(constraint);
    }
    let solution = model.solve();
    match solution {
        Ok(sol) => {
            let vars = vars
                .iter()
                .map(|name| {
                    let var = created_vars.get(name).unwrap();
                    Assignment {
                        name: name.clone(),
                        value: sol.value(*var),
                    }
                })
                .collect::<Vec<Assignment<f64>>>();
            let coeffs = lp.objective();
            //good_lp does not provide a way to get the objective value
            let value = vars
                .iter()
                .enumerate()
                .fold(lp.objective_offset(), |acc, (i, a)| {
                    acc + a.value * coeffs[i]
                });
            Ok(LpSolution::new(vars, value + lp.objective_offset()))
        }
        Err(e) => match e {
            ResolutionError::Unbounded => Err(SolverError::Unbounded),
            ResolutionError::Infeasible => Err(SolverError::Infisible),
            ResolutionError::Other(s) => Err(SolverError::Other(s.to_string())),
            ResolutionError::Str(s) => Err(SolverError::Other(s)),
        },
    }
}
