set positional-arguments

libraries := "fixed rust-decimal fix spl-math"

default:
    just --list

test:
    cargo test --workspace -- --nocapture

check:
    cargo check --workspace

clippy:
    cargo clippy --workspace --all-targets -- -D warnings

fmt:
    cargo fmt --all

fixed:
    @cargo run --quiet --bin compare-cu -- --raw fixed

rust-decimal:
    @cargo run --quiet --bin compare-cu -- --raw rust-decimal

fix:
    @cargo run --quiet --bin compare-cu -- --raw fix

spl-math:
    @cargo run --quiet --bin compare-cu -- --raw spl-math

compare *args:
    cargo run --quiet --bin compare-cu -- {{args}}

compare-all:
    cargo run --quiet --bin compare-cu -- {{libraries}}

compare-save *libs:
    cargo run --quiet --bin compare-cu -- {{libs}} --save outputs/comparison-{{ replace(libs, " ", "-") }}.csv

compare-save-all:
    cargo run --quiet --bin compare-cu -- {{libraries}} --save outputs/comparison-all.csv
