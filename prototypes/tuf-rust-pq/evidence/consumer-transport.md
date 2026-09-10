# Consumer and transport evidence

This record describes the consumer/transport experiment using public synthetic inputs. It is not an acceptance, adoption, merge, release, or production qualification record.

## Source identity

[The checksum manifest](consumer-transport-inputs.sha256) binds 52 code, fixture, lock, and patch inputs used by the recorded checks. The manifest file's SHA-256 digest is:

```text
fae03ef90d47a704a3f2dec31ea3b9d28fa309d8bb946df8c2a882be417f19f6
```

Manifest paths are relative to `prototypes/tuf-rust-pq`. Verify it from that directory with:

```sh
sha256sum -c evidence/consumer-transport-inputs.sha256
```

The accepted predecessor is commit [`fddd7f564737ed5d213061a0c8621141bd85a2e9`](https://github.com/nisavid/codiquary/commit/fddd7f564737ed5d213061a0c8621141bd85a2e9), retained in unmerged [draft pull request #19](https://github.com/nisavid/codiquary/pull/19). Its receipts are historical provenance and are not consumer/transport execution evidence. The same value appears as the makepkg fixture selection's version and records fixture-version provenance; it does not identify the executable consumer/transport source set. The executable inputs are bound separately by the checksum manifest, and this frozen scan identifies the unchanged pre-fix source as `43ee6886748349af5789d858b8fc949735f31b165733fb912b8ef4592f55495e`.

## Dependency provenance

The predecessor inspection recorded:

- 189 selected packages: 184 registry packages and five path packages;
- 181 reachable packages;
- 30 custom-build packages; and
- 16 proc-macro source trees.

These values describe the predecessor inventory. They have not been asserted as counts for the consumer/transport source set. In particular, `url` is now a direct dependency but was already transitive in the predecessor; locked, offline Cargo metadata must be inspected to determine the actual package, feature, and edge delta.

Cargo metadata resolves manifests and graph relationships without compiling or running dependency build scripts, proc macros, or tests. Metadata comparison is therefore an execution gate, not evidence that dependency code ran successfully.

## Recorded environment

The successful run used `x86_64-unknown-linux-gnu` with:

| Tool | Version |
| --- | --- |
| Rust, Cargo, and rustdoc | 1.98.1 |
| pacman | 7.1.0 |
| libalpm | 16.0.1 |
| GnuPG | 2.4.9 |
| GNU tar | 1.35 |
| zstd | 1.5.7 |
| libarchive | 3.8.9 |
| curl | 8.21.0 |

The prototype package uses Rust edition 2021. The repository's root scaffold uses Rust edition 2024. macOS was not executed.

The isolated execution exposed `/usr`, selected non-secret `/etc` files, a resolved Rust toolchain, read-only source, read-only inspected path dependencies, and the inspected locked Cargo cache. Bubblewrap used `--unshare-all`, and Cargo execution was offline and serialized with an exclusive external `flock`.

The recorded test and Clippy receipts used fresh task-only temporary and home directories with a reused task-only target cache. Before each of those commands, the owning package was removed with:

```sh
cargo clean --package tuf-rust-pq-prototype
```

Formatting used the same exclusive lock and reused target without a clean. This recorded envelope differs from the replay guide's initially fresh target, which may be reused across one frozen test, Clippy, and formatting sequence.

## Command results

The literal successful Cargo checks, run from `prototypes/tuf-rust-pq`, were:

```sh
cargo test --locked --offline
```

Result: 37 passed, zero failed, and zero ignored.

```sh
cargo clippy --locked --offline --all-targets -- -D warnings
```

Result: exit 0 with warnings denied.

```sh
cargo fmt --check
```

Result: exit 0 with no output. The tested formatting scope did not use `--all`, because that option traversed dependency workspaces and exposed unrelated formatting failures in the recorded environment.

Repository-policy checks and `git diff --check` also passed. Their scope is source shape and repository policy only.

## Fixture and API scope

### TUF profile and target handling

The experimental TUF profile uses pinned, patched Tough and pinned Sequoia with RFC 9980 ML-DSA-65 and Ed25519 composite OpenPGP signatures. The target publisher creates synthetic top-level and delegated metadata and one consistent-snapshot target.

The makepkg composition independently binds product, version, channel, purpose, policy profile, and target path before acquisition and tests each field's substitution. The pacman composition supplies the same six-field shape to acquisition, but its request and expected selection are derived from the same value; it does not independently test pacman selection substitution. Tough verifies the metadata description and collects the target. The fake policy gate receives the held bytes, and the fake consumer receives them only after the selected native check succeeds.

The fake gate proves call order and its observed input. It is not a production policy implementation. The fake consumer proves a call and byte digest; it does not install or execute content.

### PKGBUILD fixture

The makepkg fixture uses `artifact.bin`, a conventional synthetic detached signature, a synthetic public certificate, `PKGBUILD`, and `makepkg.conf`. After the policy gate, the held target is written into a read-only harness before `makepkg --verifysource` runs. The expected successful observation includes both `Verifying source file signatures with gpg...` and `artifact.bin ... Passed`.

A successful makepkg composition attempt records one native attempt immediately before invoking `makepkg`, followed by one fake-consumer call after the expected signature and checksum observations are accepted.

### Pacman fixture

The pacman fixture contains a 124-byte synthetic package, a synthetic sync database, their detached signatures, and a synthetic public certificate. Only `codiquary17-native-1.0-1-any.pkg.tar.zst` is the TUF target. The database and both detached signatures remain independent native companions.

The native helper exercises `alpm_initialize`, the refusing question callback, `alpm_pkg_load`, `alpm_pkg_check_pgp_signature`, `alpm_register_syncdb`, `alpm_db_get_valid`, `alpm_db_check_pgp_signature`, and `alpm_db_get_pkg`. Successful evidence contains:

- zero question-callback calls;
- required package signature level `1`;
- required database signature level `1024`;
- one valid, fully trusted signature result for the package;
- one valid, fully trusted signature result for the database; and
- package identity `codiquary17-native`, version `1.0-1`, filename `codiquary17-native-1.0-1-any.pkg.tar.zst`.

A successful pacman composition attempt records one native attempt, then calls the fake consumer once from the accepted native-verification callback.

Libalpm returns the primary certificate fingerprint, `3A013736081935591B079572B5BF6DA65146304C`, for both signature observations. The fixture separately declares signing subkey `5A6FC73F924A7C473C2B76AD5B9B92C920B04114`; the returned field does not independently identify that subkey.

These conventional RSA-2048/SHA-256 native fixtures test interoperability and preserved controls. They do not qualify native pacman or makepkg verification for post-quantum cryptography.

### HTTP fixture

The custom `CurlFixtureTransport` accepts only plain HTTP URLs at its assigned `127.0.0.1` port, without credentials, query strings, or fragments. It disables curl configuration, proxies, redirects, and non-HTTP protocols, uses HTTP/1.1, and records process status, HTTP status, response bodies, and server-observed exchanges.

A successful repository load observes six exchanges in order: an expected `2.root.json` 404, timestamp, snapshot, top-level targets, delegated targets, and the consistent-snapshot target.

This is a curl-based loopback Tough `Transport`. It is not Tough's feature-gated `HttpTransport`, TLS, remote transport, or production transport qualification.

## Observed paths and failure stages

| Case | Observed stopping point | Native attempts | Fake-consumer calls |
| --- | --- | ---: | ---: |
| Positive file or HTTP makepkg path | After TUF, one accepted gate call, and successful `makepkg` observations | 1 | 1 |
| Positive file or HTTP pacman path | After TUF, one accepted gate call, and accepted package/database evidence | 1 | 1 |
| Protected-selection substitution on the file-backed makepkg path | Before repository acquisition or native setup | 0 | 0 |
| HTTP target equivocation | During TUF target collection, before the gate | 0 | 0 |
| Retained-policy rejection | After held-byte collection, before native setup | 0 | 0 |
| Held-byte mutation | After acquisition and an accepted gate; the mutated bytes are staged read-only and fail the staged artifact digest check before `makepkg` invocation | 0 | 0 |
| Damaged makepkg signature | `makepkg` exits 1 after reporting native signature failure | 1 | 0 |
| Disabled makepkg signature check | `makepkg` exits 0, but the caller rejects the missing signature observation | 1 | 0 |
| Changed pacman package with stale signature | `alpm_pkg_load` rejects the package signature | 1 | 0 |
| Changed pacman database with stale signature | `alpm_db_get_valid` rejects the database after valid package handling | 1 | 0 |
| Disabled pacman package or database signature check | The caller rejects otherwise parseable evidence whose mandatory configured signature level is zero | 1 | 0 |

The mutation test records the original fully collected bytes and digest, mutates the held vector, passes the mutated bytes to the accepting fake gate, and writes them into the read-only harness. The following expected-digest check fails; `native_attempts` remains zero. The failure is therefore neither an acquisition rejection nor a post-`makepkg` check.

The selection-substitution case supplies representative evidence for the shared acquisition boundary through the file-backed makepkg composition. No test independently supplies an accepted pacman selection distinct from its request, and no pacman selection-substitution claim follows.

HTTP exchange-retention checks run while the fixture server is joined and evidence is persisted. In a successful attempt, an evidence-retention failure can occur after a fake-consumer call. The tests do not support a claim that every timeout or retention failure precedes the consumer.

## Exact test mappings

### Selection, held bytes, policy, transport, and native composition

- `file_backed_tuf_rejects_protected_selection_substitutions_before_acquisition_or_native_attempt`
- `file_backed_tuf_target_rejects_retained_codiquary_policy_before_native_or_fake_consumer`
- `file_backed_tuf_completed_held_bytes_mutation_stops_before_native_verification_or_fake_consumer`
- `loopback_http_tuf_rejects_target_transport_equivocation_before_gate_native_or_fake_consumer`
- `file_backed_tuf_target_reaches_makepkg_verified_fake_consumer`
- `loopback_http_tuf_target_reaches_makepkg_verified_fake_consumer`
- `file_backed_tuf_target_rejects_bad_native_signature_before_fake_consumer`
- `file_backed_tuf_target_rejects_disabled_native_signature_check_before_fake_consumer`
- `file_backed_published_pacman_target_reaches_native_verified_fake_consumer`
- `loopback_http_published_pacman_target_reaches_native_verified_fake_consumer`
- `file_backed_published_pacman_target_rejects_native_failures_and_disabled_mandatory_checks_before_fake_consumer`
- `loopback_http_retains_observed_server_exchange_bytes_for_success_and_equivocation_rejection`

These 12 tests are in `tests/file_makepkg_tracer.rs`.

### Native pacman behavior

- `native_positive_path_verifies_package_and_database_before_one_consumer_call`
- `native_verifies_held_package_when_original_fixture_path_is_unavailable`
- `native_rejects_mtime_changed_package_with_original_signature_before_consumer`
- `native_rejects_mtime_changed_database_with_original_signature_after_valid_package_before_consumer`
- `native_rejects_disabled_package_signature_check_before_consumer`
- `native_rejects_disabled_database_signature_check_after_valid_package_before_consumer`

These six tests are in `tests/pacman_native_tracer.rs`.

### Supporting TUF binding

- `publish_single_delegated_target_loads_frozen_pacman_package` maps the generated composite metadata fixture to the frozen pacman package.
- `publisher_and_client_bind_exact_target_bytes` maps the earlier fixed publisher/client fixture to its exact target bytes.

The complete run also includes the remaining composite-profile and interrupted-refresh tests. Their green results preserve their own existing claims; this record does not redefine those claims as consumer/transport evidence.

## Limits

The recorded checks did not install packages, execute candidate content, cross a privilege boundary, or exercise a production repository or trust store. They do not establish installation safety, updater behavior, lifecycle mapping, production policy, custody, publication, durable recovery, power-loss recovery, platform support, production hardening, or TUF adoption.

The six-field substitution test is limited to the file-backed makepkg composition. The shared acquisition negatives and integrated pacman failure cases are representative boundary cases, not an exhaustive consumer/transport or error cross-product.

On the recorded host, `/usr/include/sys/un.h` defined `sun_path[108]`. The failing deep GPG browser-socket pathname measured 108 bytes, requiring a 109th byte for its NUL terminator; the shallow layout succeeded. This supports a Unix-domain socket-length explanation, but the comparison does not isolate or prove the mechanism.
