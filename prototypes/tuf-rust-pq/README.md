# RFC 9980 TUF Rust prototype

This throwaway prototype signs and verifies a synthetic TUF metadata chain with
one OpenPGP v6 algorithm-30 composite signature. The tested verifier rejects
damage to either the Ed25519 or ML-DSA-65 component. Across root and
delegated-targets verification, one verified composite signing-key fingerprint
contributes at most once to a signature threshold even when authorized key
objects have different correctly derived TUF key IDs. Undefined fields on the
provisional OpenPGP key object or its `keyval` are rejected before Root or
Delegations metadata is accepted, including when the key is unused. The
all-role fixture binds serialized role bytes through snapshot and timestamp
Metafiles and traverses a delegated hashed target. The experimental profile
fixture is documented in [evidence/experimental-profile.md](evidence/experimental-profile.md).
The separate [interrupted-refresh observation](evidence/issue-22-interrupted-refresh.md)
uses fixed public metadata to record one process interruption at Tough's
existing datastore/filesystem seam.
The dedicated [publisher/client binding](evidence/issue-23-publisher-client-binding.md)
loads one public metadata fixture through Tough and binds its selected target
descriptor to the publisher's exact bytes.

> [!CAUTION]
> This is one synthetic publisher/verifier seam, not a production security
> design. Use a disposable, credential-free environment and no production key
> material, identities, trust stores, or metadata.

Independent review found `RR1-THRESHOLD-IDENTITY-001` in the historical frozen
slice: Tough counted successful verifications by distinct TUF key ID, while the
OpenPGP verifier ignored fields that changed that ID. The revised patch returns
a domain-separated threshold identity from signature verification. Existing key
types retain TUF-key-ID identity; this provisional OpenPGP profile uses the
verified v6 signing-key fingerprint. Root and delegation verifiers count each
identity once. The historical review evidence remains bound to its recorded
revision. This is not an operator, custody, underlying-component independence,
or broader cryptographic audit claim.

A second independent review found that the undefined-field check still ran
only for a key selected to verify a signature. The patch moves that profile
validation to the shared metadata key-map deserializer and retains it in direct
verification. The current maintenance replay covers that ingestion change, the
shared root and delegation threshold-verification change, and the conventional
ECDSA identity behavior retained through both public threshold loops. See the
[current Tough compatibility evidence](evidence/current-compatibility.md).
Final verification and focused maintenance review are tracked in
[Correct and replay the current Tough compatibility record](https://github.com/nisavid/codiquary/issues/24).

The replay receipts maintained in this document are executable-input
provenance from revision
`298fd26fe44a759da54d3fe8ef9fb34446c4c981` on
`x86_64-unknown-linux-gnu` with Rust, Cargo, and rustdoc 1.98.1 and external
OpenSSL 3.6.3. The Sequoia backend requires OpenSSL 3.5 or newer. macOS remains
unexecuted.

## Prerequisites

- Git and GNU `sha256sum`
- Rust, Cargo, and rustdoc 1.98.1
- OpenSSL 3.5 or newer, including headers and `pkg-config` metadata
- A C/C++ toolchain, CMake, `pkg-config`, and libclang for the inspected native
  build surface
- Bubblewrap, or an equivalent sandbox that can make the source read-only,
  expose only scratch build directories as writable, and deny network access
- Public network access for the source and locked-crate fetch only

Run the following commands from this directory.

## Reconstruct the pinned sources

The root manifest has two source-tree dependencies under `.scratch/`: Tough at
`.scratch/tough/tough` and Sequoia at `.scratch/sequoia/openpgp`. Cloning both
complete workspaces also supplies Tough's `olpc-cjson` and Sequoia's
`buffered-reader` path dependencies.

```sh
set -eu

tuf278_tough_commit=98d8eb8b2ce63515d9b4981c938ef6453c5b5771
tuf278_tough_tree=2eb4bd2fc529460a9f0e021dae88861a3632e1a9
tuf278_sequoia_commit=0b0c8c7f038b829de2da0d28a822941d8600f3ee
tuf278_sequoia_tree=85b7646f4bbf4a323beb03b4ead52b4db6f9ab64

test ! -e .scratch/tough
test ! -e .scratch/sequoia
mkdir -p .scratch

git init -q .scratch/tough
git -C .scratch/tough remote add origin https://github.com/awslabs/tough.git
git -C .scratch/tough -c core.hooksPath=/dev/null fetch \
  --depth=1 --no-tags origin "$tuf278_tough_commit"
git -C .scratch/tough -c core.hooksPath=/dev/null checkout \
  --quiet --detach FETCH_HEAD

git init -q .scratch/sequoia
git -C .scratch/sequoia remote add origin \
  https://gitlab.com/sequoia-pgp/sequoia.git
git -C .scratch/sequoia -c core.hooksPath=/dev/null fetch \
  --depth=1 --no-tags origin "$tuf278_sequoia_commit"
git -C .scratch/sequoia -c core.hooksPath=/dev/null checkout \
  --quiet --detach FETCH_HEAD

test "$(git -C .scratch/tough rev-parse HEAD)" = "$tuf278_tough_commit"
test "$(git -C .scratch/tough rev-parse 'HEAD^{tree}')" = "$tuf278_tough_tree"
test "$(git -C .scratch/sequoia rev-parse HEAD)" = "$tuf278_sequoia_commit"
test "$(git -C .scratch/sequoia rev-parse 'HEAD^{tree}')" = \
  "$tuf278_sequoia_tree"
```

Verify the prototype inputs, apply the main Tough source patch, and verify its
output:

```sh
sha256sum -c <<'CHECKSUMS'
c0aea0775da70c2cae9839e77aadfa07b0670f31e050c7500afc15253229613e  Cargo.toml
de70879239cb2694440a557af44b39f70b43a88f0ac620ee164ccb17565dd27f  Cargo.lock
68128e44f362f2f369723eb6f50237fc860cef3b524acdcd4bd36b6c7d479cca  src/lib.rs
b2ae11447744e91adcb6ba222d6bfddbf7787e1a76e8fe60f0655e8444ea810d  src/experimental_profile.rs
8540324f3cd231ca244928024b2b1eea92ec2e16433187b2e4b708696b8e50dc  patches/tough-openpgp-rfc9980.patch
8720ad3dd63c05109761b624922248b94a938205a1d43987032d93e73377100c  patches/tough-default-sequoia-source.patch
8951066c56b6f1fbbc391aedcdf6e15322f88356ff0f2d4d04b3ebf926fbe268  evidence/tough-default.Cargo.lock
f4a6691f666b403cbffce59a9588a845f4d9333911a72c2de5b9d0e61e954a2f  tests/composite_metadata.rs
24bb696de17047aae1866a4cd6f19e3b1dfa6d37e17110cbf12f1b92d9c5bb4c  evidence/issue-23-publisher-client-binding.md
eb57467989421816a5f4a0526c81b941d0c376df811afd6e9803a7a9b364a982  tests/publisher_client_binding.rs
4030e7e7f8020e39528969e60603563a4df0fbfa7c2909680449211bb370bb9c  tests/data/publisher-client-binding/trusted-root.json
3c83467c6f821630ddcb667ff21ae20d5eddd7b0cb715979a6cdd5714adcb887  tests/data/publisher-client-binding/metadata/1.delegated.json
0c756c4cfaee4be2656b2975eefbc4a40b50bbd9792cbdeeb8af45bc7d523dd7  tests/data/publisher-client-binding/metadata/1.snapshot.json
599bc3875889620c15c8543b471d1dd3a9081a4ec357658d069dadcac1a4666a  tests/data/publisher-client-binding/metadata/1.targets.json
155033b891103be8d1b874088f6a3efb3a1476c9d92b91f853b51721eb1f2735  tests/data/publisher-client-binding/metadata/timestamp.json
d13ecc865c37b23650615038c232eccfa5770c8bc345dbe98db595273fecda4a  tests/data/publisher-client-binding/targets/artifact.bin
e2b556e4e1b082a52da6d717526faa756da1fdb8eb806f2600a6f232120eef6b  evidence/issue-22-interrupted-refresh.md
433103db14c6ac5f705ef2a0d60143552e47f72ef9cfdc4b5f47bdee4c6fc805  tests/interrupted_refresh.rs
26773c6c52e1873974805b9f0e093f8203ef0eb2113a09d4c54ee9fc26799361  tests/interrupted_refresh_fixture/mod.rs
5023f44894b2a7c434138a2b8a5ae3c2216f217f74eff87ee999a11e21552e90  tests/data/interrupted-refresh/trusted-root.json
f28da9a44f7364d572ede28460a7a0fdab56f2c6a3f081e8cd0ff08f3cd7afa0  tests/data/interrupted-refresh/v1/1.delegated.json
e2756c673c0a345917b31a6e2e2cf0ac65c47cbdbae76bfdb8317c2f48dc4697  tests/data/interrupted-refresh/v1/1.snapshot.json
8e78b164313efe2699caa61154bba0ad9951186f95c6ae2462c449870ebf2ab1  tests/data/interrupted-refresh/v1/1.targets.json
54b539221e480c8c538e3383e8a49a4871030c4678481557140da29febd72a9a  tests/data/interrupted-refresh/v1/timestamp.json
cd1bd57f3c7451b1e32f50c9181e7da682b5deeddf4cd2f38e75a5daa083c43e  tests/data/interrupted-refresh/v2/2.delegated.json
16cb6c561569d06c75c4693981b2a81c7f5fe86be508dde35fab61316409bf95  tests/data/interrupted-refresh/v2/2.snapshot.json
6ed4c83f0c2600bdeff8cf977a30a2b120f2c7a5ff6e9cbf5fc7c97bde8fbe96  tests/data/interrupted-refresh/v2/2.targets.json
59e734efdb3afd0b51f49ec1fdfc9ecd85545f78e804b7d0a84b1d4d67fee312  tests/data/interrupted-refresh/v2/timestamp.json
CHECKSUMS

git -C .scratch/tough apply --check \
  ../../patches/tough-openpgp-rfc9980.patch
git -C .scratch/tough apply ../../patches/tough-openpgp-rfc9980.patch

sha256sum -c <<'CHECKSUMS'
8b4c3d4803ed2e0fa4250fd7e9069b628d537b67122f999e6ae78a35118cbb84  .scratch/tough/tough/Cargo.toml
ac1b3c4fb6242f109a08bce5d10fcf8b2bbb56248a8b57c14792fc498dbb0575  .scratch/tough/tough/src/schema/de.rs
3ac428c534fa2b7560febb58b959091015ab20f84b2ba04170c71193d8094993  .scratch/tough/tough/src/schema/error.rs
0b62446332794800c3b24660acceb0d22a9e9f603ec69ae79a59ddf143879104  .scratch/tough/tough/src/schema/key.rs
dc9d19ecf6332fa909b20e31d68463f0c64c79f54f8561bb0b8eeb0e714c6685  .scratch/tough/tough/src/schema/verify.rs
CHECKSUMS
```

## Fetch and inspect without executing dependencies

The repository's `.cargo/config.toml` sets `net.offline = true`. Only the
network-permitted fetch commands below override that setting with
`CARGO_NET_OFFLINE=false`; the repository configuration is not changed. All
subsequent metadata, graph, build, and test commands remain locked and offline.
An empty-cache fetch without this command-local override exits 101.

Create disposable Cargo, target, and temporary directories. No disposable
`HOME` directory or `HOME` setting is used.

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
  "$tuf278_cargo" fetch --locked --target x86_64-unknown-linux-gnu

env -i \
  PATH="$tuf278_toolchain_bin:/usr/bin:/bin" \
  CARGO_HOME="$tuf278_run/cargo-home" \
  CARGO_TERM_COLOR=never \
  "$tuf278_cargo" metadata --locked --offline \
  --filter-platform x86_64-unknown-linux-gnu --format-version 1 \
  >"$tuf278_run/metadata.json"
```

Before executing fetched code, use the metadata and `Cargo.lock` to complete
the same bounded inspection:

1. Account for 189 selected packages: 184 registry packages and five path
   packages. The executable graph has 181 reachable packages.
2. Verify all 184 selected registry archives against their lockfile SHA-256
   checksums.
3. Read every selected manifest. Inspect the entry point and included modules
   for all 30 custom-build packages, then inspect all 16 proc-macro source
   trees.
4. Confirm the build scripts invoke only the expected local Rust, C/C++, CMake,
   `pkg-config`, bindgen, LALRPOP, and linker tools, with generated output under
   Cargo's build directory.

The recorded review used targeted capability scans plus manual source reading;
it was not a full security audit. Its inventory digests and source findings are
in [the source/build inspection](evidence/source-build-inspection.md).

For this maintenance replay, the selected registry trees were byte-identical
to the retained inspected trees, every archive matched its active lock
checksum, and the path-package trees, locks, target, Tough patch, workspace
override, and feature boundary matched. That permitted reuse of the retained
feature-sensitive inspection. A future replay must revalidate the actual source
bytes and resolved graph; any delta must be inspected before execution.

The applicable Tough graph contains 263 reachable packages, including 258
registry packages, 39 custom-build packages, and 19 proc-macro packages. The
retained comparison identifies 145 package name/version pairs newly selected
relative to the prototype inventory. The exact graph, lock, results, and limits
are recorded in the [current Tough compatibility evidence](evidence/current-compatibility.md).

## Run the tested slice

First confirm that Bubblewrap can create its isolated namespaces:

```sh
bwrap --ro-bind / / --dev /dev --proc /proc --unshare-all \
  --die-with-parent --new-session /usr/bin/true
```

The maintenance probe and replay succeeded with `--unshare-all`; the parent and
sandbox network namespace identifiers differed. The command below gives the
source a read-only view of the host, makes only the three disposable directories
writable as host-backed paths whose writes persist outside the sandbox, mounts
private `/dev` and `/proc`, starts with an empty environment, selects Cargo,
rustc, and rustdoc by resolved toolchain path, and runs from the pre-fetched
cache without network access:

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
      --test composite_metadata -- --nocapture
```

At executable input revision
`298fd26fe44a759da54d3fe8ef9fb34446c4c981`, this command passed 15 tests,
failed 0, and ignored 0. Its raw command/output SHA-256 values are
`9fd26dad372c57b5eb132152791a1293ebab3a6188aec509bf75437ba34dae26`
and `f8b74dee0be02ae213821f980c61483779f1a0cd6b7d4790adf723c81a3f4b5c`.
Generated key IDs and fingerprints vary by run. Final-revision verification and
focused review belong in
[Correct and replay the current Tough compatibility record](https://github.com/nisavid/codiquary/issues/24).
The [Cycle 2 record](evidence/slice-8-cycle-2.md),
[threshold-identity record](evidence/slice-5-threshold-identity.md), and
[original red/green record](evidence/slice-3-red-green.md) remain historical
checkpoints.

## Replay the interrupted-refresh observation

The fixed, public v1/v2 input and exact hashes are recorded in the
[interrupted-refresh observation](evidence/issue-22-interrupted-refresh.md).
The test establishes v1, loads an uninterrupted v2 control in a separate
clone, interrupts a fixture child only after the v2 timestamp and FIFO-open
phases are proven, restores the complete candidate source, and classifies the
completed fresh-loader result as previous, candidate, or neither. An accepted
result includes the observed verified `Repository`; a non-acceptance includes
Tough's actual returned error without assigning it a cryptographic cause.
Fixture setup, observation, receipt, timeout, and process failures still fail
the test.

From the repository root, rather than this prototype directory, run:

```sh
rustup run 1.98.1 cargo test \
  --manifest-path prototypes/tuf-rust-pq/Cargo.toml \
  --locked --offline --test interrupted_refresh -- --nocapture
```

The command prints the actual `rawObservation` receipt path. By default, the
receipt is written inside the run's fresh disposable workspace and removed
with that workspace. Set `CODIQUARY_22_OUTPUT_DIR` to a fresh directory to
retain the full raw receipt, including nondeterministic
`latest_known_time.json` bytes. For example, prefix the command with
`CODIQUARY_22_OUTPUT_DIR="$(mktemp -d)"`.

## Replay the applicable Tough suite

After applying the main source patch above, verify and apply the test-workspace
inputs:

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

Fetch the locked Linux test graph without building it. This is the only Tough
replay command that overrides the repository's offline default:

```sh
env -i \
  PATH="$tuf278_toolchain_bin:/usr/bin:/bin" \
  CARGO_HOME="$tuf278_run/cargo-home" \
  CARGO_TERM_COLOR=never \
  CARGO_NET_OFFLINE=false \
  "$tuf278_toolchain_bin/cargo" fetch --locked \
    --manifest-path .scratch/tough/tough/Cargo.toml \
    --target x86_64-unknown-linux-gnu
```

Revalidate the resolved graph and retained inspection as described above, then
run the exact offline replay:

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

At executable input revision
`298fd26fe44a759da54d3fe8ef9fb34446c4c981`, the replay passed 78 tests,
failed 0, and ignored 1. Its raw command/output SHA-256 values are
`98ca301371e3230dbba5d3b4f1950bea1556b0f31adfadc8017b15386f499bcf`
and `0417a6f6306a0d22f04e728fba000fd35d6d735c8bc31b5ea9624564a2dec8b6`.
No Tough feature was enabled. The `http`, `http2`, `integ`, and `fips` feature
profiles remain outside this replay; the zero-test HTTP target did not exercise
HTTP behavior, and the installed integration-support packages did not expand
the invoked package's feature boundary.

## Scope

The current fixture covers the root publisher/verifier seam, the composite
threshold-identity boundary in root and delegation verification, all four
top-level role shapes, one delegated role with a hashed target, serialized
snapshot/timestamp references, a closed typed experimental profile matched
against those observations, and one interrupted datastore refresh with a
complete source available after restart. The maintenance replay additionally
checks the shared key-map ingestion validation and preserves conventional ECDSA
TUF-key-ID threshold identity through public root and delegation verification.
It does not decide lifecycle policy: expiry, clocks, rollback, root bootstrap
or rotation, consistent snapshots, broader recovery semantics, durable
power-loss recovery, target confinement, metadata limits, retained-byte
admission, held-byte policy composition, consumer transport, installation,
macOS qualification, production durability, TUF adoption, production
authority, operator or custody independence, parent closure, PR integration,
or downstream sequencing remain outside this work.
