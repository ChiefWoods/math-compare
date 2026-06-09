use core::hint::black_box;

use cu_rust_decimal::rust_decimal::{Decimal, prelude::MathematicalOps};
use svm_unit_test::svm_test;

const MAX_BPS: i64 = 10_000;

fn dec(value: i64, scale: u32) -> Decimal {
    Decimal::new(value, scale)
}

fn amount() -> Decimal {
    Decimal::new(1_234_567, 0)
}

fn bps(bps: i64) -> Decimal {
    Decimal::new(bps, 0) / Decimal::new(MAX_BPS, 0)
}

#[svm_test]
fn add_sub_mul_div() {
    let a = black_box(amount());
    let b = black_box(dec(42, 0));
    let c = black_box(dec(7, 0));

    let out = ((a + b) - c) * bps(987) / dec(3, 0);
    black_box(out);
}

#[svm_test]
fn checked_arithmetic() {
    let a = black_box(amount());
    let b = black_box(dec(321, 0));

    let out = a
        .checked_add(b)
        .and_then(|v| v.checked_sub(dec(123, 0)))
        .and_then(|v| v.checked_mul(bps(987)))
        .and_then(|v| v.checked_div(dec(2, 0)));
    black_box(out);
}

#[svm_test]
fn basis_points() {
    let principal = black_box(amount());
    let fee = principal * bps(30);
    let rebate = principal * bps(5);
    let max_fee = principal * bps(MAX_BPS);
    let net = principal - fee + rebate;

    black_box((fee, rebate, max_fee, net));
}

#[svm_test]
fn powers_and_sqrt() {
    let x = black_box(dec(11, 1));
    let x2 = x.powu(2);
    let x4 = x.powu(4);
    let root = dec(144, 0).sqrt();

    black_box((x2, x4, root));
}

#[svm_test]
fn exp() {
    let x = black_box(dec(75, 2));
    black_box(x.exp());
}
