use core::hint::black_box;

use cu_bigdecimal::{
    bigdecimal::{BigDecimal, num_bigint::BigInt},
    fixtures::{
        DELTA_MINUS_MICRO, DELTA_PLUS_MICRO, DIVISOR_MICRO, MAX_BPS, POW_BASE_MICRO,
        PRINCIPAL_MICRO, RATE_BPS,
    },
};
use svm_unit_test::svm_test;

const MICRO_DECIMAL_SCALE: i64 = 6;

fn from_micro_units(units: u128) -> BigDecimal {
    BigDecimal::new(BigInt::from(units), MICRO_DECIMAL_SCALE)
}

fn principal() -> BigDecimal {
    from_micro_units(PRINCIPAL_MICRO)
}

fn bps(bps: u128) -> BigDecimal {
    BigDecimal::from(bps) / BigDecimal::from(MAX_BPS)
}

#[svm_test]
fn add_sub_mul_div() {
    let a = black_box(principal());
    let b = black_box(from_micro_units(DELTA_PLUS_MICRO));
    let c = black_box(from_micro_units(DELTA_MINUS_MICRO));

    let out = ((a + b) - c) * bps(RATE_BPS) / from_micro_units(DIVISOR_MICRO);
    black_box(out);
}

#[svm_test]
fn basis_points() {
    let principal = black_box(principal());
    let fee = principal.clone() * bps(30);
    let rebate = principal.clone() * bps(5);
    let max_fee = principal.clone() * bps(MAX_BPS);
    let net = principal - fee.clone() + rebate.clone();

    black_box((fee, rebate, max_fee, net));
}

#[svm_test]
fn powers() {
    let x = black_box(from_micro_units(POW_BASE_MICRO));
    let x2 = x.clone() * x;
    let x4 = x2.clone() * x2.clone();

    black_box((x2, x4));
}
