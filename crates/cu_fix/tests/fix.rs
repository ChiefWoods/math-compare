use core::hint::black_box;

use cu_fix::fix::{aliases::si::Micro, typenum::N6};
use svm_unit_test::svm_test;

const MAX_BPS: i64 = 10_000;

fn amount() -> Micro<i64> {
    Micro::new(1_234_567_000_000)
}

fn scalar(value: i64) -> Micro<i64> {
    Micro::new(value * 1_000_000)
}

fn bps(bps: i64) -> Micro<i64> {
    Micro::new(bps * 1_000_000 / MAX_BPS)
}

fn sqrt_newton(n: Micro<i64>, iterations: usize) -> Micro<i64> {
    let mut guess = (n + scalar(1)) / 2;

    for _ in 0..iterations {
        let quotient: Micro<i64> = (n / guess).convert::<N6>();
        guess = ((guess + quotient).convert::<N6>()) / 2;
    }

    guess
}

fn exp_taylor(x: Micro<i64>, terms: usize) -> Micro<i64> {
    let mut sum = scalar(1);
    let mut term = scalar(1);

    let mut n = 1;
    while n <= terms {
        term = (term * x).convert::<N6>() / n as i64;
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

    let out: Micro<i64> = (((a + b) - c) * bps(987)).convert::<N6>() / 3;
    black_box(out);
}

#[svm_test]
fn assign_arithmetic() {
    let mut out = black_box(amount());

    out += scalar(321);
    out -= scalar(123);
    out = (out * bps(987)).convert::<N6>();
    out /= 2;

    black_box(out);
}

#[svm_test]
fn basis_points() {
    let principal = black_box(amount());
    let fee: Micro<i64> = (principal * bps(30)).convert::<N6>();
    let rebate: Micro<i64> = (principal * bps(5)).convert::<N6>();
    let max_fee: Micro<i64> = (principal * bps(MAX_BPS)).convert::<N6>();
    let net = principal - fee + rebate;

    black_box((fee, rebate, max_fee, net));
}

#[svm_test]
fn powers_and_sqrt() {
    let x = black_box(Micro::new(1_100_000));
    let x2: Micro<i64> = (x * x).convert::<N6>();
    let x4: Micro<i64> = (x2 * x2).convert::<N6>();
    let root = sqrt_newton(black_box(scalar(144)), black_box(20));

    black_box((x2, x4, root));
}

#[svm_test]
fn exp_approximation() {
    let x = black_box(Micro::new(750_000));
    black_box(exp_taylor(x, 10));
}
