use core::hint::black_box;

use cu_hylo_fix::hylo_fix::{
    aliases::si::Micro,
    num_traits::{CheckedAdd, CheckedSub},
    typenum::N6,
    CheckedMulFix,
};
use svm_unit_test::svm_test;

const MAX_BPS: i128 = 10_000;

fn amount() -> Micro<i128> {
    Micro::new(1_234_567_000_000)
}

fn scalar(value: i128) -> Micro<i128> {
    Micro::new(value * 1_000_000)
}

fn bps(bps: i128) -> Micro<i128> {
    Micro::new(bps * 1_000_000 / MAX_BPS)
}

fn checked_mul_at_n6(a: Micro<i128>, b: Micro<i128>) -> Option<Micro<i128>> {
    a.checked_mul(&b).map(|product| product.convert::<N6>())
}

fn sqrt_newton(n: Micro<i128>, iterations: usize) -> Micro<i128> {
    let mut guess = (n + scalar(1)) / 2;

    for _ in 0..iterations {
        let quotient: Micro<i128> = (n / guess).convert::<N6>();
        guess = ((guess + quotient).convert::<N6>()) / 2;
    }

    guess
}

fn exp_taylor(x: Micro<i128>, terms: usize) -> Micro<i128> {
    let mut sum = scalar(1);
    let mut term = scalar(1);

    let mut n = 1;
    while n <= terms {
        term = (term * x).convert::<N6>() / n as i128;
        sum += term;
        n += 1;
    }

    sum
}

#[svm_test]
fn add_sub_mul_div() {
    let a = black_box(amount());
    let b = black_box(scalar(42));
    let c = black_box(scalar(7));

    let out: Micro<i128> = (((a + b) - c) * bps(987)).convert::<N6>() / 3;
    black_box(out);
}

#[svm_test]
fn checked_arithmetic() {
    let a = black_box(amount());
    let b = black_box(scalar(321));

    let out = a
        .checked_add(&b)
        .and_then(|v| v.checked_sub(&scalar(123)))
        .and_then(|v| checked_mul_at_n6(v, bps(987)))
        .and_then(|v| v.bits.checked_div(2).map(Micro::new));
    black_box(out);
}

#[svm_test]
fn basis_points() {
    let principal = black_box(amount());
    let fee: Micro<i128> = (principal * bps(30)).convert::<N6>();
    let rebate: Micro<i128> = (principal * bps(5)).convert::<N6>();
    let max_fee: Micro<i128> = (principal * bps(MAX_BPS)).convert::<N6>();
    let net = principal - fee + rebate;

    black_box((fee, rebate, max_fee, net));
}

#[svm_test]
fn powers_and_sqrt() {
    let x = black_box(Micro::new(1_100_000));
    let x2: Micro<i128> = (x * x).convert::<N6>();
    let x4: Micro<i128> = (x2 * x2).convert::<N6>();
    let root = sqrt_newton(black_box(scalar(144)), black_box(20));

    black_box((x2, x4, root));
}

#[svm_test]
fn exp_approximation() {
    let x = black_box(Micro::new(750_000));
    black_box(exp_taylor(x, black_box(10)));
}
