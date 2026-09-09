# Experimental RFC 9980 profile fixture

The deterministic JSON fixture and closed types in
`src/experimental_profile.rs` record only facts measured by
`tests/composite_metadata.rs` through the existing Tough and Sequoia public
interfaces:

- Every exercised root, targets, delegated-targets, snapshot, and timestamp
  role carries TUF specification version `1.0.36` and is signed over Tough's
  canonical TUF JSON bytes.
- The serialized TUF key uses exact wire labels `openpgp-rfc9580` for
  `keytype` and `openpgp-rfc9980-ml-dsa-65+ed25519-sha512` for `scheme`.
- The accepted OpenPGP certificate is v6. Its detached signature is one
  canonical unarmored v6 binary packet: signature type 0, public-key algorithm
  30, SHA-512, one v6 issuer fingerprint, and Ed25519 plus ML-DSA-65 components.
  Corrupting either component fails verification.
- Serialized snapshot and timestamp `Metafile` descriptors carry `length` and
  `hashes.sha256`. The delegated target carries required `length` and
  `hashes.sha256`. SHA-512 is the role-signature digest; SHA-256 is the metadata
  and target descriptor digest.
- Opaque target `custom` data survives serialization and delegated target
  lookup. Undefined fields on the provisional OpenPGP key object or its
  `keyval` are rejected during metadata ingestion, including on unused keys.
- A verified v6 OpenPGP signing-key fingerprint is the composite threshold
  identity. One signing identity contributes at most once even when distinct
  authorized TUF key IDs project the same signing key.

The parser denies unknown fields at every profile level and rejects wrong
fixed values, missing or duplicate fields, wrong types, incomplete lifecycle
fog, and unknown nested fields. The accepted typed value serializes back to the
exact fixture bytes.

The `lifecycle_fog` array records accepted time, consistent-snapshot behavior,
expiry, root bootstrap, and rollback as unresolved questions. It selects no
expiry duration, clock source, rollback rule, bootstrap mechanism, or
consistent-snapshot policy. Durable refresh, crash recovery, held-byte policy,
transport, consumer admission or installation, credentials, release authority,
and production authority are also outside this evidence.

The independent issue-21 reviews covered commit
`59833f8a1f189a3b512b22ba9aa21f598333055d` and returned concerns. They are
historical findings against that candidate, not approval of this correction.
The coordinator owns the later independent pass over the final revision.
