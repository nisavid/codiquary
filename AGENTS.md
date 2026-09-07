# Repository instructions

## Project context

Read `CONTEXT.md` before planning or editing. Use its canonical terms in code,
contracts, tests, documentation, issues, and commits. Keep `README.md` as the
verified human entrypoint and put detailed contracts and decisions under
`docs/`.

When introducing or revising domain concepts, follow the
[shared domain naming convention](https://github.com/nisavid/dotfiles/blob/main/docs/research/CRYPTO_RELEASE_OPS_NAMING.md#domain-naming-convention).

The repository is a pre-implementation scaffold. The
[substrate decision](https://github.com/nisavid/codiquary/issues/2) selects one
root Rust 2024 package and library named `codiquary`, with `cophax` as the
operator command. The
[build-skeleton ticket](https://github.com/nisavid/codiquary/issues/8) owns the
initial build code. Command grammar, wire schemas, and adapter transports
remain with their corresponding Wayfinder decisions.

## Authority boundary

Public, reusable contracts, implementations, adapters, profiles, fixtures,
conformance, qualification records, documentation, governance, and releases
belong here. Private deployment bindings, hosts, accounts, provider resources,
secret references, protected state, ceremonies, production mutations, and
acceptance evidence belong to the system user.

- Never add a plaintext secret, private key, credential, production identifier,
  private deployment value, or live-provider payload.
- Use disposable, value-free fixtures for every test and example.
- Do not access or mutate production keys, providers, hosts, DNS, release state,
  or protected consumers from repository work.
- Generic verification cannot authorize installation or execution.
- Do not invent cryptographic primitives. Use established standards and reviewed
  implementations after the owning design and qualification tickets settle them.

## Issue tracker

The [Codiquary implementation map](https://github.com/nisavid/codiquary/issues/1)
and its native sub-issues and dependency edges are the source of truth. Claim a
Wayfinder ticket before working it. Refer to tickets by their linked titles in
human-facing prose.

Use the canonical `needs-triage`, `needs-info`, `ready-for-agent`,
`ready-for-human`, and `wontfix` labels. Wayfinder tickets use exactly one of
`wayfinder:research`, `wayfinder:prototype`, `wayfinder:grilling`, or
`wayfinder:task`; maps use `wayfinder:map`.

## Git and validation

This is a personal `nisavid` project. Use `Ivan D Vasin <ivan@nisavid.io>` for
Git work and the `nisavid` GitHub account for repository mutations. Prefix
branches with `ivan/`. Use Conventional Commits for commits and pull-request
titles. Sign off every commit under the DCO with `git commit --signoff`.

Before committing or publishing, run:

```sh
./scripts/check-repository.sh
git diff --check
```

Run every additional test or conformance suite that owns the changed surface.
Report what the checks actually prove; a green repository-policy workflow does
not prove release safety or production authority.

## Change policy

- Keep one primary core distribution until a demonstrated dependency,
  privilege, platform, or release-cadence seam justifies another.
- Normative or security-boundary changes require an ADR, threat-model review,
  conformance changes, compatibility analysis, and explicit maintainer approval.
- Core changes test against the existing contracts. Adapter-interface changes
  include migration evidence. Material adapter changes renew affected
  qualification.
- Documentation and examples cannot redefine normative behavior. Derive
  executable examples from fixtures and conformance assets.
- Before 1.0, incompatible changes require an explicit deployment-lock update
  and renewed affected qualification; never rewrite a user's accepted lock.
- Define the current increment before deepening it. Settle every valid finding
  within that increment, and record later branches as follow-up rather than
  broadening the claim.

## Pull requests

Use the pull-request template. State the owning issue, public authority surface,
provenance, compatibility effect, and exact validation run. Release, provider,
host, or production actuation requires separate explicit authority.
