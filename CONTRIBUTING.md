# Contributing

## Adding a New Math Library

Each benchmarked library lives in its own workspace member under `crates/`.
This is intentional: `svm-unit-test` generates a temporary SBPF crate that
depends only on the package containing the test. Keeping one library per package
prevents unrelated math dependencies from being linked into the generated test.

1. Add a new member under `crates/`, for example:

   ```text
   crates/cu_my_math/
   ├── Cargo.toml
   ├── src/lib.rs
   └── tests/my_math.rs
   ```

2. Add the member to the root `Cargo.toml` workspace `members` list.

3. Add the library dependency in `[workspace.dependencies]` if it should be
   shared, or directly in the new member's `Cargo.toml`.

4. Re-export the library from the member facade:

   ```rust
   #![no_std]

   pub use my_math;
   ```

5. Import through the facade inside the test file:

   ```rust
   use core::hint::black_box;
   use cu_my_math::my_math::SomeNumber;
   use svm_unit_test::svm_test;
   ```

   Do not import the external crate directly from the test file. The generated
   SBPF crate will not have that dependency in its own `Cargo.toml`.

## Test Coverage Shape

Add several small `#[svm_test]` functions rather than one large benchmark. Use
stable operation names so comparison output remains readable:

- `add_sub_mul_div`
- `checked_arithmetic`
- `basis_points`
- `powers_and_sqrt`
- `exp` or `exp_approximation`

Basis point tests should use `MAX_BPS = 10_000`.

Use `black_box` around inputs and outputs so the compiler cannot trivially remove
the work being measured.

## `no_std` and `std` Libraries

Most libraries should use upstream `svm-unit-test`:

```toml
[dev-dependencies]
svm-unit-test.workspace = true
```

If the library requires `std`, the upstream generated wrapper may conflict with
its custom panic handler. In that case, use the local std-compatible wrapper:

```toml
[dev-dependencies]
svm-unit-test = { package = "svm-unit-test-std", path = "../svm_unit_test_std" }
```

Only use this when required. Prefer upstream `svm-unit-test` for normal
`no_std`-friendly libraries.

## Wiring CLI Comparison

Update `src/bin/compare-cu.rs`:

```rust
(
    "my-math",
    Library {
        canonical: "my_math",
        package: "cu_my_math",
        test_target: "my_math",
    },
),
```

The entry fields are:

1. canonical library key used in table columns,
2. Cargo package name,
3. integration test target name.

If your operation names differ from existing categories, either rename the test
or add an entry to `operation_alias`.

Update `justfile` with a direct recipe:

```just
my-math:
    @cargo run --quiet --bin compare-cu -- --raw my-math
```

## Verifying Changes

Run host compilation first:

```sh
cargo test --workspace --tests --no-run
```

Run the new library test:

```sh
just my-math
```

Run a comparison:

```sh
just compare fixed my-math
```

Save CSV output:

```sh
just compare-save fixed my-math
```
