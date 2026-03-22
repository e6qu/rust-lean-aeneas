# Contributing

Thank you for your interest in contributing to the Rust + Lean 4 Formal Verification tutorial series.

## Getting Started

1. Read the [README.md](README.md) for a project overview.
2. Work through [Tutorial 01](tutorials/01-setup-hello-proof/README.md) to set up the toolchain.
3. Review the [PLAN.md](PLAN.md) to understand the project roadmap.

## How to Contribute

### Reporting Issues

- Open a GitHub issue describing the problem, including which tutorial and file are affected.
- For proof issues, include the Lean error output from `lake build`.

### Submitting Changes

1. Fork the repository and create a feature branch.
2. Follow the existing code style and project conventions (see [PLAN.md](PLAN.md) for design decisions).
3. Ensure all Rust code compiles (`cargo test` in the relevant `rust/` directory).
4. Ensure all Lean proofs check (`lake build` in the relevant `lean/` directory).
5. Submit a pull request with a clear description of the change.

### Tutorial Conventions

- Rust code must be Aeneas-compatible (see [AENEAS.md](AENEAS.md) for supported features).
- Use `Vec<u8>` instead of `String` in verified core code.
- Use explicit `while` loops instead of iterators.
- Each tutorial README should be self-contained and followable in order.

### Reference Documents

- [LEAN.md](LEAN.md) -- Lean 4 reference
- [AENEAS.md](AENEAS.md) -- Aeneas reference

## License

By contributing, you agree that your contributions will be licensed under the [MIT License](LICENSE).
