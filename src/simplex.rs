use std::ops::Index;

use num_rational::Rational64;
use num_traits::cast::FromPrimitive;
#[derive(Debug, Clone)]
pub struct Tableau {
    c: Vec<f64>,
    a: Vec<Vec<f64>>,
    b: Vec<f64>,
    in_basis: Vec<usize>,
    current_value: f64,
}

#[derive(Debug, Clone)]
pub struct OptimalTableau {
    values: Vec<f64>,
    tableau: Tableau,
}

impl OptimalTableau {
    fn new(values: Vec<f64>, tableau: Tableau) -> OptimalTableau {
        OptimalTableau { values, tableau }
    }

    pub fn get_variables_values(&self) -> &Vec<f64> {
        &self.values
    }
    pub fn get_optimal_value(&self) -> f64 {
        self.tableau.get_current_value()
    }
    fn get_tableau(&self) -> &Tableau {
        &self.tableau
    }
}

#[derive(Debug)]
pub enum SimplexError {
    Unbounded,
    IterationLimitReached,
    Other,
}

impl Tableau {
    pub fn new(c: Vec<f64>, a: Vec<Vec<f64>>, b: Vec<f64>, in_basis: Vec<usize>, current_value: f64) -> Tableau {
        Tableau { c, a, b, in_basis , current_value}
    }
    pub fn solve(&mut self, limit: i64) -> Result<OptimalTableau, SimplexError> {
        let mut iteration = 0;
        while iteration <= limit {
            match self.step() {
                Ok(false) => {
                    iteration += 1;
                }
                Ok(true) => {
                    return Ok(OptimalTableau::new(self.get_variables_values(), self.clone()));
                }
                Err(e) => {
                    return Err(e);
                }
            }
        }
        Err(SimplexError::IterationLimitReached)
    }

    pub fn step(&mut self) -> Result<bool, SimplexError> {
        if self.is_optimal() {
            return Ok(true);
        }
        match self.find_h() {
            None => Err(SimplexError::Unbounded),
            Some(h) => {
                let t = self.find_t(h);
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

    fn is_unbounded(&self, h: usize) -> bool {
        self.a.iter().all(|a| a[h] <= 0.0)
    }

    fn find_h(&self) -> Option<usize> {
        //uses the Bland's rule for anti-cycling
        let min = self
            .c
            .iter()
            .enumerate()
            .filter(|(i, c)| !self.in_basis.contains(i) && **c < 0.0)
            .min_by(|(_, c1), (_, c2)| c1.partial_cmp(c2).unwrap());
        match min {
            Some((i, _)) => Some(i),
            None => None,
        }
    }
    fn find_t(&self, h: usize) -> Option<usize> {
        //use the Bland's rule for anti-cycling
        //gets the index of the row with the minimum ratio
        let mut valid = self
            .a
            .iter()
            .enumerate()
            .filter(|(_, a)| a[h] > 0.0)
            .map(|(i, a)| (i, self.b[i] / a[h]));
        //println!("Valid: {:?}", valid.clone().map(|(i, ratio)| ratio).collect::<Vec<f64>>());
        match valid.next() {
            Some(first) => {
                let mut min = first;
                for (i, ratio) in valid {
                    if ratio == min.1 {
                        if self.in_basis[i] < self.in_basis[min.0] {
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
        println!("Pivot: ({},{})", t, h);
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
    pub fn to_fractional_tableau(&mut self) -> FractionalTableau {
        FractionalTableau::new(self)
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
    in_basis: Vec<usize>,
    value: f64,
}
impl FractionalTableau {
    pub fn new(tableau: &Tableau) -> FractionalTableau {
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
        let a: Vec<Vec<String>> = self.a.iter().map(|a| a.iter().map(|a| a.pretty()).collect()).collect();
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
}
