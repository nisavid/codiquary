# TUF research

These reports retain the public research that informed Codiquary's TUF work.
Their conclusions and proposed next steps describe the source revisions below.
Each imported report begins with a dated context note; every byte after that
note is the original report, including its source links and evidence limits.

| Report | Source date | Original contribution |
| --- | --- | --- |
| [TUF 1.0.36 conformance and security delta](CRYPTO_RELEASE_OPS_TUF_CONFORMANCE.md) | 2026-09-07 | [Establish the release trust TUF conformance delta](https://github.com/nisavid/dotfiles/pull/275) |
| [Rust and post-quantum TUF feasibility](CRYPTO_RELEASE_OPS_TUF_PQ_FEASIBILITY.md) | 2026-09-08 | [Evaluate Rust post-quantum TUF integration](https://github.com/nisavid/dotfiles/pull/277) |

The [TUF 1.0.36 optional mirrors clarification](TUF_1_0_36_MIRRORS_ERRATUM.md)
corrects the conformance report's repository-surface inventory and records its
effect on current Codiquary work without changing the retained source.

## Later decisions and evidence

The [conditional TUF adoption decision](https://github.com/nisavid/codiquary/issues/18#issuecomment-5624996832)
records the direction after these reports: the accepted non-TUF baseline remains
the current authority until the qualification gate passes, followed by one
explicit cutover to TUF as the sole current-release authority plane. Retaining
the earlier reports does not pass that gate or reopen the decision.

The [Rust/RFC 9980 feasibility result](https://github.com/nisavid/codiquary/issues/16#issuecomment-5605946809)
and [consumer/transport result](https://github.com/nisavid/codiquary/issues/17#issuecomment-5622220888)
record the subsequent bounded prototype evidence. The later
[Rust TUF capability comparison](https://github.com/nisavid/codiquary/blob/e4f8b746d1e0b4383d5dfd7bdae1cc1976e573a1/docs/research/rust-tuf-exact-target-capabilities.md)
is a separate retained report, reviewed through
[Compare maintained Rust TUF implementations for the accepted exact-target contract](https://github.com/nisavid/codiquary/issues/37#issuecomment-5692539300).

Use the [implementation map](https://github.com/nisavid/codiquary/issues/1) for
current work and dependencies. The historical reports' fixture inventories and
experiment proposals do not establish executable conformance, dependency
acceptance, qualification, or installation authority. Library and protocol claims
remain bound to their stated versions and original evidence; this import adds
no new technical assessment.

## Migration provenance

Both reports retain their original filenames under `docs/research/`. The
transformation is a copy with a prefixed context note, with no translation or
revision of the research text. The SHA-256 values below identify the original
UTF-8 file bytes, excluding each new context note.

| Report | Immutable source | Original file SHA-256 |
| --- | --- | --- |
| Conformance and security delta | [`82d39dfcf845b8959d891229f5aa697084b8ddac`](https://github.com/nisavid/dotfiles/blob/82d39dfcf845b8959d891229f5aa697084b8ddac/docs/research/CRYPTO_RELEASE_OPS_TUF_CONFORMANCE.md) | `ecaf2be6518ea8500ec095c20a37fe9eae231ecb5292980303e09e105249924a` |
| Rust and post-quantum feasibility | [`cfef8c04711e1f8dc1fcc7ded9d1fc5cd9a0370e`](https://github.com/nisavid/dotfiles/blob/cfef8c04711e1f8dc1fcc7ded9d1fc5cd9a0370e/docs/research/CRYPTO_RELEASE_OPS_TUF_PQ_FEASIBILITY.md) | `c0aad3d929d8436b02a2c4e2dba88034c1218aa7cdea26293619dd55692d7407` |

The source histories attribute both reports and their revisions to
Ivan D Vasin. The immutable source links preserve those contribution records.
Both source revisions carry the Unlicense
([conformance revision](https://github.com/nisavid/dotfiles/blob/82d39dfcf845b8959d891229f5aa697084b8ddac/UNLICENSE),
[feasibility revision](https://github.com/nisavid/dotfiles/blob/cfef8c04711e1f8dc1fcc7ded9d1fc5cd9a0370e/UNLICENSE)).
The reports' original dotfiles issue links identify their historical owners;
current public release-trust ownership follows the
[Codiquary handoff](../provenance/reusable-release-trust-handoff.md).
