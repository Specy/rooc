use crate::simplex::Tableauable;






enum ConstraintType{
    Lower,
    LowerOrEqual,
    Upper,
    UpperOrEqual,
    Equal,
}
enum Equal{

}
struct Constraint<T>{
    constraint_type: ConstraintType,
    coefficients: Vec<T>,
    rhs: T,
}
struct EqualityConstraint<T>{
    coefficients: Vec<T>,
    rhs: T,
}
enum OptimizationType{
    Min,
    Max,
}

/*
    min c^T x
    s.t. 
        Ax = b
         x >= 0
 */



pub struct StandardLinearProblem{
    objective: Vec<f64>,
    constraints: Vec<EqualityConstraint<f64>>,
}
impl StandardLinearProblem{
    fn new(objective: Vec<f64>, constraints: Vec<EqualityConstraint<f64>>) -> StandardLinearProblem{
        StandardLinearProblem{objective, constraints}
    }
}
struct LinearProblem{
    optimization_type: OptimizationType,
    objective: Vec<f64>,
    constraints: Vec<Constraint<f64>>,
}
impl LinearProblem{
    fn new(optimization_type: OptimizationType, objective: Vec<f64>, constraints: Vec<Constraint<f64>>) -> LinearProblem{
        LinearProblem{optimization_type, objective, constraints}
    }
}
struct PL01{
    optimization_type: OptimizationType,
    objective: Vec<i64>,
    constraints: Vec<Constraint<i64>>
}
impl PL01{
    fn new(optimization_type: OptimizationType, objective: Vec<i64>, constraints: Vec<Constraint<i64>>) -> PL01{
        PL01{optimization_type, objective, constraints}
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



