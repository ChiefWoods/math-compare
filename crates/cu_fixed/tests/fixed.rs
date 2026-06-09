use core::hint::black_box;

use cu_fixed::fixtures::{
    CHECKED_ADDEND_MICRO, CHECKED_DIVISOR_MICRO, CHECKED_SUBTRAHEND_MICRO, DELTA_MINUS_MICRO,
    DELTA_PLUS_MICRO, DIVISOR_MICRO, EXP_INPUT_MICRO, MAX_BPS, MICRO_SCALE, POW_BASE_MICRO,
    PRINCIPAL_MICRO, RATE_BPS, SQRT_INPUT_MICRO,
};
use cu_fixed::fixed::types::I80F48;
use svm_unit_test::svm_test;

fn from_micro_units(units: i128) -> I80F48 {
    I80F48::from_num(units) / I80F48::from_num(MICRO_SCALE as i128)
}

fn principal() -> I80F48 {
    from_micro_units(PRINCIPAL_MICRO as i128)
}

fn bps(bps: i128) -> I80F48 {
    I80F48::from_num(bps) / I80F48::from_num(MAX_BPS as i128)
}

fn exp_taylor(x: I80F48, terms: usize) -> I80F48 {
    let mut sum = I80F48::ONE;
    let mut term = I80F48::ONE;

    let mut n = 1;
    while n <= terms {
        term = term * x / I80F48::from_num(n);
        sum += term;
        n += 1;
    }

    sum
}

#[svm_test]
fn add_sub_mul_div() {
    let a = black_box(principal());
    let b = black_box(from_micro_units(DELTA_PLUS_MICRO as i128));
    let c = black_box(from_micro_units(DELTA_MINUS_MICRO as i128));

    let out = ((a + b) - c) * bps(RATE_BPS as i128) / from_micro_units(DIVISOR_MICRO as i128);
    black_box(out);
}

#[svm_test]
fn checked_arithmetic() {
    let a = black_box(principal());
    let b = black_box(from_micro_units(CHECKED_ADDEND_MICRO as i128));

    let out = a
        .checked_add(b)
        .and_then(|v| v.checked_sub(from_micro_units(CHECKED_SUBTRAHEND_MICRO as i128)))
        .and_then(|v| v.checked_mul(bps(RATE_BPS as i128)))
        .and_then(|v| v.checked_div(from_micro_units(CHECKED_DIVISOR_MICRO as i128)));
    black_box(out);
}

#[svm_test]
fn basis_points() {
    let principal = black_box(principal());
    let fee = principal * bps(30);
    let rebate = principal * bps(5);
    let max_fee = principal * bps(MAX_BPS as i128);
    let net = principal - fee + rebate;

    black_box((fee, rebate, max_fee, net));
}

#[svm_test]
fn powers_and_sqrt() {
    let x = black_box(from_micro_units(POW_BASE_MICRO as i128));
    let x2 = x * x;
    let x4 = x2 * x2;
    let root = black_box(from_micro_units(SQRT_INPUT_MICRO as i128)).sqrt();

    black_box((x2, x4, root));
}

#[svm_test]
fn exp_approximation() {
    let x = black_box(from_micro_units(EXP_INPUT_MICRO as i128));
    black_box(exp_taylor(x, black_box(10)));
}
