use crate::lexer::{ConstraintType, ObjectiveType, Operation, Token};

#[derive(Debug)]
enum Element {
    Variable(f64, String),
    Number(f64),
}

#[derive(Debug)]
struct Constraint {
    lhs: Vec<Element>,
    constraint_type: ConstraintType,
    rhs: Vec<Element>,
}
impl Constraint {
    pub fn new(
        lhs: Vec<Element>,
        constraint_type: ConstraintType,
        rhs: Vec<Element>,
    ) -> Constraint {
        Constraint {
            lhs,
            constraint_type,
            rhs,
        }
    }
    pub fn new_empty() -> Constraint {
        Constraint {
            lhs: Vec::new(),
            constraint_type: ConstraintType::Equal,
            rhs: Vec::new(),
        }
    }
    pub fn add_lhs_element(&mut self, element: Element) {
        self.lhs.push(element);
    }
    pub fn add_rhs_element(&mut self, element: Element) {
        self.rhs.push(element);
    }
    pub fn set_constraint_type(&mut self, constraint_type: ConstraintType) {
        self.constraint_type = constraint_type;
    }
    pub fn get_lhs(&self) -> &Vec<Element> {
        &self.lhs
    }
    pub fn get_rhs(&self) -> &Vec<Element> {
        &self.rhs
    }
    pub fn get_constraint_type(&self) -> &ConstraintType {
        &self.constraint_type
    }
}

#[derive(Debug)]
struct ObjectiveFunction {
    objective_type: ObjectiveType,
    elements: Vec<Element>,
}
impl ObjectiveFunction {
    pub fn new(objective_type: ObjectiveType, elements: Vec<Element>) -> ObjectiveFunction {
        ObjectiveFunction {
            objective_type,
            elements,
        }
    }
    pub fn new_empty() -> ObjectiveFunction {
        ObjectiveFunction {
            objective_type: ObjectiveType::Max,
            elements: Vec::new(),
        }
    }
    pub fn set_type(&mut self, objective_type: ObjectiveType) {
        self.objective_type = objective_type;
    }
    pub fn add_element(&mut self, element: Element) {
        self.elements.push(element);
    }
    pub fn get_type(&self) -> &ObjectiveType {
        &self.objective_type
    }
    pub fn get_elements(&self) -> &Vec<Element> {
        &self.elements
    }
}

struct LinearProblem {
    objective: ObjectiveFunction,
    constraints: Vec<Constraint>,
}
impl LinearProblem {
    fn new(objective: ObjectiveFunction, constraints: Vec<Constraint>) -> LinearProblem {
        LinearProblem {
            objective,
            constraints,
        }
    }
}
