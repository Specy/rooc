use std::collections::HashMap;

use crate::parser::model_transformer::model::Model;
use crate::parser::model_transformer::transformer_context::DomainVariable;
use crate::transformers::linear_model::LinearModel;

/**TODO
The linearizer module contains the code for attempting to linearize a problem into a linear problem
where the lhs is formed only by addition of variables with a constant multiplier and the rhs is formed by a single constant value.
It achieves this by following the linearization rules, where:
1. Multiplication and division of two variables is not permitted as it is not linear, but linearized if
   the lhs is all divided/multiplied by a variable, and the rhs is a constant.
2. The MIN function can be converted in a linear way by:
     min(x1, x2) + y <= b
      BECOMES:
3. The MAX function can be converted in a linear way by:
     max(x1, x2) <= b
      BECOMES:
4. The ABS function can be converted in a linear way by:
     |x1| + y <= b
       BECOMES:
      x1 + y>= b
      x1 - y>= -b
 */

struct LinearVariable {
    name: String,
    multiplier: f64,
}


enum ContextLocation{
    Constraint,
    Objective
}
pub struct LinearizerContext{
    surplus_count: u32,
    slack_count: u32,
    extra_count: u32,
    domain: HashMap<String, DomainVariable>
}


pub struct Linearizer {

}


pub fn linearize_model(model: Model) -> LinearModel {
    for constraint in model.get_constraints() {
    }
    todo!()
}


