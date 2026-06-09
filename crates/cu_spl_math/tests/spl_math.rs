// `PreciseNumber` only exposes checked arithmetic (`checked_add`, `checked_mul`, etc.).
// Unlike the other library crates, non-`checked_arithmetic` tests here cannot use
// unchecked operators and will call `checked_*` (typically via `.unwrap()`).

use core::hint::black_box;

use cu_spl_math::fixtures::{
    CHECKED_ADDEND_MICRO, CHECKED_DIVISOR_MICRO, CHECKED_SUBTRAHEND_MICRO, DELTA_MINUS_MICRO,
    DELTA_PLUS_MICRO, DIVISOR_MICRO, EXP_INPUT_MICRO, MAX_BPS, MICRO_SCALE, POW_BASE_MICRO,
    PRINCIPAL_MICRO, RATE_BPS, SQRT_INPUT_MICRO,
};
use cu_spl_math::spl_math::precise_number::PreciseNumber;
use svm_unit_test::svm_test;

fn number(value: u128) -> PreciseNumber {
    PreciseNumber::new(value).unwrap()
}

fn from_micro_units(units: u128) -> PreciseNumber {
    number(units).checked_div(&number(MICRO_SCALE)).unwrap()
}

fn principal() -> PreciseNumber {
    from_micro_units(PRINCIPAL_MICRO)
}

fn bps(bps: u128) -> PreciseNumber {
    number(bps).checked_div(&number(MAX_BPS)).unwrap()
}

fn exp_taylor(x: &PreciseNumber, terms: u128) -> PreciseNumber {
    let mut sum = number(1);
    let mut term = number(1);

    for n in 1..=terms {
        term = term.checked_mul(x).unwrap();
        term = term.checked_div(&number(n)).unwrap();
        sum = sum.checked_add(&term).unwrap();
    }

    sum
}

#[svm_test]
fn add_sub_mul_div() {
    let a = black_box(principal());
    let b = black_box(from_micro_units(DELTA_PLUS_MICRO));
    let c = black_box(from_micro_units(DELTA_MINUS_MICRO));
    let rate = bps(RATE_BPS);

    let out = a
        .checked_add(&b)
        .unwrap()
        .checked_sub(&c)
        .unwrap()
        .checked_mul(&rate)
        .unwrap()
        .checked_div(&from_micro_units(DIVISOR_MICRO))
        .unwrap();
    black_box(out);
}

#[svm_test]
fn checked_arithmetic() {
    let a = black_box(principal());
    let b = black_box(from_micro_units(CHECKED_ADDEND_MICRO));

    let out = a
        .checked_add(&b)
        .and_then(|v| v.checked_sub(&from_micro_units(CHECKED_SUBTRAHEND_MICRO)))
        .and_then(|v| v.checked_mul(&bps(RATE_BPS)))
        .and_then(|v| v.checked_div(&from_micro_units(CHECKED_DIVISOR_MICRO)));
    black_box(out);
}

#[svm_test]
fn basis_points() {
    let principal = black_box(principal());
    let fee = principal.checked_mul(&bps(30)).unwrap();
    let rebate = principal.checked_mul(&bps(5)).unwrap();
    let max_fee = principal.checked_mul(&bps(MAX_BPS)).unwrap();
    let net = principal
        .checked_sub(&fee)
        .unwrap()
        .checked_add(&rebate)
        .unwrap();

    black_box((fee, rebate, max_fee, net));
}

#[svm_test]
fn powers_and_sqrt() {
    let x = black_box(from_micro_units(POW_BASE_MICRO));
    let x2 = x.checked_pow(2).unwrap();
    let x4 = x.checked_pow(4).unwrap();
    let root = black_box(from_micro_units(SQRT_INPUT_MICRO))
        .sqrt()
        .unwrap();

    black_box((x2, x4, root));
}

#[svm_test]
fn exp_approximation() {
    let x = black_box(from_micro_units(EXP_INPUT_MICRO));
    black_box(exp_taylor(&x, black_box(10)));
}
