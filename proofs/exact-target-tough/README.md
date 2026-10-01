# Disposable exact-target Tough proof

This unpublished nested package contains the frozen dependency checkpoint and
fixture author for the experimental `exact-target-v1` investigation. It uses
the reviewed offline executor to generate public, synthetic inputs.

The current corpus is under
[`fixtures/conformance/exact-target-v1/`](../../fixtures/conformance/exact-target-v1/).
Its manifest binds the generation revision and literal expected observations.
The generated public output passed static inspection; no verifier or helper
behavior has been observed yet.

The first two Tough patches contain dependency declarations only. Algorithm-30
verification and the fixed test clock remain to be implemented through the
first failing behavior test and its smallest passing correction.

Follow the single procedure in
[`docs/agents/replay-exact-target-tough-proof.md`](../../docs/agents/replay-exact-target-tough-proof.md).
Every build or test command requires a reviewed source and command binding to
the offline executor. The package remains outside Codiquary's root package,
production dependency graph, public API, and release authority.
