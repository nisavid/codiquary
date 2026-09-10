# Documentation

Use these pages to understand Codiquary, inspect prototype evidence, or prepare a contribution. Normative behavior belongs in `contracts/`; documentation explains and links it without redefining it.

## Understand Codiquary

- [Project context](../CONTEXT.md) defines the canonical model, authority boundaries, and planned stages.
- [Reusable release-trust handoff](provenance/reusable-release-trust-handoff.md) records the reviewed source and ownership handoff.
- [Architecture decision record policy](adr/) explains the required record shape. No ADR has been accepted yet.

## Examine the TUF experiment

- [Prototype orientation](../prototypes/tuf-rust-pq/) routes readers through the RFC 9980 and consumer/transport work.
- [Consumer and transport findings](../prototypes/tuf-rust-pq/docs/consumer-transport.md) explains what the experiment establishes and what an adoption decision would still require.
- [Replay the consumer and transport checks](../prototypes/tuf-rust-pq/docs/replay-consumer-transport.md) gives the prepared-input replay procedure.
- [Consumer and transport evidence](../prototypes/tuf-rust-pq/evidence/consumer-transport.md) records inputs, results, observations, limits, and test mappings.

## Contribute

- [Contributing to Codiquary](../CONTRIBUTING.md) covers scope, evidence, DCO, and pull requests.
- [Validate the Rust scaffold](how-to/validate-scaffold.md) covers the root build placeholders.
- [Agent documentation](agents/) covers repository mechanics used by coding agents.
