# Replay the consumer and transport checks

Use this guide to repeat the Linux checks from prepared, checksum-verified, and source-inspected inputs. It is not a fresh-install guide or a new dependency-audit procedure.

The replay parses synthetic package content but does not install or execute it. Do not substitute real repositories, trust stores, providers, keys, credentials, or candidate artifacts.

## Prerequisites

Use an x86_64 Linux host with these tools already provisioned:

- Rust, Cargo, rustdoc, rustfmt, and Clippy 1.98.1;
- pacman 7.1.0 and libalpm 16.0.1;
- GnuPG 2.4.9;
- GNU tar 1.35, zstd 1.5.7, and libarchive 3.8.9;
- curl 8.21.0;
- Bubblewrap, `flock`, GNU `sha256sum`, a C compiler, and `pkg-config`; and
- OpenSSL 3.5 or newer with its headers and `pkg-config` metadata.

The recorded run used OpenSSL-compatible Sequoia support and the exact versions listed in [the evidence record](../evidence/consumer-transport.md). macOS is not covered.

Also prepare:

- a task-owned source checkout containing one frozen source candidate;
- the pinned, patched Tough and Sequoia workspaces already acquired and inspected under a separate dependency directory;
- the complete locked Cargo cache whose selected archives and source trees were inspected;
- a shared lock file owned by that cache's coordinator; and
- fresh task-only target, temporary, and home directories.

Do not install tools or dependencies as part of this replay.

## Verify the prepared inputs

The manifest paths are relative to `prototypes/tuf-rust-pq`. From the repository root, run:

```sh
(
  cd prototypes/tuf-rust-pq
  sha256sum -c evidence/consumer-transport-inputs.sha256
)
```

The manifest contains 52 code, fixture, lock, and patch inputs. Verify the manifest file itself from the repository root:

```sh
printf '%s  %s\n' \
  fae03ef90d47a704a3f2dec31ea3b9d28fa309d8bb946df8c2a882be417f19f6 \
  prototypes/tuf-rust-pq/evidence/consumer-transport-inputs.sha256 \
  | sha256sum -c -
```

This manifest supersedes predecessor checksum blocks for the changed inputs. The predecessor [source and build inspection](../evidence/source-build-inspection.md) remains preparation provenance, but its old checksum commands are not the integrity check for this source set.

Source and dependency acquisition is a separate, network-permitted phase that must finish before execution. During that phase, confirm the pinned Tough and Sequoia commits and trees, Cargo archives, manifests, build scripts, proc-macro sources, and patch outputs against the retained inspection. Do not enable networking in the commands below.

## Inspect the resolved graph

The predecessor inspection recorded 189 selected packages: 184 registry packages and five path packages. It also recorded 181 reachable packages, 30 custom-build packages, and 16 proc-macro source trees. These are predecessor inventory and provenance values, not verified counts for this source set.

The source set adds `url` as a direct dependency; it was transitive in the predecessor. Compare locked, offline metadata with the predecessor inspection and adjudicate the actual delta before executing tests, Clippy, build scripts, proc macros, or test binaries. Do not assume the package counts remain equal merely because the package was already present transitively.

Generating Cargo metadata resolves manifests, packages, features, and target relationships. It does not compile the graph or run dependency build scripts, proc macros, or tests, so successful metadata resolution is not execution evidence.

## Define task-specific paths

Use Bash and set each variable to an absolute path:

```bash
export CQ_SOURCE=/absolute/path/to/task-source
export CQ_DEPS=/absolute/path/to/inspected-dot-scratch
export CQ_TOOLCHAIN=/absolute/path/to/rust-1.98.1-toolchain
export CQ_CARGO_CACHE=/absolute/path/to/inspected-locked-cargo-cache
export CQ_CACHE_LOCK=/absolute/path/to/cache-coordinator.lock
export CQ_RUN="$(mktemp -d "${TMPDIR:-/tmp}/cq-replay.XXXXXXXX")"

mkdir -p "$CQ_RUN/target" "$CQ_RUN/tmp" "$CQ_RUN/home"

test -d "$CQ_SOURCE/prototypes/tuf-rust-pq"
test -d "$CQ_DEPS/tough/tough"
test -d "$CQ_DEPS/sequoia/openpgp"
test -x "$CQ_TOOLCHAIN/bin/cargo"
test -d "$CQ_CARGO_CACHE"
test -f "$CQ_CACHE_LOCK"
```

`CQ_RUN` and its target, temporary, and home directories must be fresh and task-owned at the start of this frozen replay. The target may then be reused by the test, Clippy, and formatting commands below; it is not newly empty before each command.

Each different source candidate must use a new target directory. If the coordinator deliberately reuses a task-only target instead, it must serialize access with the same exclusive lock and run `/toolchain/bin/cargo clean --package tuf-rust-pq-prototype` before executing that candidate. Never share an unlocked target or cache between candidates.

The shared Cargo cache is writable only because the tested envelope required it. Hold its coordinator-provided exclusive lock for every Cargo command.

The sandbox maps task scratch to `/tmp` and its task home to `/home/fixture`. Keep these namespace paths short. On the recorded host, `/usr/include/sys/un.h` defined `sun_path[108]`; the failing deep GPG browser-socket path measured 108 bytes before its 109th NUL byte, while a shallow layout succeeded. Those observations support a Unix-domain socket-length explanation but do not isolate or prove the mechanism.

## Build the isolation envelope

The envelope exposes `/usr`, the resolved Rust toolchain, the task source, inspected dependencies, and selected non-secret `/etc` files. It does not mount the host root, pass the invoking environment, expose credentials, or provide network access.

```bash
cq_bwrap_args=(
  --unshare-all
  --ro-bind /usr /usr
  --symlink usr/bin /bin
  --symlink usr/bin /sbin
  --symlink usr/lib /lib
  --symlink usr/lib /lib64
  --dir /etc
  --dir /etc/ssl
  --proc /proc
  --dev /dev
  --ro-bind "$CQ_SOURCE" /source
  --ro-bind "$CQ_DEPS" /source/prototypes/tuf-rust-pq/.scratch
  --ro-bind "$CQ_TOOLCHAIN" /toolchain
  --bind "$CQ_CARGO_CACHE" /cargo
  --bind "$CQ_RUN/target" /target
  --bind "$CQ_RUN/tmp" /tmp
  --bind "$CQ_RUN/home" /home/fixture
  --chdir /source/prototypes/tuf-rust-pq
  --die-with-parent
  --new-session
  --ro-bind /etc/ld.so.cache /etc/ld.so.cache
  --ro-bind /etc/localtime /etc/localtime
  --ro-bind /etc/ssl/openssl.cnf /etc/ssl/openssl.cnf
)

run_isolated() {
  flock "$CQ_CACHE_LOCK" \
    bwrap "${cq_bwrap_args[@]}" \
    /usr/bin/env -i \
      PATH=/toolchain/bin:/usr/bin:/bin \
      RUSTC=/toolchain/bin/rustc \
      RUSTDOC=/toolchain/bin/rustdoc \
      CARGO_HOME=/cargo \
      CARGO_TARGET_DIR=/target \
      CARGO_TERM_COLOR=never \
      HOME=/home/fixture \
      TMPDIR=/tmp \
      GIT_CONFIG_NOSYSTEM=1 \
      GIT_CONFIG_GLOBAL=/dev/null \
      "$@"
}
```

If Bubblewrap cannot create this namespace, stop. Do not fall through to unsandboxed execution or broaden the mounts.

## Confirm the resolved graph

```bash
run_isolated /toolchain/bin/cargo metadata \
  --locked \
  --offline \
  --filter-platform x86_64-unknown-linux-gnu \
  --format-version 1 \
  >"$CQ_RUN/metadata.json"
```

Compare `metadata.json`, `Cargo.lock`, registry archive checksums, and path-package trees with the predecessor inspection. Review the direct-dependency and feature delta, then approve or stop before executing dependency code. Do not change pins, regenerate fixtures, fetch sources, or execute an unapproved graph.

## Run the checks

Keep the source candidate frozen and use the same exclusive lock and initially fresh target for all three commands:

```bash
run_isolated /toolchain/bin/cargo test --locked --offline

run_isolated /toolchain/bin/cargo clippy \
  --locked \
  --offline \
  --all-targets \
  -- \
  -D warnings

run_isolated /toolchain/bin/cargo fmt --check
```

Do not add `--all` to `cargo fmt`; it traverses dependency workspaces outside the tested package scope in this layout.

## Confirm completion

The recorded run completed with:

- 37 passed tests, zero failed, and zero ignored;
- 16 `composite_metadata` tests;
- 12 `file_makepkg_tracer` tests;
- two `interrupted_refresh` tests;
- six `pacman_native_tracer` tests;
- one `publisher_client_binding` test;
- zero unit or documentation tests;
- strict Clippy success; and
- formatting success with no output.

Positive makepkg and pacman composition cases report one retained-policy call, one native attempt, and one fake-consumer call. Each negative case must match its documented stopping stage and consumer-call count. Inspect retained native and HTTP evidence below `$CQ_RUN/tmp` before ending the replay.

Any checksum, graph, tool-version, sandbox, GPG, native-signature, evidence-retention, test, Clippy, or formatting failure is a failed replay. Stop and investigate within the documented boundary; do not weaken checks or retry outside the sandbox.
