pub fn float_eq_precision(a: f64, b: f64, precision: u8) -> bool {
    let diff = (a - b).abs();
    diff < 10_f64.powi(-(precision as i32))
}

pub fn float_ne_precision(a: f64, b: f64, precision: u8) -> bool {
    !float_eq_precision(a, b, precision)
}

pub fn float_lt_precision(a: f64, b: f64, precision: u8) -> bool {
    let diff = (a - b).abs();
    a < b //|| diff < 10_f64.powi(-(precision as i32))
}
pub fn float_gt_precision(a: f64, b: f64, precision: u8) -> bool {
    let diff = (a - b).abs();
    a > b //|| diff < 10_f64.powi(-(precision as i32))
}

pub fn float_le_precision(a: f64, b: f64, precision: u8) -> bool {
    return a <= b;
    float_lt_precision(a, b, precision) || float_eq_precision(a, b, precision)
}

pub fn float_ge_precision(a: f64, b: f64, precision: u8) -> bool {
    return a >= b;
    float_gt_precision(a, b, precision) || float_eq_precision(a, b, precision)
}

const NEAR_ZERO_PRECISION: u8 = 5;

pub fn float_eq(a: f64, b: f64) -> bool {
    float_eq_precision(a, b, NEAR_ZERO_PRECISION)
}

pub fn float_ne(a: f64, b: f64) -> bool {
    float_ne_precision(a, b, NEAR_ZERO_PRECISION)
}
pub fn float_lt(a: f64, b: f64) -> bool {
    float_lt_precision(a, b, NEAR_ZERO_PRECISION)
}
pub fn float_gt(a: f64, b: f64) -> bool {
    float_gt_precision(a, b, NEAR_ZERO_PRECISION)
}
pub fn float_le(a: f64, b: f64) -> bool {
    float_le_precision(a, b, NEAR_ZERO_PRECISION)
}
pub fn float_ge(a: f64, b: f64) -> bool {
    float_ge_precision(a, b, NEAR_ZERO_PRECISION)
}
