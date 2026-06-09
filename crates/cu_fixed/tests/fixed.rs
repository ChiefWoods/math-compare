use core::hint::black_box;

use cu_fixed::fixed::types::I80F48;
use svm_unit_test::svm_test;

const MAX_BPS: i128 = 10_000;

fn amount() -> I80F48 {
    I80F48::from_num(1_234_567)
}

fn rate() -> I80F48 {
    I80F48::from_num(987) / I80F48::from_num(MAX_BPS)
}

fn bps(bps: i128) -> I80F48 {
    I80F48::from_num(bps) / I80F48::from_num(MAX_BPS)
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
    let a = black_box(amount());
    let b = black_box(I80F48::from_num(42));
    let c = black_box(I80F48::from_num(7));

    let out = ((a + b) - c) * rate() / I80F48::from_num(3);
    black_box(out);
}

#[svm_test]
fn checked_arithmetic() {
    let a = black_box(amount());
    let b = black_box(I80F48::from_num(321));

    let out = a
        .checked_add(b)
        .and_then(|v| v.checked_sub(I80F48::from_num(123)))
        .and_then(|v| v.checked_mul(rate()))
        .and_then(|v| v.checked_div(I80F48::from_num(2)));
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
    let x = black_box(I80F48::from_num(11) / I80F48::from_num(10));
    let x2 = x * x;
    let x4 = x2 * x2;
    let root = black_box(I80F48::from_num(144)).sqrt();

    black_box((x2, x4, root));
}

#[svm_test]
fn exp_approximation() {
    let x = black_box(I80F48::from_num(75) / I80F48::from_num(100));
    black_box(exp_taylor(x, black_box(10)));
}
