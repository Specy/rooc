use indexmap::IndexMap;
use rooc::{Assignment, DualValues, LpSolution};

#[test]
fn lp_solution_exposes_optional_named_shadow_prices() {
    let mut duals = IndexMap::new();
    duals.insert("capacity".to_string(), 2.5);
    let solution = LpSolution::new(
        vec![Assignment {
            name: "x".to_string(),
            value: 1.0,
        }],
        1.0,
        IndexMap::new(),
    )
    .with_shadow_prices(duals);

    assert_eq!(solution.shadow_price("capacity"), Some(2.5));
    assert_eq!(solution.shadow_price("missing"), None);

    let serialized = serde_json::to_value(&solution).unwrap();
    assert!(serialized.get("shadow_prices").is_none());
}

#[test]
fn lp_solution_lookup_preserves_assignment_order_and_first_duplicate() {
    let solution = LpSolution::new(
        vec![
            Assignment {
                name: "x".to_string(),
                value: 1.0,
            },
            Assignment {
                name: "y".to_string(),
                value: 2.0,
            },
            Assignment {
                name: "x".to_string(),
                value: 3.0,
            },
        ],
        0.0,
        IndexMap::new(),
    );

    assert_eq!(solution.value_of("x"), Some(1.0));
    assert_eq!(solution.value_of("x"), Some(1.0));
    assert_eq!(solution.value_of("missing"), None);
    assert_eq!(
        solution
            .assignment()
            .iter()
            .map(|item| item.name.as_str())
            .collect::<Vec<_>>(),
        vec!["x", "y", "x"]
    );
}

#[test]
fn lp_solution_serializes_and_restores_its_lookup_map() {
    let solution = LpSolution::new(
        vec![
            Assignment {
                name: "x".to_string(),
                value: 1.0,
            },
            Assignment {
                name: "y".to_string(),
                value: 2.0,
            },
        ],
        0.0,
        IndexMap::new(),
    );

    let serialized = serde_json::to_value(&solution).unwrap();
    let assignment_by_name = serialized
        .get("assignment_by_name")
        .expect("the lookup map should be serialized");
    assert_eq!(assignment_by_name.get("x").unwrap().as_f64(), Some(1.0));
    assert_eq!(assignment_by_name.get("y").unwrap().as_f64(), Some(2.0));

    let deserialized: LpSolution<f64> = serde_json::from_value(serialized).unwrap();
    assert_eq!(deserialized.value_of("y"), Some(2.0));
}
