
use crate::simplex::Tableauable;




struct EqualityConstraint<T>{
    coefficients: Vec<T>,
    rhs: T,
}

pub struct StandardLinearProblem{
    objective: Vec<f64>,
    constraints: Vec<EqualityConstraint<f64>>,
}

impl StandardLinearProblem{
    fn new(objective: Vec<f64>, constraints: Vec<EqualityConstraint<f64>>) -> StandardLinearProblem{
        StandardLinearProblem{objective, constraints}
    }
}



impl Tableauable for StandardLinearProblem {
    fn get_b(&self) -> Vec<f64>{
        self.constraints.iter().map(|c| c.rhs).collect()
    }
    fn get_c(&self) -> Vec<f64>{
        self.objective.clone()
    }
    fn get_a(&self) -> Vec<Vec<f64>>{
        self.constraints.iter().map(|c| c.coefficients.clone()).collect()
    }
}



