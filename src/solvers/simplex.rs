use core::fmt;
use std::fmt::Display;

//TODO make the implementation use row vectors with a trait so that i can implement fraction and float versions
//togehter with overriding the operators, so that i can use the same code for both versions
use num_rational::Rational64;
use num_traits::cast::FromPrimitive;
use term_table::row::Row;
use term_table::table_cell::TableCell;
use term_table::Table;
use wasm_bindgen::prelude::wasm_bindgen;
use wasm_bindgen::JsValue;

#[derive(Debug, Clone)]
#[wasm_bindgen]
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
        let pretty = self.clone().to_fractional_tableau();
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

#[wasm_bindgen]
impl Tableau {
    pub fn wasm_get_variables(&self) -> Vec<String> {
        self.variables.clone()
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

    pub fn wasm_step(&mut self, variables_to_avoid: Vec<usize>) -> Result<bool, SimplexError> {
        self.step(&variables_to_avoid)
    }
    pub fn wasm_to_string(&self) -> String {
        self.to_string()
    }
}

#[derive(Debug, Clone)]
#[wasm_bindgen]
pub struct OptimalTableau {
    flip_result: bool,
    values: Vec<f64>,
    tableau: Tableau,
}

impl OptimalTableau {
    fn new(values: Vec<f64>, tableau: Tableau) -> OptimalTableau {
        OptimalTableau {
            values,
            flip_result: tableau.flip_result,
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

#[derive(Debug)]
#[wasm_bindgen]
pub enum SimplexError {
    Unbounded,
    IterationLimitReached,
    Other,
}
impl Display for SimplexError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let s = match self {
            SimplexError::Unbounded => "Unbounded Problem",
            SimplexError::IterationLimitReached => "Iteration Limit Reached",
            SimplexError::Other => "Other",
        };
        f.write_str(s)
    }
}

impl Tableau {
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
    pub fn solve(&mut self, limit: i64) -> Result<OptimalTableau, SimplexError> {
        self.solve_avoiding(limit, &vec![])
    }

    pub fn solve_avoiding(
        &mut self,
        limit: i64,
        variables_to_avoid: &Vec<usize>,
    ) -> Result<OptimalTableau, SimplexError> {
        let mut iteration = 0;
        while iteration <= limit {
            match self.step(variables_to_avoid) {
                Ok(false) => {
                    iteration += 1;
                }
                Ok(true) => {
                    return Ok(OptimalTableau::new(
                        self.get_variables_values(),
                        self.clone(),
                    ));
                }
                Err(e) => {
                    return Err(e);
                }
            }
        }
        Err(SimplexError::IterationLimitReached)
    }
    pub fn step(&mut self, variables_to_avoid: &Vec<usize>) -> Result<bool, SimplexError> {
        if self.is_optimal() {
            return Ok(true);
        }
        match self.find_h(variables_to_avoid) {
            None => Err(SimplexError::Unbounded),
            Some(h) => {
                let t = self.find_t(h, variables_to_avoid);
                match t {
                    None => Err(SimplexError::Unbounded),
                    Some(t) => match self.pivot(t, h) {
                        Ok(()) => Ok(false),
                        Err(_) => Err(SimplexError::Other),
                    },
                }
            }
        }
    }

    fn is_optimal(&self) -> bool {
        self.c.iter().all(|c| *c >= 0.0)
    }

    #[allow(unused)]
    fn is_unbounded(&self, h: usize) -> bool {
        self.a.iter().all(|a| a[h] <= 0.0)
    }

    //finds the variable that will enter the basis
    #[allow(unused)]
    fn find_h(&self, variables_to_avoid: &Vec<usize>) -> Option<usize> {
        //uses the Bland's rule for anti-cycling
        let min = self
            .c
            .iter()
            .enumerate()
            .filter(|(i, c)| !self.in_basis.contains(i) && **c < 0.0)
            .min_by(|(_, c1), (_, c2)| c1.partial_cmp(c2).unwrap());
        min.map(|(i, _)| i)
    }

    //finds the variable that will leave the basis, prioritize variabls_to_prefer
    fn find_t(&self, h: usize, variables_to_prefer: &Vec<usize>) -> Option<usize> {
        //use the Bland's rule for anti-cycling
        //gets the index of the row with the minimum ratio
        let mut valid = self
            .a
            .iter()
            .enumerate()
            .filter(|(_, a)| a[h] > 0.0)
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
                Some(min.0)
            }
            None => None,
        }
    }

    fn get_variables_values(&self) -> Vec<f64> {
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
        for i in 0..c.len() {
            c[i] -= factor * a[t][i];
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
    pub fn get_current_value(&self) -> f64 {
        self.current_value
    }
    pub fn get_value_offset(&self) -> f64 {
        self.value_offset
    }
    pub fn to_fractional_tableau(self) -> FractionalTableau {
        FractionalTableau::new(self)
    }
    pub fn get_a(&self) -> &Vec<Vec<f64>> {
        &self.a
    }
    pub fn get_b(&self) -> &Vec<f64> {
        &self.b
    }
    pub fn get_c(&self) -> &Vec<f64> {
        &self.c
    }
    pub fn get_in_basis(&self) -> &Vec<usize> {
        &self.in_basis
    }
}

pub struct PrettyFraction {
    numerator: i64,
    denominator: i64,
}

impl PrettyFraction {
    fn new(num: f64) -> PrettyFraction {
        //TODO make it use precision for smaller numbers
        let f = Rational64::from_f64(num).unwrap();

        PrettyFraction {
            numerator: *f.numer(),
            denominator: *f.denom(),
        }
    }
    #[allow(unused)]
    fn to_f64(&self) -> f64 {
        self.numerator as f64 / self.denominator as f64
    }
    fn pretty(&self) -> String {
        match self.denominator {
            1 => format!("{}", self.numerator),
            _ => format!("{}/{}", self.numerator, self.denominator),
        }
    }
}

pub struct FractionalTableau {
    c: Vec<PrettyFraction>,
    a: Vec<Vec<PrettyFraction>>,
    b: Vec<PrettyFraction>,
    #[allow(unused)]
    in_basis: Vec<usize>,
    value: f64,
}

impl FractionalTableau {
    pub fn new(tableau: Tableau) -> FractionalTableau {
        FractionalTableau {
            c: tableau.c.iter().map(|&c| PrettyFraction::new(c)).collect(),
            a: tableau
                .a
                .iter()
                .map(|a| a.iter().map(|&a| PrettyFraction::new(a)).collect())
                .collect(),
            b: tableau.b.iter().map(|&b| PrettyFraction::new(b)).collect(),
            in_basis: tableau.in_basis.clone(),
            value: tableau.get_current_value(),
        }
    }
    pub fn pretty_table(&self) -> Vec<Vec<String>> {
        let mut header: Vec<String> = self.c.iter().map(|c| c.pretty()).collect();
        let a: Vec<Vec<String>> = self
            .a
            .iter()
            .map(|a| a.iter().map(|a| a.pretty()).collect())
            .collect();
        let b: Vec<String> = self.b.iter().map(|b| b.pretty()).collect();
        let v = PrettyFraction::new(self.value * -1.0).pretty();
        header.push(v);
        let mut table = vec![header];
        for i in 0..a.len() {
            let mut row = a[i].clone();
            row.push(b[i].clone());
            table.push(row);
        }
        table
    }
    pub fn pretty_string(&self) -> String {
        let table = self.pretty_table();
        let mut string = String::new();
        for row in table {
            for cell in row {
                string.push_str(&format!("{: >5} ", cell));
            }
            string.push('\n');
        }
        string
    }
}

pub trait Tableauable {
    fn get_b(&self) -> Vec<f64>;
    fn get_c(&self) -> Vec<f64>;
    fn get_a(&self) -> Vec<Vec<f64>>;
    fn get_variables(&self) -> Vec<String>;
    fn get_objective_offset(&self) -> f64;
}

#[derive(Debug)]
pub enum CanonicalTransformError {
    Raw(String),
    InvalidBasis(String),
    Infesible(String),
    SimplexError(String),
}

impl fmt::Display for CanonicalTransformError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let s = match self {
            Self::Raw(s) => s.clone(),
            Self::InvalidBasis(s) => format!("Invalid Basis: {}", s),
            Self::Infesible(s) => format!("Infesible: {}", s),
            Self::SimplexError(s) => format!("Simplex Error: {}", s),
        };
        f.write_str(&s)
    }
}

pub trait IntoCanonicalTableau {
    fn into_canonical(&self) -> Result<Tableau, CanonicalTransformError>;
}

pub fn divide_matrix_row_by(matrix: &mut Vec<Vec<f64>>, row: usize, value: f64) {
    for i in 0..matrix[row].len() {
        matrix[row][i] /= value;
    }
}
