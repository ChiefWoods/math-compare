use core::hint::black_box;

use cu_rust_decimal::fixtures::{
    CHECKED_ADDEND_MICRO, CHECKED_DIVISOR_MICRO, CHECKED_SUBTRAHEND_MICRO, DELTA_MINUS_MICRO,
    DELTA_PLUS_MICRO, DIVISOR_MICRO, EXP_INPUT_MICRO, MAX_BPS, POW_BASE_MICRO, PRINCIPAL_MICRO,
    RATE_BPS, SQRT_INPUT_MICRO,
};
use cu_rust_decimal::rust_decimal::{Decimal, prelude::MathematicalOps};
use svm_unit_test::svm_test;

const MICRO_DECIMAL_SCALE: u32 = 6;

fn from_micro_units(units: i64) -> Decimal {
    Decimal::new(units, MICRO_DECIMAL_SCALE)
}

fn principal() -> Decimal {
    from_micro_units(PRINCIPAL_MICRO as i64)
}

fn bps(bps: i64) -> Decimal {
    Decimal::new(bps, 0) / Decimal::new(MAX_BPS as i64, 0)
}

#[svm_test]
fn add_sub_mul_div() {
    let a = black_box(principal());
    let b = black_box(from_micro_units(DELTA_PLUS_MICRO as i64));
    let c = black_box(from_micro_units(DELTA_MINUS_MICRO as i64));

    let out = ((a + b) - c) * bps(RATE_BPS as i64) / from_micro_units(DIVISOR_MICRO as i64);
    black_box(out);
}

#[svm_test]
fn checked_arithmetic() {
    let a = black_box(principal());
    let b = black_box(from_micro_units(CHECKED_ADDEND_MICRO as i64));

    let out = a
        .checked_add(b)
        .and_then(|v| v.checked_sub(from_micro_units(CHECKED_SUBTRAHEND_MICRO as i64)))
        .and_then(|v| v.checked_mul(bps(RATE_BPS as i64)))
        .and_then(|v| v.checked_div(from_micro_units(CHECKED_DIVISOR_MICRO as i64)));
    black_box(out);
}

#[svm_test]
fn basis_points() {
    let principal = black_box(principal());
    let fee = principal * bps(30);
    let rebate = principal * bps(5);
    let max_fee = principal * bps(MAX_BPS as i64);
    let net = principal - fee + rebate;

    black_box((fee, rebate, max_fee, net));
}

#[svm_test]
fn powers_and_sqrt() {
    let x = black_box(from_micro_units(POW_BASE_MICRO as i64));
    let x2 = x.powu(2);
    let x4 = x.powu(4);
    let root = black_box(from_micro_units(SQRT_INPUT_MICRO as i64)).sqrt();

    black_box((x2, x4, root));
}

#[svm_test]
fn exp() {
    let x = black_box(from_micro_units(EXP_INPUT_MICRO as i64));
    black_box(x.exp());
}
