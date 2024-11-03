use crate::solvers::simplex::Tableau;
use num_rational::Rational64;
use num_traits::cast::FromPrimitive;

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
            c: tableau
                .c_vec()
                .iter()
                .map(|&c| PrettyFraction::new(c))
                .collect(),
            a: tableau
                .a_matrix()
                .iter()
                .map(|a| a.iter().map(|&a| PrettyFraction::new(a)).collect())
                .collect(),
            b: tableau
                .b_vec()
                .iter()
                .map(|&b| PrettyFraction::new(b))
                .collect(),
            in_basis: tableau.in_basis().clone(),
            value: tableau.current_value(),
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
