set positional-arguments

libraries := "bigdecimal fixed rust-decimal hylo-fix spl-math"

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

bigdecimal:
    @cargo run --quiet --bin compare-cu -- --raw bigdecimal

rust-decimal:
    @cargo run --quiet --bin compare-cu -- --raw rust-decimal

hylo-fix:
    @cargo run --quiet --bin compare-cu -- --raw hylo-fix

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
