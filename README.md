# Math Compare

Benchmark Solana SBF compute-unit usage across Rust math libraries.

Each math library is tested in its own crate under `crates/` so generated
`svm-unit-test` programs only depend on the library being measured.

## Libraries

- `fixed`
- `bigdecimal`
- `rust-decimal`
- `hylo-fix`
- `spl-math`

## Running Tests

Run all workspace tests:

```sh
just test
```

Run one library and print only CU readings:

```sh
just fixed
just bigdecimal
just rust-decimal
just hylo-fix
just spl-math
```

Compare libraries:

```sh
just compare bigdecimal fixed rust-decimal hylo-fix spl-math
```

Or compare between all libraries:

```sh
just compare-all
```

Save comparison output as CSV:

```sh
just compare-save bigdecimal fixed rust-decimal hylo-fix spl-math
```

Compare and save output of all libraries:

```sh
just compare-save-all
```

## Notes

Most library crates use upstream `svm-unit-test`. `bigdecimal`, `hylo-fix`, and
`spl-math` use the local `svm-unit-test-std` wrapper: BigDecimal needs its
allocator-backed runtime path, while the other generated SBF tests pull in
`std`, which conflicts with the upstream no-std panic handler.

## Contributing

See [CONTRIBUTING.md](CONTRIBUTING.md).
