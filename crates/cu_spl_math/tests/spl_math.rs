use core::hint::black_box;

use cu_spl_math::spl_math::precise_number::PreciseNumber;
use svm_unit_test::svm_test;

const MAX_BPS: u128 = 10_000;

fn number(value: u128) -> PreciseNumber {
    PreciseNumber::new(value).unwrap()
}

fn bps(bps: u128) -> PreciseNumber {
    number(bps).checked_div(&number(MAX_BPS)).unwrap()
}

#[svm_test]
fn add_sub_mul_div() {
    let a = black_box(number(1_234_567));
    let b = black_box(number(42));
    let c = black_box(number(7));
    let rate = bps(987);

    let out = a
        .checked_add(&b)
        .unwrap()
        .checked_sub(&c)
        .unwrap()
        .checked_mul(&rate)
        .unwrap()
        .checked_div(&number(3))
        .unwrap();
    black_box(out);
}

#[svm_test]
fn checked_arithmetic() {
    let a = black_box(number(1_234_567));
    let b = black_box(number(321));

    let out = a
        .checked_add(&b)
        .and_then(|v| v.checked_sub(&number(123)))
        .and_then(|v| v.checked_mul(&bps(987)))
        .and_then(|v| v.checked_div(&number(2)));
    black_box(out);
}

#[svm_test]
fn basis_points() {
    let principal = black_box(number(1_234_567));
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
    let x = black_box(number(11).checked_div(&number(10)).unwrap());
    let x2 = x.checked_pow(2).unwrap();
    let x4 = x.checked_pow(4).unwrap();
    let root = number(144).sqrt().unwrap();

    black_box((x2, x4, root));
}

#[svm_test]
fn pow_series() {
    let base = black_box(number(10075).checked_div(&number(10000)).unwrap());
    let compounded = base.checked_pow(10).unwrap();

    black_box(compounded);
}
