# Codiquary

Codiquary is the public, provider-neutral home of a reusable release
operations system. It will provide the contracts and tooling needed to prepare,
authorize, sign, publish, verify, withdraw, and recover software releases while
keeping every authority boundary explicit and reviewable.

> [!IMPORTANT]
> This repository is in its genesis stage. It has no released package, supported
> runtime, production authority, signing key, provider binding, or trusted
> deployment. The [implementation map](https://github.com/nisavid/codiquary/issues/1)
> is the source of truth for reaching the first authenticated self-release.

## Ownership boundary

This repository owns public, reusable material:

- normative release, status, admission, and lifecycle contracts;
- one primary core implementation and its compact public interfaces;
- role-specific adapter interfaces and justified reference adapters;
- public, non-secret policy profiles;
- canonical and hostile fixtures, conformance suites, and qualification records;
- operator, verifier, integrator, contributor, and incident documentation; and
- project governance, compatibility declarations, and immutable releases.

Each system user owns its own deployment: exact profile selection and locks,
private bindings, hosts, accounts, provider resources, opaque secret references,
protected state, ceremonies, production mutations, and acceptance evidence.
Generic release verification never grants a protected consumer permission to
install or execute anything.

## Planned system surfaces

- **Release authority** evaluates immutable intent and produces explicit
  positive authorization artifacts.
- **Status authority** independently withdraws, freezes, or reports
  indeterminate state without creating a valid release.
- **Protected producer** prepares deterministic candidate bytes and provenance
  without receiving signing or release authority.
- **Verifier** checks signatures, grants, freshness, history, and exact bytes
  independently from acquisition.
- **Protected consumer** combines generic verification with its own admission,
  installation, and execution policy.
- **Publication adapters** expose canonical state and mirrors without becoming
  cryptographic authority.
- **Operator tools** guide explicit stages and report denial, failure, or
  possible effects without inventing hidden session authority.

Codophylax is the public operator, with the future CLI/TUI command `cophax`
(“co-fax”). One root Rust 2024 package contains the `codiquary` library and
`cophax` executable. Both are non-operational build placeholders: the library
has no public API, and the command reports that it is not implemented and exits
with failure. Command grammar and wire schemas remain with their owning design
tickets. Worker and documentation names remain unassigned.

## Repository layout

- [`contracts/`](contracts/) will contain normative serialized contracts and
  lifecycle rules.
- [`src/`](src/) contains the single primary distribution's build placeholders.
- [`adapters/`](adapters/) defines role-specific interfaces and reference
  implementations only where a real seam exists.
- [`profiles/`](profiles/) contains complete public policy profiles without
  private deployment values.
- [`fixtures/`](fixtures/) and [`conformance/`](conformance/) own executable
  canonical and hostile examples.
- [`qualification/`](qualification/) contains immutable evidence and prospective
  status records.
- [`release/`](release/) owns versioned release metadata; it currently contains
  no release-manifest instance.
- [`docs/`](docs/) contains architecture, decisions, provenance, and the future
  Diataxis documentation system.

## Build and validate the scaffold

Install the exact toolchain selected by [`rust-toolchain.toml`](rust-toolchain.toml),
then build and run the test harness without network access:

```sh
rustup toolchain install 1.98.1 --profile minimal --no-self-update
cargo build --frozen
cargo test --frozen
```

There are no dependencies or behavioral tests yet. `Cargo.lock` and the offline
Cargo configuration are committed; a vendored source tree is needed only when
actual dependencies are introduced. A successful test run currently reports
zero tests and proves that the placeholder test harness builds and runs.

The [Rust build workflow](.github/workflows/rust-build.yml) builds and runs that
harness natively on Linux x86_64 GNU and macOS Apple Silicon, recording the
actual compiler and runner versions. These are build checks, not minimum-OS
qualification, reproducible-genesis evidence, or an authenticated release.

Run the language-neutral repository checks:

```sh
./scripts/check-repository.sh
```

These checks validate repository policy only. They do not prove a release
contract, implementation, authority, deployment, or release.

## Project status

Every public interface is experimental before 1.0. Releases are need-driven.
The project currently promises no release cadence, compatibility or deprecation
window, backports, remediation deadline, platform-support term, or post-1.0
stability. See [GOVERNANCE.md](GOVERNANCE.md),
[CONTRIBUTING.md](CONTRIBUTING.md), and [SECURITY.md](SECURITY.md).
