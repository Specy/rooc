#[allow(unused_imports)]
use crate::prelude::*;
use crate::solvers::{LpSolution, Tableau};
use core::fmt;
use indexmap::IndexMap;
use std::fmt::Display;

#[derive(Debug, Clone)]
#[cfg_attr(target_arch = "wasm32", wasm_bindgen)]
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

    pub fn variables_values(&self) -> &Vec<f64> {
        &self.values
    }
    pub fn optimal_value(&self) -> f64 {
        //the offset is in the original problem's frame, it must not be
        //negated together with the tableau value when the objective was flipped
        let flip = if self.flip_result { -1.0 } else { 1.0 };
        -self.tableau.current_value() * flip + self.tableau.value_offset()
    }
    pub fn tableau(&self) -> &Tableau {
        &self.tableau
    }

    pub fn as_lp_solution(&self) -> LpSolution<f64> {
        let values = self.variables_values().clone();
        let value = self.optimal_value();
        let names = self.tableau.variables();
        // Map standard-form variables back to the original model's variables:
        // recombine a free variable's split `x = $px - $mx`, and drop the internal
        // slack/surplus/artificial variables the standardizer/two-phase added.
        let map: IndexMap<String, f64> = names
            .iter()
            .cloned()
            .zip(values.iter().cloned())
            .collect();
        let mut assignment = Vec::new();
        for (name, val) in names.iter().zip(values.iter()) {
            if name.starts_with("$su_") || name.starts_with("$sl_") || name.starts_with("$a_") {
                continue; // internal slack/surplus/artificial variable
            }
            // Negative half of a free-variable split: already accounted for by the positive half.
            if let Some(rest) = name.strip_prefix("$m") {
                if map.contains_key(&format!("$p{}", rest)) {
                    continue;
                }
            }
            // Positive half of a free-variable split: reconstruct the original variable.
            if let Some(rest) = name.strip_prefix("$p") {
                if let Some(minus) = map.get(&format!("$m{}", rest)) {
                    assignment.push(crate::solvers::Assignment {
                        name: rest.to_string(),
                        value: *val - *minus,
                    });
                    continue;
                }
            }
            assignment.push(crate::solvers::Assignment {
                name: name.clone(),
                value: *val,
            });
        }
        LpSolution::new(assignment, value, IndexMap::new())
    }
}

#[cfg_attr(target_arch = "wasm32", wasm_bindgen)]
#[cfg(target_arch = "wasm32")]
impl OptimalTableau {
    pub fn wasm_get_variables_values(&self) -> Vec<f64> {
        self.values.clone()
    }
    pub fn wasm_get_optimal_value(&self) -> f64 {
        self.optimal_value()
    }
    pub fn wasm_get_tableau(&self) -> Tableau {
        self.tableau.clone()
    }
}

impl Display for OptimalTableau {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let tableau = self.tableau.to_string();
        write!(f, "{}\n\nOptimal Value: {}", tableau, self.optimal_value())
    }
}

#[derive(Debug, Clone)]
#[cfg_attr(target_arch = "wasm32", wasm_bindgen)]
pub struct SimplexStep {
    #[allow(unused)]
    tableau: Tableau,
    #[allow(unused)]
    entering: usize,
    #[allow(unused)]
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

impl Display for SimplexStep {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let tableau = self.tableau.to_string();
        write!(f, "{}", tableau)
    }
}

#[cfg_attr(target_arch = "wasm32", wasm_bindgen)]
#[cfg(target_arch = "wasm32")]
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
#[cfg_attr(target_arch = "wasm32", wasm_bindgen)]
pub struct OptimalTableauWithSteps {
    result: OptimalTableau,
    steps: Vec<SimplexStep>,
}
impl OptimalTableauWithSteps {
    pub fn new(result: OptimalTableau, steps: Vec<SimplexStep>) -> OptimalTableauWithSteps {
        OptimalTableauWithSteps { result, steps }
    }
    pub fn result(&self) -> &OptimalTableau {
        &self.result
    }
    pub fn steps(&self) -> &Vec<SimplexStep> {
        &self.steps
    }
}

impl Display for OptimalTableauWithSteps {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{}\n{}",
            self.steps
                .iter()
                .map(|v| v.to_string())
                .collect::<Vec<_>>()
                .join("\n"),
            self.result
        )
    }
}

#[cfg_attr(target_arch = "wasm32", wasm_bindgen)]
#[cfg(target_arch = "wasm32")]
impl OptimalTableauWithSteps {
    pub fn wasm_get_result(&self) -> OptimalTableau {
        self.result.clone()
    }
    pub fn wasm_get_steps(&self) -> Vec<SimplexStep> {
        self.steps.clone()
    }
}
