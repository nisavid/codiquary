# Codiquary context

## Purpose

Codiquary defines a provider-neutral system for authorizing, publishing,
verifying, withdrawing, and recovering software releases. It keeps candidate
construction separate from release authority and keeps generic verification
separate from a consumer's installation and execution authority.

## Project identity

- **Codiquary**: the release-operations project for authenticated software
  artifacts.
- **Codophylax**: Codiquary's personified public operator across interfaces;
  `cophax` (“co-fax”) is its prospective CLI/TUI command.
- **Sacrysty**: the separate reusable cryptographic-operations project. Its
  public operator is Sacrystan, with the prospective command `sacryd`
  (“sacred”).

## Canonical model

- **Core contract**: provider-neutral normative rules and machine-readable
  artifacts.
- **Core implementation**: the maintained primary distribution implementing the
  core contract.
- **Adapter interface**: a narrow role-specific interface owned by the core
  contract.
- **Adapter**: an implementation of one adapter interface.
- **Policy profile**: a public, complete, non-secret document selecting
  compatible contracts, required capabilities, and additional policy for one
  system user.
- **Deployment binding**: private values connecting a policy profile to real
  hosts, accounts, resources, and opaque secret references.
- **Qualification record**: immutable evidence that exact named versions,
  capabilities, platforms, and policy passed a defined evaluation.
- **Release authority**: the positive authority that accepts immutable intent
  and authorizes a release.
- **Status authority**: a separate subtractive authority that may withdraw,
  freeze, or report indeterminate state but cannot create a valid release.
- **Protected consumer**: a consumer that applies its own admission,
  installation, execution, archive, and updater policy after generic
  verification succeeds.

## Invariants

1. No adapter, provider, maintainer workflow, or candidate can create authority
   that the normative contract does not grant.
2. Every non-success grants no authority. An indeterminate mutation is
   reconciled before retry or failover.
3. Public configuration may name opaque secret slots but never resolved secrets.
4. Normative shapes are closed by default. Unknown required security data is
   unsupported, never silently ignored or downgraded.
5. Published artifacts and evidence bind exact immutable bytes and versions.
6. A system user's private binding never becomes generic semantics.
7. Generic verification is necessary but insufficient for protected consumer
   acceptance.

## Current stage

The repository contains governance and a non-operational Rust build skeleton.
It has no release-operations implementation, released package, production
configuration, key, provider binding, or accepted release.
