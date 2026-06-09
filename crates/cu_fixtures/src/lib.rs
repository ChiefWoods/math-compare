#![no_std]

//! Semantic test fixtures shared across math-compare library crates.
//!
//! All values use 6 decimal places. Each crate encodes `*_MICRO` constants
//! into its native fixed-point type.

pub const MICRO_SCALE: u128 = 1_000_000;
pub const MAX_BPS: u128 = 10_000;
pub const RATE_BPS: u128 = 987;

/// 1_234_567.890123
pub const PRINCIPAL_MICRO: u128 = 1_234_567_890_123;
/// 42.375000
pub const DELTA_PLUS_MICRO: u128 = 42_375_000;
/// 7.125000
pub const DELTA_MINUS_MICRO: u128 = 7_125_000;
/// 3.250000
pub const DIVISOR_MICRO: u128 = 3_250_000;
/// 321.375000
pub const CHECKED_ADDEND_MICRO: u128 = 321_375_000;
/// 123.125000
pub const CHECKED_SUBTRAHEND_MICRO: u128 = 123_125_000;
/// 2.500000
pub const CHECKED_DIVISOR_MICRO: u128 = 2_500_000;
/// 1.100000
pub const POW_BASE_MICRO: u128 = 1_100_000;
/// 0.750000
pub const EXP_INPUT_MICRO: u128 = 750_000;
/// 123.456000
pub const SQRT_INPUT_MICRO: u128 = 123_456_000;
