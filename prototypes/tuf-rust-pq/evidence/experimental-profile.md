# Experimental RFC 9980 profile fixture

The deterministic fixture in `tests/composite_metadata.rs` records only facts
already exercised by this prototype:

- TUF specification version `1.0.36`.
- RFC 9980 OpenPGP v6 algorithm 30 with Ed25519 and ML-DSA-65 components,
  signed with SHA-512.
- Canonical TUF JSON serialization and SHA-512 role signing bytes.
- SHA-256 hash and length descriptors for serialized metadata and targets.
- A verified OpenPGP signing-key fingerprint as the composite threshold
  identity, with one identity counted at most once.

The fixture is closed over its fields and rejects an undefined profile field.
Its `lifecycle_fog` list records questions that the next lifecycle increment
must settle without silently turning them into this profile’s production policy:
accepted time, consistent-snapshot behavior, expiry, root bootstrap, and
rollback. This document and fixture establish no transport, credential,
consumer, release, or production authority.
