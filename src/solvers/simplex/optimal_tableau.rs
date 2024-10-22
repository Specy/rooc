use crate::solvers::{LpSolution, Tableau};
use core::fmt;
use std::fmt::Display;
use wasm_bindgen::prelude::wasm_bindgen;

#[derive(Debug, Clone)]
#[wasm_bindgen]
pub struct OptimalTableau {
    flip_result: bool,
    values: Vec<f64>,
    tableau: Tableau,
}

impl OptimalTableau {
    pub(crate) fn new(values: Vec<f64>, tableau: Tableau) -> OptimalTableau {
        OptimalTableau {
            values,
            flip_result: tableau.flip_result(),
            tableau,
        }
    }

    pub fn get_variables_values(&self) -> &Vec<f64> {
        &self.values
    }
    pub fn get_optimal_value(&self) -> f64 {
        let flip = if self.flip_result { -1.0 } else { 1.0 };
        ((self.tableau.get_current_value() + self.tableau.get_value_offset()) * -1.0) * flip
    }
    pub fn get_tableau(&self) -> &Tableau {
        &self.tableau
    }

    pub fn as_lp_solution(&self) -> LpSolution<f64> {
        let values = self.get_variables_values().clone();
        let value = self.get_optimal_value();
        let assignment = self
            .tableau
            .wasm_get_variables()
            .iter()
            .zip(values.iter())
            .map(|(var, val)| crate::solvers::Assignment {
                name: var.clone(),
                value: *val,
            })
            .collect();
        LpSolution::new(assignment, value)
    }
}

#[wasm_bindgen]
impl OptimalTableau {
    pub fn wasm_get_variables_values(&self) -> Vec<f64> {
        self.values.clone()
    }
    pub fn wasm_get_optimal_value(&self) -> f64 {
        self.get_optimal_value()
    }
    pub fn wasm_get_tableau(&self) -> Tableau {
        self.tableau.clone()
    }
}

impl Display for OptimalTableau {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let tableau = self.tableau.to_string();
        write!(
            f,
            "{}\n\nOptimal Value: {}",
            tableau,
            self.get_optimal_value(),
        )
    }
}

#[derive(Debug, Clone)]
#[wasm_bindgen]
pub struct SimplexStep {
    tableau: Tableau,
    entering: usize,
    leaving: usize,
    #[allow(unused)]
    ratio: f64,
}
impl SimplexStep {
    pub fn new(tableau: Tableau, entering: usize, leaving: usize, ratio: f64) -> SimplexStep {
        SimplexStep {
            tableau,
            entering,
            leaving,
            ratio,
        }
    }
}

#[wasm_bindgen]
impl SimplexStep {
    pub fn wasm_get_tableau(&self) -> Tableau {
        self.tableau.clone()
    }
    pub fn wasm_get_entering(&self) -> usize {
        self.entering
    }
    pub fn wasm_get_leaving(&self) -> usize {
        self.leaving
    }
}

#[derive(Debug, Clone)]
#[wasm_bindgen]
pub struct OptimalTableauWithSteps {
    result: OptimalTableau,
    steps: Vec<SimplexStep>,
}
impl OptimalTableauWithSteps {
    pub fn new(result: OptimalTableau, steps: Vec<SimplexStep>) -> OptimalTableauWithSteps {
        OptimalTableauWithSteps { result, steps }
    }
}

#[wasm_bindgen]
impl OptimalTableauWithSteps {
    pub fn wasm_get_result(&self) -> OptimalTableau {
        self.result.clone()
    }
    pub fn wasm_get_steps(&self) -> Vec<SimplexStep> {
        self.steps.clone()
    }
}
