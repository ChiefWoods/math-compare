// `PreciseNumber` only exposes checked arithmetic (`checked_add`, `checked_mul`, etc.).
// Unlike the other library crates, non-`checked_arithmetic` tests here cannot use
// unchecked operators and will call `checked_*` (typically via `.unwrap()`).

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
    let root = black_box(number(144)).sqrt().unwrap();

    black_box((x2, x4, root));
}

#[svm_test]
fn exp_approximation() {
    let x = black_box(number(75).checked_div(&number(100)).unwrap());
    black_box(exp_taylor(&x, black_box(10)));
}
