use core::hint::black_box;

use cu_hylo_fix::fixtures::{
    CHECKED_ADDEND_MICRO, CHECKED_DIVISOR_MICRO, CHECKED_SUBTRAHEND_MICRO, DELTA_MINUS_MICRO,
    DELTA_PLUS_MICRO, DIVISOR_MICRO, EXP_INPUT_MICRO, MAX_BPS, MICRO_SCALE, POW_BASE_MICRO,
    PRINCIPAL_MICRO, RATE_BPS, SQRT_INPUT_MICRO,
};
use cu_hylo_fix::hylo_fix::{
    aliases::si::Micro,
    num_traits::{CheckedAdd, CheckedSub},
    typenum::N6,
    CheckedMulFix,
};
use svm_unit_test::svm_test;

fn principal() -> Micro<i128> {
    Micro::new(PRINCIPAL_MICRO as i128)
}

fn bps(bps: i128) -> Micro<i128> {
    Micro::new(bps * MICRO_SCALE as i128 / MAX_BPS as i128)
}

fn checked_mul_at_n6(a: Micro<i128>, b: Micro<i128>) -> Option<Micro<i128>> {
    a.checked_mul(&b).map(|product| product.convert::<N6>())
}

fn sqrt_newton(n: Micro<i128>, iterations: usize) -> Micro<i128> {
    let mut guess = (n + Micro::new(MICRO_SCALE as i128)) / 2;

    for _ in 0..iterations {
        let quotient: Micro<i128> = (n / guess).convert::<N6>();
        guess = ((guess + quotient).convert::<N6>()) / 2;
    }

    guess
}

fn exp_taylor(x: Micro<i128>, terms: usize) -> Micro<i128> {
    let mut sum = Micro::new(MICRO_SCALE as i128);
    let mut term = Micro::new(MICRO_SCALE as i128);

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
    let a = black_box(principal());
    let b = black_box(Micro::new(DELTA_PLUS_MICRO as i128));
    let c = black_box(Micro::new(DELTA_MINUS_MICRO as i128));

    let out = ((((a + b) - c) * bps(RATE_BPS as i128)).convert::<N6>()
        / Micro::new(DIVISOR_MICRO as i128))
        .convert::<N6>();
    black_box(out);
}

#[svm_test]
fn checked_arithmetic() {
    let a = black_box(principal());
    let b = black_box(Micro::new(CHECKED_ADDEND_MICRO as i128));

    let out = a
        .checked_add(&b)
        .and_then(|v| v.checked_sub(&Micro::new(CHECKED_SUBTRAHEND_MICRO as i128)))
        .and_then(|v| checked_mul_at_n6(v, bps(RATE_BPS as i128)))
        .map(|v| (v / Micro::new(CHECKED_DIVISOR_MICRO as i128)).convert::<N6>());
    black_box(out);
}

#[svm_test]
fn basis_points() {
    let principal = black_box(principal());
    let fee: Micro<i128> = (principal * bps(30)).convert::<N6>();
    let rebate: Micro<i128> = (principal * bps(5)).convert::<N6>();
    let max_fee: Micro<i128> = (principal * bps(MAX_BPS as i128)).convert::<N6>();
    let net = principal - fee + rebate;

    black_box((fee, rebate, max_fee, net));
}

#[svm_test]
fn powers_and_sqrt() {
    let x = black_box(Micro::new(POW_BASE_MICRO as i128));
    let x2: Micro<i128> = (x * x).convert::<N6>();
    let x4: Micro<i128> = (x2 * x2).convert::<N6>();
    let root = sqrt_newton(black_box(Micro::new(SQRT_INPUT_MICRO as i128)), black_box(20));

    black_box((x2, x4, root));
}

#[svm_test]
fn exp_approximation() {
    let x = black_box(Micro::new(EXP_INPUT_MICRO as i128));
    black_box(exp_taylor(x, black_box(10)));
}
