# Codiquary

Codiquary is being built to help software teams publish exact release bytes, verify who authorized them and whether they remain valid, and withdraw or recover releases without confusing verification with permission to install or run them.

Codiquary is experimental and not yet available as a usable release system. The repository contains governance, a non-operational Rust scaffold, and disposable prototypes; the [implementation map](https://github.com/nisavid/codiquary/issues/1) tracks the path to the first authenticated self-release.

## Explore the project

- Start with the [project context](CONTEXT.md) to understand the release model, authority boundaries, and planned stages.
- Use the [documentation landing page](docs/) to find explanations, evidence, and contributor guides.
- Inspect the [RFC 9980 TUF Rust prototype](prototypes/tuf-rust-pq/) for synthetic experiments with composite OpenPGP signatures, retained target bytes, loopback transport, and native package-verification fixtures.
- Follow the [implementation map](https://github.com/nisavid/codiquary/issues/1) for planned work and dependency order.

Codiquary separates release authorization, withdrawal and freeze authority, candidate production, generic verification, and protected-consumer policy. Generic verification never grants permission to install or execute an artifact. The [project context](CONTEXT.md) describes these responsibilities; [GOVERNANCE.md](GOVERNANCE.md) defines project authority and maintenance rules.

Codophylax is the project's public operator identity. Its prospective CLI/TUI command is `cophax` (“co-fax”). The root Rust 2024 package contains non-operational `codiquary` library and `cophax` executable placeholders.

## Contribute

Start with [CONTRIBUTING.md](CONTRIBUTING.md) for the public/private boundary, evidence expectations, DCO sign-off, and pull-request requirements. Contributors working on the scaffold can use [Validate the Rust scaffold](docs/how-to/validate-scaffold.md).

Report security concerns through [SECURITY.md](SECURITY.md). Before 1.0, the project promises no release cadence, compatibility window, platform-support term, or remediation deadline.
