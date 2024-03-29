use crate::solvers::simplex::{CanonicalTransformError, divide_matrix_row_by, IntoCanonicalTableau, Tableau, Tableauable};
use crate::transformers::linear_model::LinearModel;
use crate::transformers::standardizer::to_standard_form;

pub struct EqualityConstraint {
    coefficients: Vec<f64>,
    rhs: f64,
}

struct IndependentVariable {
    row: usize,
    column: usize,
    value: f64,
}


impl IntoCanonicalTableau for StandardLinearModel {
    fn into_canonical(&self) -> Result<Tableau, CanonicalTransformError> {
        let mut usable_independent_vars: Vec<IndependentVariable> = Vec::new();
        //find independent variables by checking if the column has a single value, and if so, add it to the independent list
        for column in 0..self.variables.len() {
            let mut independent_count = 0;
            let mut independent_row = 0;
            let mut independent_value = 0.0;
            for (row, constraint) in self.constraints.iter().enumerate() {
                let coeff = constraint.get_coefficient(column);
                if coeff != 0.0 {
                    independent_count += 1;
                    independent_row = row;
                    independent_value = constraint.get_coefficient(column);
                }
            }
            //only positive values are allowed, as the B column must be all positive
            if independent_count == 1 && independent_value > 0.0 {
                usable_independent_vars.push(IndependentVariable {
                    row: independent_row,
                    column,
                    value: independent_value,
                });
            }
        }
        if usable_independent_vars.len() >= self.constraints.len() {
            //can form a canonical tableau
            let mut a = self.get_a();
            let mut b = self.get_b();
            let mut c = self.get_c();
            let mut value = 0.0;
            //normalize the rows of the independent variables
            for independent_variable in usable_independent_vars.iter() {
                divide_matrix_row_by(&mut a, independent_variable.row, independent_variable.value);
                b[independent_variable.row] /= independent_variable.value;
                let amount = c[independent_variable.column];
                for (index, coefficient) in a[independent_variable.row].iter().enumerate() {
                    c[index] -= amount * coefficient;
                }
                value += amount * b[independent_variable.row];
            }

            let mut basis = usable_independent_vars
                .iter()
                .map(|i| i.column)
                .collect::<Vec<_>>();
            //we only need as many basis variables as there are constraints
            basis.resize(self.constraints.len(), 0);
            Ok(Tableau::new(
                self.get_c(),
                a,
                b,
                basis,
                value,
                self.get_objective_offset(),
                self.get_variables(),
            ))
        } else {
            //use the 2 phase method to find a canonical tableau by adding artificial variables to the constraints and solvign the tableau
            let mut a = self.get_a();
            //TODO can i simplify this by only adding necessary artificial variables? reusing the independent variables?
            let number_of_artificial_variables = self.constraints.len();
            let number_of_variables = self.variables.len();
            let mut variables = self.get_variables();
            let mut c = vec![0.0; number_of_variables + number_of_artificial_variables];
            let mut basis = vec![0; number_of_artificial_variables];
            for i in 0..number_of_artificial_variables {
                c[number_of_variables + i] = 1.0;
                basis[i] = number_of_variables + i;
            }
            let b = self.get_b();

            let mut value = 0.0;
            //add the variables to the matrix and turn the objective function into
            //canonical form by subtracting all rows from the objective function
            for (i, constraint) in a.iter_mut().enumerate() {
                constraint.resize(number_of_variables + number_of_artificial_variables, 0.0);
                constraint[i + number_of_variables] = 1.0;
                variables.push(format!("_a{}", i));
                for (j, coefficient) in constraint.iter().enumerate() {
                    c[j] -= coefficient;
                }
                value -= b[i];
            }

            let mut tableau = Tableau::new(
                c,
                a,
                b,
                basis,
                value,
                self.get_objective_offset(),
                variables,
            );
            let artificial_variables = (number_of_variables
                ..number_of_variables + number_of_artificial_variables)
                .collect::<Vec<_>>();
            match tableau.solve_avoiding(10000, &artificial_variables) {
                Ok(optimal_tableau) => {
                    let tableau = optimal_tableau.get_tableau();
                    if tableau.get_current_value() != 0.0 {
                        return Err(CanonicalTransformError::Infesible(
                            "Initial problem is infesible".to_string(),
                        ));
                    }
                    let new_basis = tableau.get_in_basis().clone();
                    //check that the new basis is valid
                    match new_basis.iter().all(|&i| i < number_of_variables) {
                        true => {
                            //restore the original objective function
                            let mut new_a = tableau.get_a().clone();
                            //remove the artificial variables from the tableau
                            for row in 0..new_a.len() {
                                new_a[row].resize(number_of_variables, 0.0);
                            }
                            let mut value = 0.0;
                            let mut new_c = self.get_c();
                            let new_b = tableau.get_b().clone();
                            //put in the original objective function in canonical form
                            for (row_index, variable_index) in new_basis.iter().enumerate() {
                                //values in base need to be 0, we know that the coefficient in basis is 0 or 1 so we can
                                //simply multiply by the coefficient of the row
                                let coefficient = new_c[*variable_index];
                                for (index, c) in new_c.iter_mut().enumerate() {
                                    *c -= coefficient * new_a[row_index][index];
                                }
                                value -= coefficient * new_b[row_index];
                            }

                            Ok(Tableau::new(
                                new_c,
                                new_a,
                                new_b,
                                new_basis,
                                value,
                                self.get_objective_offset(),
                                self.get_variables(),
                            ))
                        }
                        false => Err(CanonicalTransformError::InvalidBasis(format!(
                            "Invalid basis: {:?}",
                            new_basis
                        ))),
                    }
                }
                Err(e) => Err(CanonicalTransformError::SimplexError(format!(
                    "Error solving initial tableau: {:?}",
                    e
                ))),
            }
        }
    }
}

impl EqualityConstraint {
    pub fn new(coefficients: Vec<f64>, rhs: f64) -> EqualityConstraint {
        match rhs < 0.0 {
            true => EqualityConstraint {
                coefficients: coefficients.iter().map(|c| c * -1.0).collect(),
                rhs: -rhs,
            },
            false => EqualityConstraint { coefficients, rhs },
        }
    }
    pub fn get_coefficients(&self) -> &Vec<f64> {
        &self.coefficients
    }
    pub fn get_coefficient(&self, index: usize) -> f64 {
        self.coefficients[index]
    }
    pub fn get_rhs(&self) -> f64 {
        self.rhs
    }
    pub fn ensure_size(&mut self, size: usize) {
        self.coefficients.resize(size, 0.0);
    }
}

pub struct StandardLinearModel {
    variables: Vec<String>,
    objective_offset: f64,
    objective: Vec<f64>,
    constraints: Vec<EqualityConstraint>,
}

impl StandardLinearModel {
    pub fn new(
        mut objective: Vec<f64>,
        mut constraints: Vec<EqualityConstraint>,
        variables: Vec<String>,
        objective_offset: f64,
    ) -> StandardLinearModel {
        constraints
            .iter_mut()
            .for_each(|c| c.ensure_size(variables.len()));
        objective.resize(variables.len(), 0.0);
        StandardLinearModel {
            objective,
            constraints,
            variables,
            objective_offset,
        }
    }
    pub fn from_linear_problem(linear_problem: LinearModel) -> Result<StandardLinearModel, ()> {
        to_standard_form(&linear_problem)
    }
}

impl Tableauable for StandardLinearModel {
    fn get_b(&self) -> Vec<f64> {
        self.constraints.iter().map(|c| c.rhs).collect()
    }
    fn get_c(&self) -> Vec<f64> {
        self.objective.clone()
    }
    fn get_a(&self) -> Vec<Vec<f64>> {
        self.constraints
            .iter()
            .map(|c| c.coefficients.clone())
            .collect()
    }
    fn get_variables(&self) -> Vec<String> {
        self.variables.clone()
    }
    fn get_objective_offset(&self) -> f64 {
        self.objective_offset
    }
}