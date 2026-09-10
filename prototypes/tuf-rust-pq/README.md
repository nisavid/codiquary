# RFC 9980 TUF Rust prototype

This disposable Rust 2021 prototype investigates two related questions:

1. Can a pinned Tough and Sequoia experiment sign and verify synthetic TUF metadata with an RFC 9980 ML-DSA-65 and Ed25519 composite OpenPGP signature while preserving threshold identity and closed extension-field handling?
2. Can file and loopback HTTP acquisition carry TUF-verified targets through retained policy and representative PKGBUILD or pacman native signature checks before a fake consumer receives the same bytes, with six-field selection-substitution evidence supplied by the makepkg path?

The prototype uses public synthetic fixtures in a nonprivileged environment. It is not a reusable Codiquary implementation, package repository, updater, transport adapter, installation path, or production security design.

> [!CAUTION]
> Do not use production keys, identities, trust stores, metadata, repositories, credentials, or candidate content with this prototype. Its native fixtures use conventional synthetic signatures and do not qualify pacman or makepkg for post-quantum cryptography.

## Choose what you need

- To understand the findings and remaining adoption work, read [Consumer and transport findings](docs/consumer-transport.md).
- To repeat the checks from prepared and inspected inputs, use [Replay the consumer and transport checks](docs/replay-consumer-transport.md).
- To audit inputs, commands, results, fixtures, tools, limits, and test mappings, consult [Consumer and transport evidence](evidence/consumer-transport.md).
- For the experimental composite profile, see [Experimental profile evidence](evidence/experimental-profile.md).
- For patched-Tough behavior, see [Current Tough compatibility evidence](evidence/current-compatibility.md).

## Evidence boundaries

The consumer/transport extension has a recorded Linux run of 37 passing tests, strict Clippy, formatting, and repository-policy checks. Those results support only the experimental claims documented here; they do not accept or adopt TUF as a Codiquary release design.

The accepted predecessor evidence is bound to [commit `fddd7f564737ed5d213061a0c8621141bd85a2e9`](https://github.com/nisavid/codiquary/commit/fddd7f564737ed5d213061a0c8621141bd85a2e9). Its retained [draft pull request #19](https://github.com/nisavid/codiquary/pull/19) remains unmerged. That predecessor is historical and distinct from the consumer/transport source set.

Earlier evidence retains its original identities and claims:

- [Publisher/client binding](evidence/issue-23-publisher-client-binding.md)
- [Interrupted-refresh observation](evidence/issue-22-interrupted-refresh.md)
- [Source and build inspection](evidence/source-build-inspection.md)
- [Threshold-identity record](evidence/slice-5-threshold-identity.md)
- [Original red/green record](evidence/slice-3-red-green.md)
- [Cycle 2 record](evidence/slice-8-cycle-2.md)

[The consumer/transport checksum manifest](evidence/consumer-transport-inputs.sha256) binds 52 code, fixture, lock, and patch inputs. The manifest file's SHA-256 digest is `fae03ef90d47a704a3f2dec31ea3b9d28fa309d8bb946df8c2a882be417f19f6`.
