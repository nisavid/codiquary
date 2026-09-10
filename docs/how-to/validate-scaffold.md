# Validate the Rust scaffold

Use this guide to build and check the non-operational root Rust 2024 package. These checks validate the scaffold and repository policy; they do not qualify a release implementation or deployment.

## Prerequisites

Use an existing Rust 1.98.1 toolchain that provides `rustc`, Cargo, and rustdoc. Git and the shell utilities required by `scripts/check-repository.sh` must also be available. Do not install tools as part of this procedure.

Run the commands from the repository root without network access.

## Build and test

```sh
rustup run 1.98.1 cargo build --frozen
rustup run 1.98.1 cargo test --frozen
```

The root package has no external dependencies or behavioral tests. A successful test run reports zero tests and establishes only that the placeholder harness builds and runs. `Cargo.lock` and the offline Cargo configuration are committed; a vendored source tree will be needed when dependencies are introduced.

The [Rust build workflow](../../.github/workflows/rust-build.yml) runs the harness on Linux x86_64 GNU and macOS Apple Silicon. Those jobs are build checks, not minimum-OS qualification, reproducible-release evidence, or an authenticated release.

## Check repository policy

```sh
./scripts/check-repository.sh
git diff --check
```

A clean result establishes the checked source shape and repository policy only. It does not establish release contracts, operational behavior, authority, publication, or deployment acceptance.
