use crate::math::{float_ge, float_gt, float_le, float_lt};
#[allow(unused_imports)]
use crate::prelude::*;
use crate::solvers::{
    FractionalTableau, OptimalTableau, OptimalTableauWithSteps, SimplexError, SimplexStep,
    StepAction,
};
use core::fmt;
use std::fmt::Display;
use term_table::row::Row;
use term_table::table_cell::TableCell;
use term_table::Table;

#[derive(Debug, Clone)]
#[cfg_attr(target_arch = "wasm32", wasm_bindgen)]
pub struct Tableau {
    flip_result: bool,
    variables: Vec<String>,
    c: Vec<f64>,
    a: Vec<Vec<f64>>,
    b: Vec<f64>,
    in_basis: Vec<usize>,
    current_value: f64,
    value_offset: f64,
}

impl Display for Tableau {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let pretty = FractionalTableau::new(self.clone());
        let table = pretty.pretty_table();
        let mut cli_table = Table::new();
        let vars: Vec<String> = self
            .variables
            .iter()
            .zip(self.c.iter())
            .map(|(v, c)| format!("{}: {}", v, c))
            .collect();
        let header = Row::new(vars.iter().map(TableCell::new));
        cli_table.add_row(header);
        let empty: Vec<TableCell> = Vec::new();
        cli_table.add_row(Row::new(empty));
        table.iter().for_each(|row| {
            cli_table.add_row(Row::new(row.iter().map(TableCell::new)));
        });
        write!(f, "{}", cli_table.render())
    }
}

#[cfg_attr(target_arch = "wasm32", wasm_bindgen)]
#[cfg(target_arch = "wasm32")]
impl Tableau {
    pub fn wasm_get_variables(&self) -> Vec<String> {
        self.variables().to_owned()
    }
    pub fn wasm_get_c(&self) -> Vec<f64> {
        self.c.clone()
    }
    pub fn wasm_get_a(&self) -> JsValue {
        serde_wasm_bindgen::to_value(&self.a).unwrap()
    }
    pub fn wasm_get_b(&self) -> Vec<f64> {
        self.b.clone()
    }
    pub fn wasm_get_in_basis(&self) -> Vec<usize> {
        self.in_basis.clone()
    }
    pub fn wasm_get_current_value(&self) -> f64 {
        self.current_value
    }
    pub fn wasm_get_value_offset(&self) -> f64 {
        self.value_offset
    }

    pub fn wasm_step(&mut self, variables_to_avoid: Vec<usize>) -> Result<JsValue, SimplexError> {
        self.step(&variables_to_avoid)
            .map(|action| serde_wasm_bindgen::to_value(&action).unwrap())
    }
    pub fn wasm_to_string(&self) -> String {
        self.to_string()
    }
}

impl Tableau {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        c: Vec<f64>,
        a: Vec<Vec<f64>>,
        b: Vec<f64>,
        in_basis: Vec<usize>,
        current_value: f64,
        value_offset: f64,
        variables: Vec<String>,
        flip_result: bool,
    ) -> Tableau {
        Tableau {
            c,
            a,
            b,
            in_basis,
            current_value,
            value_offset,
            variables,
            flip_result,
        }
    }

    pub fn flip_result(&self) -> bool {
        self.flip_result
    }
    pub fn solve(&mut self, limit: i64) -> Result<OptimalTableau, SimplexError> {
        self.solve_avoiding(limit, &[])
    }
    pub fn variables(&self) -> &Vec<String> {
        &self.variables
    }

    pub fn solve_step_by_step(
        &mut self,
        limit: i64,
    ) -> Result<OptimalTableauWithSteps, SimplexError> {
        let mut iteration = 0;
        let empty = vec![];
        let mut steps = vec![];
        while iteration <= limit {
            let prev = self.clone();
            match self.step(&empty) {
                Ok(StepAction::Pivot {
                    entering,
                    leaving,
                    ratio,
                }) => {
                    iteration += 1;
                    steps.push(SimplexStep::new(prev, entering, leaving, ratio));
                }
                Ok(StepAction::Finished) => {
                    return Ok(OptimalTableauWithSteps::new(
                        OptimalTableau::new(self.variables_values(), self.clone()),
                        steps,
                    ));
                }
                Err(e) => {
                    return Err(e);
                }
            }
        }
        Err(SimplexError::IterationLimitReached)
    }

    pub fn solve_avoiding(
        &mut self,
        limit: i64,
        variables_to_avoid: &[usize],
    ) -> Result<OptimalTableau, SimplexError> {
        let mut iteration = 0;
        while iteration <= limit {
            match self.step(variables_to_avoid) {
                Ok(StepAction::Pivot { .. }) => {
                    iteration += 1;
                }
                Ok(StepAction::Finished) => {
                    return Ok(OptimalTableau::new(self.variables_values(), self.clone()));
                }
                Err(e) => {
                    return Err(e);
                }
            }
        }
        Err(SimplexError::IterationLimitReached)
    }
    pub fn step(&mut self, variables_to_avoid: &[usize]) -> Result<StepAction, SimplexError> {
        if self.is_optimal() {
            return Ok(StepAction::Finished);
        }
        match self.find_h(variables_to_avoid) {
            None => Err(SimplexError::Unbounded),
            Some(h) => {
                let t = self.find_t(h, variables_to_avoid);
                match t {
                    None => Err(SimplexError::Unbounded),
                    Some((t, ratio)) => match self.pivot(t, h) {
                        Ok(()) => Ok(StepAction::Pivot {
                            entering: h,
                            leaving: t,
                            ratio,
                        }),
                        Err(_) => Err(SimplexError::Other),
                    },
                }
            }
        }
    }

    fn is_optimal(&self) -> bool {
        self.c.iter().all(|c| float_ge(*c, 0.0))
    }

    #[allow(unused)]
    fn is_unbounded(&self, h: usize) -> bool {
        self.a.iter().all(|a| float_le(a[h], 0.0))
    }

    //finds the variable that will enter the basis
    #[allow(unused)]
    fn find_h(&self, variables_to_avoid: &[usize]) -> Option<usize> {
        //uses the Bland's rule for anti-cycling
        let min = self
            .c
            .iter()
            .enumerate()
            .filter(|(i, c)| !self.in_basis.contains(i) && float_lt(**c, 0.0))
            .min_by(|(_, c1), (_, c2)| c1.partial_cmp(c2).unwrap());
        min.map(|(i, _)| i)
    }

    //finds the variable that will leave the basis, prioritize variabls_to_prefer
    fn find_t(&self, h: usize, variables_to_prefer: &[usize]) -> Option<(usize, f64)> {
        //use the Bland's rule for anti-cycling
        //gets the index of the row with the minimum ratio
        let mut valid = self
            .a
            .iter()
            .enumerate()
            .filter(|(_, a)| float_gt(a[h], 0.0))
            .map(|(i, a)| (i, self.b[i] / a[h]));
        let basis = &self.in_basis;
        match valid.next() {
            Some(first) => {
                let mut min = first;
                for (i, ratio) in valid {
                    if ratio == min.1 {
                        //if we found a tie, we use the Bland's rule for anti-cycling, but prefer to prioritize some variables
                        let to_prefer = variables_to_prefer.contains(&basis[i])
                            && !variables_to_prefer.contains(&basis[min.0]);
                        if basis[i] < basis[min.0] || to_prefer {
                            min = (i, ratio);
                        }
                    } else if ratio < min.1 {
                        min = (i, ratio);
                    }
                }
                Some(min)
            }
            None => None,
        }
    }

    fn variables_values(&self) -> Vec<f64> {
        let mut values = vec![0.0; self.c.len()];
        for (i, &j) in self.in_basis.iter().enumerate() {
            values[j] = self.b[i];
        }
        values
    }
    //performs the pivot operation where variable h enters the basis and variable B(t) leaves the basis
    fn pivot(&mut self, t: usize, h: usize) -> Result<(), ()> {
        let in_basis = &mut self.in_basis;
        let a = &mut self.a;
        let b = &mut self.b;
        let c = &mut self.c;
        let pivot = a[t][h];

        //normalize the pivot column
        for i in 0..a.len() {
            if i != t {
                let factor = a[i][h] / pivot;
                for j in 0..a[i].len() {
                    a[i][j] -= factor * a[t][j];
                }
                b[i] -= factor * b[t];
            }
        }
        //normalize the objective function
        let factor = c[h] / pivot;
        for (i, row) in c.iter_mut().enumerate() {
            *row -= factor * a[t][i];
        }
        self.current_value -= factor * b[t];
        //normalize the pivot row
        for i in 0..a[t].len() {
            a[t][i] /= pivot;
        }
        //normalize the pivot's row value
        b[t] /= pivot;
        //update the basis
        in_basis[t] = h;
        Ok(())
    }
    pub fn current_value(&self) -> f64 {
        self.current_value
    }
    pub fn value_offset(&self) -> f64 {
        self.value_offset
    }
    pub fn a_matrix(&self) -> &Vec<Vec<f64>> {
        &self.a
    }
    pub fn b_vec(&self) -> &Vec<f64> {
        &self.b
    }
    pub fn c_vec(&self) -> &Vec<f64> {
        &self.c
    }
    pub fn in_basis(&self) -> &Vec<usize> {
        &self.in_basis
    }
}
