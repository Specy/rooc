use indexmap::IndexMap;
use rooc::model_transformer::{TransformError, TransformerContext};
use rooc::type_checker::type_checker_context::{FunctionContext, TypeCheckerContext};
use rooc::Linearizer;
use rooc::PreExp;
use rooc::RoocFunction;
use rooc::RoocParser;
use rooc::{solve_integer_binary_lp_problem, FunctionContextMap};
use rooc::{Constant, IterableKind, Primitive, PrimitiveKind};

fn main() {
    let source = "
max sum((value, i) in enumerate(doubler(values))) { value * x_i }
s.t.
    sum((weight, i) in enumerate(weights)) { weight * x_i } <= capacity

define
    x_i as Boolean for i in 0..len(weights)";

    let rooc = RoocParser::new(source.to_string());
    let parsed = rooc.parse().unwrap();
    let constants = vec![
        Constant::from_primitive(
            "weights",
            IterableKind::Integers(vec![10, 60, 30, 40, 30, 20, 20, 2]).into_primitive(),
        ),
        Constant::from_primitive(
            "values",
            IterableKind::Integers(vec![1, 10, 15, 40, 60, 90, 100, 15]).into_primitive(),
        ),
        Constant::from_primitive("capacity", Primitive::Integer(102)),
    ];
    let mut fns: FunctionContextMap = IndexMap::new();
    fns.insert("doubler".to_string(), Box::new(Doubler {}));

    let model = parsed.transform(constants, &fns).unwrap();
    let linear = Linearizer::linearize(model).unwrap();
    let solution = solve_integer_binary_lp_problem(&linear).unwrap();
    println!("{}", solution)
}

#[derive(Debug)]
struct Doubler {}

impl RoocFunction for Doubler {
    fn call(
        &self,
        args: &[PreExp],
        context: &TransformerContext,
        fn_context: &FunctionContext,
    ) -> Result<Primitive, TransformError> {
        let arr = args.first().unwrap().as_iterator(context, fn_context)?;
        match arr {
            IterableKind::Integers(i) => {
                let doubled = i.iter().map(|v| v * 2).collect();
                Ok(Primitive::Iterable(IterableKind::Integers(doubled)))
            }
            _ => unreachable!(),
        }
    }

    fn type_signature(
        &self,
        _args: &[PreExp],
        _context: &TypeCheckerContext,
        _fn_context: &FunctionContext,
    ) -> Vec<(String, PrimitiveKind)> {
        vec![(
            "of_array".to_string(),
            PrimitiveKind::Iterable(Box::new(PrimitiveKind::Integer)),
        )]
    }

    fn return_type(
        &self,
        _args: &[PreExp],
        _context: &TypeCheckerContext,
        _fn_context: &FunctionContext,
    ) -> PrimitiveKind {
        PrimitiveKind::Iterable(Box::new(PrimitiveKind::Integer))
    }

    fn function_name(&self) -> String {
        "doubler".to_string()
    }
}
