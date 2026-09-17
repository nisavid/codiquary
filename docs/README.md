# Documentation

The documentation system will serve five goal-first routes: understand the
system, adopt it, operate it, integrate or extend it, and investigate or
respond to failures. Each route may use tutorials, how-to guides, reference,
and explanation as appropriate.

Normative behavior belongs in `contracts/`. Documentation explains and links
that behavior; it does not redefine it. Executable examples should come from
`fixtures/` and `conformance/` so copied prose cannot drift from tested results.

- `adr/` records accepted architectural decisions.
- `agents/` documents repository mechanics for coding agents.
- `provenance/` records the reviewed source decisions and ownership handoff.
- `repository-controls/` documents contribution and repository-admission
  controls without granting release or deployment authority.

The experimental Tough exact-target proof has one repo-carried procedure:
[`agents/replay-exact-target-tough-proof.md`](agents/replay-exact-target-tough-proof.md).
It is not a production or release procedure.
