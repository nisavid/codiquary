# Current Tough compatibility evidence

At executable input revision
`298fd26fe44a759da54d3fe8ef9fb34446c4c981`, the corrected maintenance
replay passed the focused 15-test prototype suite and the applicable Tough
`--no-default-features` suite. This is narrow compatibility evidence for the
shared metadata-ingestion and threshold-verification maintenance surface. It is
not a conventional-cryptography qualification, lifecycle certification,
adoption decision, or parent-wide review. Final-revision verification and the
focused independent maintenance review are tracked in
[Correct and replay the current Tough compatibility record](https://github.com/nisavid/codiquary/issues/24).

## Exact inputs

- Executable input revision
  `298fd26fe44a759da54d3fe8ef9fb34446c4c981`
- Tough commit `98d8eb8b2ce63515d9b4981c938ef6453c5b5771`, tree
  `2eb4bd2fc529460a9f0e021dae88861a3632e1a9`
- Sequoia commit `0b0c8c7f038b829de2da0d28a822941d8600f3ee`, tree
  `85b7646f4bbf4a323beb03b4ead52b4db6f9ab64`
- Tough source patch SHA-256
  `8540324f3cd231ca244928024b2b1eea92ec2e16433187b2e4b708696b8e50dc`
- Tough test-workspace source override SHA-256
  `8720ad3dd63c05109761b624922248b94a938205a1d43987032d93e73377100c`
- Replay-layout Tough workspace manifest SHA-256
  `dd5807256002ffa16dfa4eba7c7db03ee3ba7daed2a74a9099b3496eda2314ba`
- Ordinary Cargo-resolved Tough test lock SHA-256
  `8951066c56b6f1fbbc391aedcdf6e15322f88356ff0f2d4d04b3ebf926fbe268`
- Patched Tough crate manifest SHA-256
  `8b4c3d4803ed2e0fa4250fd7e9069b628d537b67122f999e6ae78a35118cbb84`
- Focused prototype test source SHA-256
  `f4a6691f666b403cbffce59a9588a845f4d9333911a72c2de5b9d0e61e954a2f`

The executable-input patch above used the historical helper name
`validate_metadata_profile`. The current source uses
`validate_openpgp_extension_fields` with unchanged behavior; its current
checksum does not replace this revision-bound hash.

The pristine upstream Tough lock has SHA-256
`4614aae895dc084dc1abb34a13f3322f60456108a3abf529f52af246f6b5bfdb`.
It is not the replay lock. Selecting the pinned Sequoia checkout through the
workspace path override requires ordinary Cargo resolution. The stored replay
lock contains no source checkout, cache, or host path.

The replay-layout workspace manifest selects Sequoia 2.4.1 from the pinned
checkout and Tough 0.24.0 from the patched checkout. A historical executed
workspace manifest used a different relative checkout spelling and had
SHA-256 `93b0bc5e23558e9bc5475b05695a2359884bfa61a062f409bd1b9c25614fd2dc`;
that layout is not the current replay input.

## Maintenance surface and coverage

The patch changes the shared Tough maintenance surface in four connected
places:

- The metadata key-map deserializer structurally validates every OpenPGP key,
  including unused keys: it accepts only recognized wire labels and decodable
  public bytes, rejects undefined outer and `keyval` fields through
  `validate_openpgp_extension_fields`, and requires a matching, nonduplicate
  key ID.
- When an OpenPGP key is selected, signature verification additionally checks
  canonical certificate and signature encodings, the v6 algorithm-30 and sole
  eligible signing-key constraints, the SHA-512 binary signature and issuer
  fingerprint, and successful composite verification.
- `Key::verify` returns a threshold identity after successful verification.
  Existing RSA, Ed25519, and ECDSA keys retain their authorized TUF key ID;
  the provisional composite OpenPGP key uses its verified v6 signing-key
  fingerprint.
- Public root and delegation threshold verification count each returned
  identity at most once.

The 15 prototype tests cover the experimental-profile constraints, composite
component rejection, canonical certificate and signature encodings, undefined
OpenPGP fields at ingestion and verification, root and delegation threshold
identity, the all-role metadata chain, and conventional-key extra-field
compatibility. The two conventional ECDSA tests are:

- `root_retains_conventional_ecdsa_tuf_id_threshold_identity`
- `delegations_retain_conventional_ecdsa_tuf_id_threshold_identity`

Each serializes and reparses an accepted key map containing two ECDSA key
objects with the same public key and distinct correctly derived TUF key IDs.
Two valid signatures, one under each authorized ID, satisfy threshold 2. One
signature does not, and repeating an exact signature key ID is rejected. This
locks the existing behavior of returning the authorized TUF key ID from
`Key::verify`; it does not claim that the two objects represent independent
signing keys or qualify other conventional algorithms and encodings.

The applicable Tough replay verifies conventional RSA signatures,
reference-repository Ed25519 signatures, and ingestion of three ECDSA metadata
encodings. It has no ECDSA signature-verification test, so the two focused
prototype tests cover that changed identity path through public
`Root::verify_role` and `Delegations::verify_role` seams. They do not expand
Tough's accepted conventional parsing boundary.

## Executable-input results

The receipts in this section are immutable provenance for executable input
revision `298fd26fe44a759da54d3fe8ef9fb34446c4c981`; they are not the separate
final-candidate verification receipts recorded in
[Correct and replay the current Tough compatibility record](https://github.com/nisavid/codiquary/issues/24).

The prototype command passed 15 tests, failed 0, and ignored 0. Its raw
command/output SHA-256 values are
`9fd26dad372c57b5eb132152791a1293ebab3a6188aec509bf75437ba34dae26`
and `f8b74dee0be02ae213821f980c61483779f1a0cd6b7d4790adf723c81a3f4b5c`.

The Tough command passed 78 tests, failed 0, and ignored 1. Its raw
command/output SHA-256 values are
`98ca301371e3230dbba5d3b4f1950bea1556b0f31adfadc8017b15386f499bcf`
and `0417a6f6306a0d22f04e728fba000fd35d6d735c8bc31b5ea9624564a2dec8b6`.

Both commands used direct Cargo, rustc, and rustdoc 1.98.1 paths. Their reported
versions and executable-file SHA-256 values are:

- Cargo: `cargo 1.98.1 (797e8a9bc 2026-08-05)`,
  `da77c8b33849312255ccde3179198ada4c8deb370488d050286146b1d1b27e14`
- rustc: `rustc 1.98.1 (48a229cea 2026-09-01)`,
  `859254978c0a0402c32f949f6de0d99aee73be8d15f45aac00ae1448aac51e74`
- rustdoc: `rustdoc 1.98.1 (48a229cea 2026-09-01)`,
  `3ff7fed6d1064d8d75d24ae8beff9fe1d68031d3d819966481d22c3c9002b709`

The host supplied OpenSSL 3.6.3 on `x86_64-unknown-linux-gnu`. Bubblewrap used
`--unshare-all`; the parent and sandbox network namespace identifiers differed.
Only the disposable Cargo home, target, and temporary directories were
writable host-backed paths whose writes persisted outside the sandbox.
Bubblewrap also mounted private `/dev` and `/proc`; no `HOME` setting or fourth
writable host-backed path was used.

No Tough feature was enabled. The `http`, `http2`, `integ`, and `fips` feature
profiles were excluded. The zero-test HTTP target did not exercise HTTP
behavior, and the installed `noxious` and integration-support packages did not
expand the invoked package's feature boundary. macOS was not executed.

## Inspection boundary

The prototype inventory contains 189 packages: 184 registry packages and five
path packages, with 181 packages reachable in the executable graph. It includes
30 custom-build packages and 16 proc-macro packages. The Tough inventory
contains 263 reachable packages, including 258 registry packages, 39
custom-build packages, and 19 proc-macro packages. All selected registry
archives matched their active lock checksums, and comparison with the retained
inspected registry sources found no missing, mismatched, or extra source trees.

Every selected registry source tree was byte-identical to the retained
historically inspected tree. The path-package trees, locks, target, Tough patch,
workspace override, manifests, and `--no-default-features` boundary also
matched. This stronger direct byte comparison allowed reuse of the retained
feature-sensitive source/build inspection even though some historical
inventory-digest files were unavailable. The retained new-package TSV still
matched its recorded digest.

The inspection was bounded source/build review, not a full security audit. A
future replay must compare the actual source bytes and resolved graph with this
inventory. Any source, manifest, lock, target, feature, build-script,
proc-macro, or package-selection delta must be inspected before dependency code
runs. See the [source/build inspection](source-build-inspection.md) and the
historical [Cycle 2 record](slice-8-cycle-2.md) for retained limits and
provenance.

## Replay

After reconstructing the pinned checkouts and applying the main source patch as
described in the README, verify and apply the test-workspace inputs:

```sh
sha256sum -c <<'CHECKSUMS'
8720ad3dd63c05109761b624922248b94a938205a1d43987032d93e73377100c  patches/tough-default-sequoia-source.patch
8951066c56b6f1fbbc391aedcdf6e15322f88356ff0f2d4d04b3ebf926fbe268  evidence/tough-default.Cargo.lock
CHECKSUMS

git -C .scratch/tough apply --check \
  ../../patches/tough-default-sequoia-source.patch
git -C .scratch/tough apply \
  ../../patches/tough-default-sequoia-source.patch
cp evidence/tough-default.Cargo.lock .scratch/tough/Cargo.lock
sha256sum -c <<'CHECKSUMS'
dd5807256002ffa16dfa4eba7c7db03ee3ba7daed2a74a9099b3496eda2314ba  .scratch/tough/Cargo.toml
8951066c56b6f1fbbc391aedcdf6e15322f88356ff0f2d4d04b3ebf926fbe268  .scratch/tough/Cargo.lock
CHECKSUMS
```

The repository's `.cargo/config.toml` sets `net.offline = true`. An empty-cache
fetch without an override exits 101. Override the setting only for the fetch;
do not change the repository configuration:

```sh
tuf278_root="$(pwd -P)"
tuf278_run="$(mktemp -d)"
tuf278_cargo="$(rustup which --toolchain 1.98.1 cargo)"
tuf278_toolchain_bin="$(dirname "$tuf278_cargo")"
mkdir -p "$tuf278_run/cargo-home" "$tuf278_run/target" "$tuf278_run/tmp"

env -i \
  PATH="$tuf278_toolchain_bin:/usr/bin:/bin" \
  CARGO_HOME="$tuf278_run/cargo-home" \
  CARGO_TERM_COLOR=never \
  CARGO_NET_OFFLINE=false \
  "$tuf278_toolchain_bin/cargo" fetch --locked \
    --manifest-path .scratch/tough/tough/Cargo.toml \
    --target x86_64-unknown-linux-gnu
```

Revalidate the graph and byte-equality evidence before execution. Then confirm
that Bubblewrap can create the required namespace boundary:

```sh
bwrap --ro-bind / / --dev /dev --proc /proc --unshare-all \
  --die-with-parent --new-session /usr/bin/true
```

Run the exact locked, offline Tough replay:

```sh
bwrap --ro-bind / / --dev /dev --proc /proc --unshare-all \
  --bind "$tuf278_run/cargo-home" "$tuf278_run/cargo-home" \
  --bind "$tuf278_run/target" "$tuf278_run/target" \
  --bind "$tuf278_run/tmp" "$tuf278_run/tmp" \
  --chdir "$tuf278_root" --die-with-parent --new-session \
  /usr/bin/env -i \
    PATH="$tuf278_toolchain_bin:/usr/bin:/bin" \
    RUSTC="$tuf278_toolchain_bin/rustc" \
    RUSTDOC="$tuf278_toolchain_bin/rustdoc" \
    CARGO_HOME="$tuf278_run/cargo-home" \
    CARGO_TARGET_DIR="$tuf278_run/target" \
    CARGO_TERM_COLOR=never \
    TMPDIR="$tuf278_run/tmp" \
    "$tuf278_toolchain_bin/cargo" test --locked --offline \
      --manifest-path .scratch/tough/tough/Cargo.toml \
      --package tough \
      --target x86_64-unknown-linux-gnu \
      --no-default-features
```

## Historical evidence

Earlier compatibility records remain useful as revision-bound history, not as
current qualification. A nine-test prototype run recorded command/output
SHA-256 values
`d94b2cd81ab9d24ee8f1de0bdecdc523875666a462e1fa15848990c3366bb915`
and `cfb0507862239fc091d4a36989aafa8ccf3761cf65ecc62cbbe2fc9767805940`.
A separate two-test focused run recorded
`6b17e5f0227ee6e6d74ee11fc0a0cc858bebe1bc67818ee568c5755c1826fcec`
and `7836dde79f86e8e03b76c29b3ff4d23d392a68a8eac9717a0656e435ce37e4f3`.
Its earlier source hash does not describe the present 15-test file; the current
source checksum is listed under Exact inputs. Historical clippy command/output
SHA-256 values are
`572a1e0c147cb74420e84531abcfcd3875860802f31d89999fe8759306f7e53d`
and `38a93aafab9e9b62498013cf8cab9bc6665b441191c7b1b4d10aee06ccb7059e`.

An earlier exact-toolchain Tough run passed 78 tests, failed 0, and ignored 1;
its command/output SHA-256 values were
`2904bc139ea944a3a59341b68fc2899aa31820c987808b6ebd5af948d57833d6`
and `995301fdff04d3c578fa6ba4ba58337d3b79da588885ec39b7e92be6349eb4d7`.
An earlier Rust and Cargo 1.98.0 run also passed 78 tests. Its command, first
full output, and later verification-output SHA-256 values were
`43afd683959bc5cdeae6bcdce5b0cf3729ed0c810947da3949a82037bf3e4057`,
`64cc71590b23bacc59be7a403c5c7fe62c0c2bc5a907d7ceeb2938713e178808`,
and `ba0c1788ef2c536a9dad2ce57dab274b112dc30b0997b96161c40f2b49e2e978`.
Those results are superseded diagnostics and are not the current maintenance
evidence.

## Retained limitations

This record does not decide expiry, clocks, rollback, root bootstrap or
rotation, consistent snapshots, broader recovery semantics, transport,
retained-byte admission, installation, macOS support, production durability,
TUF adoption, production authority, parent closure, PR integration, or
downstream sequencing. The interrupted-refresh observation is one bounded
process-interruption result, not exhaustive lifecycle or power-loss
certification. Broad final integrated review of the prototype remains outside
this maintenance record.
