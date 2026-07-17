pub(crate) fn float_eq_precision(a: f64, b: f64, precision: u8) -> bool {
    let diff = (a - b).abs();
    diff < 10_f64.powi(-(precision as i32))
}

pub(crate) fn float_ne_precision(a: f64, b: f64, precision: u8) -> bool {
    !float_eq_precision(a, b, precision)
}

// The ordering predicates are made consistent with `float_eq_precision`: two values
// that are equal within tolerance are treated as equal by all six predicates, so the
// relation stays a valid (partial) order (`eq => le && ge`, `lt => !eq`, etc.). This
// prevents e.g. the simplex from seeing a reduced cost as simultaneously "== 0" and
// "< 0", which previously caused spurious unbounded/infeasible outcomes.
pub(crate) fn float_lt_precision(a: f64, b: f64, precision: u8) -> bool {
    a < b && !float_eq_precision(a, b, precision)
}
pub(crate) fn float_gt_precision(a: f64, b: f64, precision: u8) -> bool {
    a > b && !float_eq_precision(a, b, precision)
}

pub(crate) fn float_le_precision(a: f64, b: f64, precision: u8) -> bool {
    a < b || float_eq_precision(a, b, precision)
}

pub(crate) fn float_ge_precision(a: f64, b: f64, precision: u8) -> bool {
    a > b || float_eq_precision(a, b, precision)
}

const NEAR_ZERO_PRECISION: u8 = 5;

/// Checks if two numbers are the same within 5 decimal digits
pub fn float_eq(a: f64, b: f64) -> bool {
    float_eq_precision(a, b, NEAR_ZERO_PRECISION)
}

/// Checks if two numbers are different within 5 decimal digits
pub fn float_ne(a: f64, b: f64) -> bool {
    float_ne_precision(a, b, NEAR_ZERO_PRECISION)
}

pub(crate) fn float_lt(a: f64, b: f64) -> bool {
    float_lt_precision(a, b, NEAR_ZERO_PRECISION)
}

pub(crate) fn float_gt(a: f64, b: f64) -> bool {
    float_gt_precision(a, b, NEAR_ZERO_PRECISION)
}
pub(crate) fn float_le(a: f64, b: f64) -> bool {
    float_le_precision(a, b, NEAR_ZERO_PRECISION)
}
pub(crate) fn float_ge(a: f64, b: f64) -> bool {
    float_ge_precision(a, b, NEAR_ZERO_PRECISION)
}
