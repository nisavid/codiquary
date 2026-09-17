# Prepare and replay the disposable Tough exact-target proof

This is the single procedure for preparing and replaying the public,
synthetic Tough `exact-target-v1` proof. The proof workstream produces and
maintains it. Its immediate consumers are the final Linux x86_64 and
GitHub-hosted macOS arm64 evidence runs. A later genesis workstream may consume
only a reviewed proof revision and evidence bound to that revision.

This procedure does not adopt a production dependency, qualify a platform,
authorize a release, or grant installation or execution authority. It is
currently incomplete: stop after the dependency checkpoint until the executor
source/input bundle, built OCI identity, admission observations, and all
evidence they affect have a clean renewed review. The final replay commands are
recorded now so later evidence cannot silently choose a different method, but
they are not authorized to run against this foundation revision.

## Public inputs and prerequisites

Prepare one public-source directory with this relative layout:

```text
github-awslabs-tough/98d8eb8b2ce63515d9b4981c938ef6453c5b5771.tar.gz
crate-sequoia-openpgp/sequoia-openpgp-2.4.1.crate
```

`proofs/exact-target-tough/source-archives.sha256` binds both archives. The
Tough archive comes from
<https://codeload.github.com/awslabs/tough/tar.gz/98d8eb8b2ce63515d9b4981c938ef6453c5b5771>.
The Sequoia archive comes from
<https://crates.io/api/v1/crates/sequoia-openpgp/2.4.1/download>.
Acquisition is a separate coordinator-owned step. It may use the network;
source preparation and final replay may not.

Use Rust and Cargo 1.98.1, Python 3.11 or newer, `bash`, `git`, `patch`, `tar`, `sha256sum`,
and `sha512sum`. Execution also requires declared C and C++ compilers, linker,
archiver, indexer, CMake and its generator, make, Perl, libclang, and the
platform SDK. Use a fresh, task-owned Cargo home for each acquisition or
execution phase. Do not expose credential-bearing environment values in
evidence.

## Dependency checkpoint

Only the coordinator runs this stage. It resolves and fetches public Cargo
inputs, but it must not build or run build scripts, proc macros, libraries,
fixture authors, or tests.

From the repository root, set fresh task-owned input, Cargo-cache, and evidence
directories, then run:

```sh
set -euo pipefail
CQ_REPO=$(git rev-parse --show-toplevel)
CQ_PROOF="$CQ_REPO/proofs/exact-target-tough"
CQ_INPUTS=${CQ_INPUTS:?set CQ_INPUTS to the public-source directory}
CQ_CARGO_HOME=${CQ_CARGO_HOME:?set CQ_CARGO_HOME to a fresh task-owned Cargo home}
CQ_EVIDENCE=${CQ_EVIDENCE:?set CQ_EVIDENCE to a fresh task-owned output directory}
export CARGO_HOME=$CQ_CARGO_HOME

test "$(git rev-parse HEAD)" = "$(git rev-parse --verify HEAD)"
(cd "$CQ_INPUTS" && sha256sum -c "$CQ_PROOF/source-archives.sha256")
"$CQ_PROOF/scripts/prepare-sources.sh" \
  --dependency-checkpoint "$CQ_INPUTS" "$CQ_PROOF/.work/tough"

case "$(rustc --version)" in
  "rustc 1.98.1 "*) ;;
  *) echo "Rust 1.98.1 is required" >&2; exit 1 ;;
esac
case "$(cargo --version)" in
  "cargo 1.98.1 "*) ;;
  *) echo "Cargo 1.98.1 is required" >&2; exit 1 ;;
esac
rustc -Vv >"$CQ_EVIDENCE/rustc.txt"
cargo -Vv >"$CQ_EVIDENCE/cargo.txt"
sha256sum "$CQ_PROOF/.work/tough/.codiquary-prepared-tree" \
  >"$CQ_EVIDENCE/dependency-tree.sha256"

mkdir -p "$CQ_PROOF/locks"
cargo generate-lockfile \
  --manifest-path "$CQ_PROOF/Cargo.toml" \
  --config net.offline=false
cargo generate-lockfile \
  --manifest-path "$CQ_PROOF/.work/tough/Cargo.toml" \
  --config net.offline=false
cp "$CQ_PROOF/.work/tough/Cargo.lock" \
  "$CQ_PROOF/locks/tough-workspace.Cargo.lock"

cargo fetch --manifest-path "$CQ_PROOF/Cargo.toml" --locked \
  --target x86_64-unknown-linux-gnu --config net.offline=false
cargo fetch --manifest-path "$CQ_PROOF/Cargo.toml" --locked \
  --target aarch64-apple-darwin --config net.offline=false
cargo fetch --manifest-path "$CQ_PROOF/.work/tough/Cargo.toml" --locked \
  --target x86_64-unknown-linux-gnu --config net.offline=false
cargo fetch --manifest-path "$CQ_PROOF/.work/tough/Cargo.toml" --locked \
  --target aarch64-apple-darwin --config net.offline=false

for target in x86_64-unknown-linux-gnu aarch64-apple-darwin; do
  cargo metadata --manifest-path "$CQ_PROOF/Cargo.toml" \
    --locked --offline --format-version 1 --filter-platform "$target" \
    >"$CQ_EVIDENCE/linux-generated-proof-metadata-$target.json"
  cargo metadata --manifest-path "$CQ_PROOF/.work/tough/Cargo.toml" \
    --locked --offline --format-version 1 --filter-platform "$target" \
    >"$CQ_EVIDENCE/linux-generated-tough-workspace-metadata-$target.json"
  cargo tree --manifest-path "$CQ_PROOF/Cargo.toml" \
    --locked --offline --target "$target" -e features \
    >"$CQ_EVIDENCE/linux-generated-proof-features-$target.txt"
  cargo tree --manifest-path "$CQ_PROOF/.work/tough/Cargo.toml" \
    --locked --offline --target "$target" -e features \
    >"$CQ_EVIDENCE/linux-generated-tough-workspace-features-$target.txt"
  cargo tree --manifest-path "$CQ_PROOF/.work/tough/Cargo.toml" \
    --locked --offline -p tough --target "$target" -e features \
    >"$CQ_EVIDENCE/linux-generated-tough-package-features-$target.txt"
  cargo tree --manifest-path "$CQ_PROOF/.work/tough/Cargo.toml" \
    --locked --offline -p tough --target "$target" \
    -e normal,build,dev --no-dedupe --prefix depth \
    >"$CQ_EVIDENCE/linux-generated-tough-package-reachability-$target.txt"
  cargo tree --manifest-path "$CQ_PROOF/.work/tough/Cargo.toml" \
    --locked --offline -p tough --target "$target" \
    -e normal,build,dev --no-dedupe --prefix none --format '{p}' \
    | sed '/^\[/d' | LC_ALL=C sort -u \
    >"$CQ_EVIDENCE/linux-generated-tough-package-coordinates-$target.txt"
done

sha256sum "$CQ_PROOF/Cargo.lock" \
  "$CQ_PROOF/locks/tough-workspace.Cargo.lock" \
  >"$CQ_EVIDENCE/locks.sha256"
```

Resolve the locks only once. After the coordinator freezes those exact files,
confirm the actual hosted macOS arm64 graph without regenerating either lock.
On the hosted macOS checkout, set a fresh public-source directory, Cargo home,
and evidence directory, then run:

```sh
set -euo pipefail
CQ_REPO=$(git rev-parse --show-toplevel)
CQ_PROOF="$CQ_REPO/proofs/exact-target-tough"
CQ_INPUTS=${CQ_INPUTS:?set CQ_INPUTS to the public-source directory}
CQ_CARGO_HOME=${CQ_CARGO_HOME:?set CQ_CARGO_HOME to a fresh task-owned Cargo home}
CQ_EVIDENCE=${CQ_EVIDENCE:?set CQ_EVIDENCE to a fresh task-owned output directory}
CQ_REVISION=${CQ_REVISION:?set CQ_REVISION to the frozen dependency-checkpoint revision}
export CARGO_HOME=$CQ_CARGO_HOME

test "$(git rev-parse HEAD)" = "$CQ_REVISION"
test -z "$(git status --porcelain=v1)"
(cd "$CQ_INPUTS" && sha256sum -c "$CQ_PROOF/source-archives.sha256")
"$CQ_PROOF/scripts/prepare-sources.sh" \
  --dependency-checkpoint "$CQ_INPUTS" "$CQ_PROOF/.work/tough"
cp "$CQ_PROOF/locks/tough-workspace.Cargo.lock" \
  "$CQ_PROOF/.work/tough/Cargo.lock"

case "$(rustc --version)" in
  "rustc 1.98.1 "*) ;;
  *) echo "Rust 1.98.1 is required" >&2; exit 1 ;;
esac
case "$(cargo --version)" in
  "cargo 1.98.1 "*) ;;
  *) echo "Cargo 1.98.1 is required" >&2; exit 1 ;;
esac
rustc -Vv >"$CQ_EVIDENCE/rustc.txt"
cargo -Vv >"$CQ_EVIDENCE/cargo.txt"

cargo fetch --manifest-path "$CQ_PROOF/Cargo.toml" --locked \
  --target aarch64-apple-darwin --config net.offline=false
cargo fetch --manifest-path "$CQ_PROOF/.work/tough/Cargo.toml" --locked \
  --target aarch64-apple-darwin --config net.offline=false
cargo metadata --manifest-path "$CQ_PROOF/Cargo.toml" \
  --locked --offline --format-version 1 \
  --filter-platform aarch64-apple-darwin \
  >"$CQ_EVIDENCE/hosted-macos-proof-metadata-aarch64-apple-darwin.json"
cargo metadata --manifest-path "$CQ_PROOF/.work/tough/Cargo.toml" \
  --locked --offline --format-version 1 \
  --filter-platform aarch64-apple-darwin \
  >"$CQ_EVIDENCE/hosted-macos-tough-workspace-metadata-aarch64-apple-darwin.json"
cargo tree --manifest-path "$CQ_PROOF/Cargo.toml" \
  --locked --offline --target aarch64-apple-darwin -e features \
  >"$CQ_EVIDENCE/hosted-macos-proof-features-aarch64-apple-darwin.txt"
cargo tree --manifest-path "$CQ_PROOF/.work/tough/Cargo.toml" \
  --locked --offline --target aarch64-apple-darwin -e features \
  >"$CQ_EVIDENCE/hosted-macos-tough-workspace-features-aarch64-apple-darwin.txt"
cargo tree --manifest-path "$CQ_PROOF/.work/tough/Cargo.toml" \
  --locked --offline -p tough --target aarch64-apple-darwin -e features \
  >"$CQ_EVIDENCE/hosted-macos-tough-package-features-aarch64-apple-darwin.txt"
cargo tree --manifest-path "$CQ_PROOF/.work/tough/Cargo.toml" \
  --locked --offline -p tough --target aarch64-apple-darwin \
  -e normal,build,dev --no-dedupe --prefix depth \
  >"$CQ_EVIDENCE/hosted-macos-tough-package-reachability-aarch64-apple-darwin.txt"
cargo tree --manifest-path "$CQ_PROOF/.work/tough/Cargo.toml" \
  --locked --offline -p tough --target aarch64-apple-darwin \
  -e normal,build,dev --no-dedupe --prefix none --format '{p}' \
  | sed '/^\[/d' | LC_ALL=C sort -u \
  >"$CQ_EVIDENCE/hosted-macos-tough-package-coordinates-aarch64-apple-darwin.txt"
sha256sum "$CQ_PROOF/Cargo.lock" \
  "$CQ_PROOF/locks/tough-workspace.Cargo.lock" \
  "$CQ_PROOF/.work/tough/.codiquary-prepared-tree" \
  >"$CQ_EVIDENCE/source-and-locks.sha256"
```

The Linux-generated macOS-target receipts are cross-target resolution evidence,
not a hosted-macOS observation. The hosted block consumes the frozen locks
without regenerating them and records the actual host separately.

For every `tough` package coordinate receipt above, derive its reachable
build-script and proc-macro inventory from the matching target-filtered metadata.
Set the three paths to one matching observer and target, then run exactly:

```sh
CQ_METADATA=${CQ_METADATA:?set CQ_METADATA to the matching Tough workspace metadata}
CQ_COORDINATES=${CQ_COORDINATES:?set CQ_COORDINATES to the matching Tough package coordinates}
CQ_INVENTORY=${CQ_INVENTORY:?set CQ_INVENTORY to a new JSON output path}
python3 - "$CQ_METADATA" "$CQ_COORDINATES" "$CQ_INVENTORY" <<'PY'
import json
from pathlib import Path
import re
import sys

metadata_path = Path(sys.argv[1])
coordinates_path = Path(sys.argv[2])
output_path = Path(sys.argv[3])

metadata = json.loads(metadata_path.read_text(encoding="utf-8"))
coordinates = [
    line.strip()
    for line in coordinates_path.read_text(encoding="utf-8").splitlines()
    if line.strip()
]

packages = []
build_scripts = []
proc_macros = []
for coordinate in coordinates:
    match = re.match(r"^([A-Za-z0-9_.+-]+) v([^ ]+)", coordinate)
    if match is None:
        raise SystemExit(f"unrecognized cargo tree coordinate: {coordinate}")
    name, version = match.groups()
    candidates = [
        package
        for package in metadata["packages"]
        if package["name"] == name and package["version"] == version
    ]
    if len(candidates) != 1:
        raise SystemExit(
            f"coordinate is not unique in metadata: {coordinate} ({len(candidates)} matches)"
        )
    package = candidates[0]
    identity = {
        "checksum": package.get("checksum"),
        "license": package.get("license"),
        "name": name,
        "source": package.get("source"),
        "version": version,
    }
    packages.append(identity)
    for target in package["targets"]:
        target_record = {
            "crate_types": target["crate_types"],
            "kind": target["kind"],
            "package": identity,
            "target": target["name"],
        }
        if "custom-build" in target["kind"]:
            build_scripts.append(target_record)
        if "proc-macro" in target["kind"]:
            proc_macros.append(target_record)

payload = {
    "build_scripts": sorted(
        build_scripts,
        key=lambda item: (item["package"]["name"], item["package"]["version"], item["target"]),
    ),
    "package_coordinates": sorted(
        packages,
        key=lambda item: (item["name"], item["version"], item["source"] or ""),
    ),
    "proc_macros": sorted(
        proc_macros,
        key=lambda item: (item["package"]["name"], item["package"]["version"], item["target"]),
    ),
}
output_path.write_text(
    json.dumps(payload, indent=2, sort_keys=True) + "\n",
    encoding="utf-8",
)
PY
```

The feature receipt is the exact `-p tough` feature graph. The reachability
receipt retains normal, build, and development edges without deduplication. The
coordinate and derived JSON receipts identify every reachable package and every
reachable Cargo build-script or proc-macro target. Workspace metadata and
workspace-wide feature trees remain separate resolution evidence and cannot
substitute for those selected-package receipts.

After every graph and derived inventory for one observer is complete, bind all
of its files with:

```sh
(
  cd "$CQ_EVIDENCE"
  find . -maxdepth 1 -type f ! -name dependency-evidence.sha256 -print \
    | LC_ALL=C sort \
    | while IFS= read -r path; do sha256sum "$path"; done \
    >dependency-evidence.sha256
)
```

Before any compilation or execution, review and freeze all of those outputs.
Require exactly Tough 0.24.0 from the prepared local path, Sequoia OpenPGP
2.4.1 with `crypto-openssl` and no other Sequoia crypto backend, one
`openssl-sys` path to one exact `openssl-src` version, no unexpected source or
platform-specific graph change, and a complete build-script/proc-macro and
license inventory. A missing, yanked, unavailable, unexpected, or unreviewed
input stops the work; do not substitute it.

The local OpenSSL executable and `pkg-config` result do not satisfy this gate.
The vendored Cargo graph must select OpenSSL 3.5 or newer, and later runtime
evidence must report and enforce that version.

## Linux executor source and admission

Linux execution uses a proof-only OCI image derived from
`docker.io/library/rust@sha256:cdb2da72943ec036bf0c731ef5ac9e5fb2b1d17d3a9256a581bad64cf6fc093d`,
the `linux/amd64` platform manifest observed for Rust `1.98.1-bookworm`.
The upstream image metadata proves the platform, Rust version, and its
`buildpack-deps:bookworm` ancestry. It does not prove that CMake, one usable
libclang, Python 3.11 or newer, route inspection, rustfmt, or Clippy is present,
so the upstream image is not itself an admitted executor.

The derivative is declared only by
`proofs/exact-target-tough/executor/Containerfile` and
`proofs/exact-target-tough/executor/inputs.sha256`. The input manifest must bind
the upstream platform manifest, config, fixed Dockerfile, every added Debian
package archive, and the Rust 1.98.1 rustfmt and Clippy component archives. The
recipe must install only those held bytes, expose one exact libclang file at
`/opt/codiquary/lib/libclang.so`, and perform no network access. A build from an
incomplete manifest is invalid.

The frozen Debian acquisition used `apt-get --download-only --assume-yes`
because its stdin was closed. Its Rust checksum inventory excluded
`inputs.sha256` itself and then verified the completed manifest. The build
rechecks those corrected nested manifests before installing anything. It uses
`--assume-yes --no-download` with all 19 local Debian archives and runs the
standard component installers from the two held Rust archives with
`--disable-ldconfig`.

Every native OCI action crosses one controller interface. It verifies the
reviewed Podman, crun, conmon, and pasta bytes immediately before each action,
selects `/usr/bin/crun` and `/usr/bin/conmon` through Podman's shared global
option array, records their versions and the exact argv, and writes the result
to a host-only receipt directory. Podman's [`--conmon` global
option](https://docs.podman.io/en/v6.1.0/markdown/podman.1.html)
selects the conmon binary instead of the configured default. Each fresh
controller state contains only an explicit empty
`containers.conf` and an explicit empty `mounts.conf` under its read-only
configuration root. The controller requires rootless Podman, selects both files
through its scrubbed environment, and verifies their empty-file digest before
and after every action. The user `mounts.conf` therefore overrides ambient
system mount defaults, while `CONTAINERS_CONF` prevents ambient container
defaults from contributing another mount. Podman's version receipt uses the
same scrubbed environment, task-owned state, storage configuration, runtime
selection, and configuration files as the action it precedes. The controller
never changes the caller's shell options. That directory is local review
evidence: it can contain host paths and must never be mounted into a container
or published. Public evidence uses the container-only mount projection emitted
by the admission preflight below. A controller action is complete only when
`receipt.sha256` has exactly the five declared entries, all five checksums pass,
and a private same-directory temporary manifest has been atomically renamed to
that final path. A partial receipt directory or temporary manifest grants no
completed controller result.

Define this controller once in the coordinator shell before construction,
loading, inspection, or execution:

```sh
CQ_PODMAN_SHA256=1af2379af2bf863b365a12b77a2dc420eb3babff3846c24391ed207bf6073c5f
CQ_CRUN_SHA256=aca05aa473e1d81c35efeffbf14b4eeaeab07fb8571f06fa559dd770b5d7d339
CQ_CONMON_SHA256=bb6dcb31b2a7c055a6fc5b5c4ddfd325ac12036e13e004dcf63eb812a68016d6
CQ_PASTA_SHA256=e268bd80b093b3582d407996c665b39acdd8938a88ab2f476c2878ad4def9360
CQ_EMPTY_CONFIG_SHA256=e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855
readonly CQ_PODMAN_SHA256 CQ_CRUN_SHA256 CQ_CONMON_SHA256 CQ_PASTA_SHA256
readonly CQ_EMPTY_CONFIG_SHA256

cq_prepare_podman_state() {
  test "$#" -eq 1 || return 1
  local state=$1
  [[ "$state" = /* && "$state" != *$'\n'* ]] || return 1
  test ! -e "$state" || return 1
  mkdir -p "$state/home" "$state/runtime" "$state/root" "$state/runroot" \
    "$state/config/containers" || return 1
  : > "$state/config/containers/containers.conf" || return 1
  : > "$state/config/containers/mounts.conf" || return 1
  chmod 700 "$state" "$state/home" "$state/runtime" \
    "$state/root" "$state/runroot" || return 1
  chmod 500 "$state/config" "$state/config/containers" || return 1
  chmod 400 "$state/config/containers/containers.conf" \
    "$state/config/containers/mounts.conf" || return 1
}

cq_validate_sha256_manifest() {
  test "$#" -ge 3 || return 1
  local root=$1
  local manifest=$2
  shift 2 || return 1
  [[ "$root" = /* && -d "$root" && "$root" != *$'\n'* ]] || return 1
  [[ "$manifest" != /* && "$manifest" != *$'\n'* ]] || return 1
  case "/$manifest/" in
    */../*) return 1 ;;
  esac
  test -f "$root/$manifest" && test ! -L "$root/$manifest" || return 1

  local -a expected=("$@")
  local -a recorded=()
  local path line
  for path in "${expected[@]}"; do
    [[ "$path" != /* && "$path" != *$'\n'* ]] || return 1
    case "/$path/" in
      */../*) return 1 ;;
    esac
  done
  while IFS= read -r line; do
    [[ "$line" =~ ^([0-9a-f]{64})[[:space:]][[:space:]]([^[:space:]].*)$ ]] \
      || return 1
    recorded+=("${BASH_REMATCH[2]}")
  done < "$root/$manifest"
  test "${#recorded[@]}" -eq "${#expected[@]}" || return 1
  local index
  for ((index = 0; index < ${#expected[@]}; index++)); do
    test "${recorded[index]}" = "${expected[index]}" || return 1
  done
  (cd "$root" && /usr/bin/sha256sum --check --strict "$manifest" >/dev/null)
}

cq_controller_receipt_status() {
  test "$#" -eq 1 || return 1
  local receipt_dir=$1
  cq_validate_sha256_manifest "$receipt_dir" receipt.sha256 \
    controller.sha256 versions.txt version-argv.txt argv.txt status.txt \
    || return 1
  local -a status_lines=()
  mapfile -t status_lines < "$receipt_dir/status.txt" || return 1
  test "${#status_lines[@]}" -eq 1 || return 1
  local status=${status_lines[0]}
  [[ "$status" =~ ^(0|[1-9][0-9]{0,2})$ ]] || return 1
  test "$((10#$status))" -le 255 || return 1
  printf '%s\n' "$status"
}

cq_oci_controller() {
  test "$#" -ge 4 || return 125
  local state=$1
  local receipt_root=$2
  local receipt_name=$3
  shift 3 || return 125
  [[ "$state" = /* && -d "$state" ]] || return 125
  [[ "$receipt_root" = /* && -d "$receipt_root" ]] || return 125
  [[ "$receipt_name" =~ ^[a-z0-9][a-z0-9-]*$ ]] || return 125
  for directory in home runtime root runroot config config/containers; do
    test -d "$state/$directory" && test ! -L "$state/$directory" || return 125
  done
  local containers_conf="$state/config/containers/containers.conf"
  local mounts_conf="$state/config/containers/mounts.conf"
  test -f "$containers_conf" && test ! -L "$containers_conf" || return 125
  test -f "$mounts_conf" && test ! -L "$mounts_conf" || return 125
  test ! -s "$containers_conf" && test ! -s "$mounts_conf" || return 125
  test "$(/usr/bin/stat -c '%a' "$state/config")" = 500 || return 125
  test "$(/usr/bin/stat -c '%a' "$state/config/containers")" = 500 \
    || return 125
  test "$(/usr/bin/stat -c '%a' "$containers_conf")" = 400 || return 125
  test "$(/usr/bin/stat -c '%a' "$mounts_conf")" = 400 || return 125
  local unexpected_config
  unexpected_config=$(/usr/bin/find "$state/config" -mindepth 1 \
    ! -path "$state/config/containers" \
    ! -path "$containers_conf" ! -path "$mounts_conf" -print -quit) \
    || return 125
  test -z "$unexpected_config" || return 125
  test "$(/usr/bin/id -u)" -ne 0 || return 125

  local receipt_dir="$receipt_root/controller/$receipt_name"
  test ! -e "$receipt_dir" || return 125
  mkdir -p "$receipt_dir" || return 125
  chmod 700 "$receipt_dir" || return 125

  local -a podman_environment=(
    /usr/bin/env -i
    "HOME=$state/home"
    "XDG_CONFIG_HOME=$state/config"
    "XDG_RUNTIME_DIR=$state/runtime"
    "CONTAINERS_CONF=$containers_conf"
    "PATH=/usr/bin:/bin"
  )
  local -a podman_command=(
    /usr/bin/podman
    --root "$state/root"
    --runroot "$state/runroot"
    --storage-driver vfs
    --runtime /usr/bin/crun
    --conmon /usr/bin/conmon
    --cgroup-manager cgroupfs
    --events-backend file
  )

  local controller_stdin=
  local controller_stdin_sha256=
  if [[ ${1:-} = --controller-stdin ]]; then
    test "$#" -ge 4 || return 125
    controller_stdin=$2
    controller_stdin_sha256=$3
    shift 3 || return 125
    [[ "$controller_stdin" = /* && "$controller_stdin" != *$'\n'* ]] \
      || return 125
    [[ "$controller_stdin_sha256" =~ ^[0-9a-f]{64}$ ]] || return 125
    test -f "$controller_stdin" && test ! -L "$controller_stdin" \
      || return 125
  fi

  printf '%q ' "${podman_environment[@]}" "${podman_command[@]}" version \
    > "$receipt_dir/version-argv.txt" || return 125
  printf '\n' >> "$receipt_dir/version-argv.txt" || return 125

  {
    printf '%s  %s\n' \
      "$CQ_PODMAN_SHA256" /usr/bin/podman \
      "$CQ_CRUN_SHA256" /usr/bin/crun \
      "$CQ_CONMON_SHA256" /usr/bin/conmon \
      "$CQ_PASTA_SHA256" /usr/bin/pasta \
      "$CQ_EMPTY_CONFIG_SHA256" "$containers_conf" \
      "$CQ_EMPTY_CONFIG_SHA256" "$mounts_conf"
    if [[ -n "$controller_stdin" ]]; then
      printf '%s  %s\n' "$controller_stdin_sha256" "$controller_stdin"
    fi
  } \
    | /usr/bin/sha256sum -c - > "$receipt_dir/controller.sha256" \
    || return 125
  if {
    "${podman_environment[@]}" "${podman_command[@]}" version &&
      /usr/bin/id -u &&
      /usr/bin/env -i PATH=/usr/bin:/bin /usr/bin/crun --version &&
      /usr/bin/env -i PATH=/usr/bin:/bin /usr/bin/conmon --version &&
      /usr/bin/env -i PATH=/usr/bin:/bin /usr/bin/pasta --version
  } > "$receipt_dir/versions.txt" 2>&1; then
    :
  else
    return 125
  fi
  printf '%q ' "${podman_environment[@]}" "${podman_command[@]}" "$@" \
    > "$receipt_dir/argv.txt" || return 125
  printf '\n' >> "$receipt_dir/argv.txt" || return 125

  local status
  if [[ -n "$controller_stdin" ]]; then
    if "${podman_environment[@]}" "${podman_command[@]}" "$@" \
      < "$controller_stdin"; then
      status=0
    else
      status=$?
    fi
  else
    if "${podman_environment[@]}" "${podman_command[@]}" "$@"; then
      status=0
    else
      status=$?
    fi
  fi
  {
    printf '%s  %s\n' \
      "$CQ_PODMAN_SHA256" /usr/bin/podman \
      "$CQ_CRUN_SHA256" /usr/bin/crun \
      "$CQ_CONMON_SHA256" /usr/bin/conmon \
      "$CQ_PASTA_SHA256" /usr/bin/pasta \
      "$CQ_EMPTY_CONFIG_SHA256" "$containers_conf" \
      "$CQ_EMPTY_CONFIG_SHA256" "$mounts_conf"
    if [[ -n "$controller_stdin" ]]; then
      printf '%s  %s\n' "$controller_stdin_sha256" "$controller_stdin"
    fi
  } \
    | /usr/bin/sha256sum -c - >> "$receipt_dir/controller.sha256" \
    || return 125
  printf '%s\n' "$status" > "$receipt_dir/status.txt" || return 125
  local receipt_tmp receipt_tmp_name
  receipt_tmp=$(/usr/bin/mktemp "$receipt_dir/.receipt.sha256.XXXXXX") \
    || return 125
  receipt_tmp_name=${receipt_tmp##*/}
  if (
    cd "$receipt_dir" &&
      /usr/bin/sha256sum \
        controller.sha256 versions.txt version-argv.txt argv.txt status.txt \
        > "$receipt_tmp"
  ); then
    :
  else
    return 125
  fi
  cq_validate_sha256_manifest "$receipt_dir" "$receipt_tmp_name" \
    controller.sha256 versions.txt version-argv.txt argv.txt status.txt \
    || return 125
  /usr/bin/mv --no-target-directory \
    "$receipt_tmp" "$receipt_dir/receipt.sha256" || return 125
  local recorded_status
  recorded_status=$(cq_controller_receipt_status "$receipt_dir") || return 125
  test "$recorded_status" -eq "$status" || return 125
  return "$status"
}
```

Acquisition, construction, and execution are coordinator-owned phases. Call
`cq_prepare_podman_state` once for each fresh acquisition, construction, and
execution state before its first controller action. A retained acquisition
state must already have that closed configuration; adding it after acquisition
cannot establish the earlier controller receipts. Construction starts from an
OCI export of the acquired base but otherwise uses fresh Podman state. It has no
Cargo home and its context contains only the reviewed Containerfile, input
manifest, and public input bytes. The source bundle is reviewed before
construction. Export and load must preserve the accepted base
platform-manifest and config digests; a changed digest stops the build. The
build itself uses `--network=none --pull=never`. After iproute2 is installed
from held bytes, a build stage requires empty IPv4 and IPv6 route tables and
`ENETUNREACH` for both numeric TEST-NET probes. Its completed receipt is
retained in the image and repeated in the later preflight; the controller's
build argv and the build log bind that observation to the network-denied build.

Use this exact staging and offline-construction procedure after the source and
input bundle receives a clean review. It does not execute the derived image or
any proof candidate:

```sh
set -euo pipefail

CQ_REPO=$(git rev-parse --show-toplevel)
CQ_PROOF="$CQ_REPO/proofs/exact-target-tough"
CQ_ACQ=${CQ_ACQ:?set the frozen executor-acquisition directory}
CQ_PUBLIC=${CQ_PUBLIC:?set the frozen public executor-documentation directory}
CQ_ACQ_STATE=${CQ_ACQ_STATE:?set the task-owned Podman acquisition-state directory}
CQ_BUILD=${CQ_BUILD:?set a fresh task-owned executor-build directory}
CQ_BUILD_STATE=${CQ_BUILD_STATE:?set a fresh task-owned Podman build-state directory}
CQ_BUILD_EVIDENCE=${CQ_BUILD_EVIDENCE:?set a fresh task-owned build-evidence directory}
CQ_BASE=docker.io/library/rust@sha256:cdb2da72943ec036bf0c731ef5ac9e5fb2b1d17d3a9256a581bad64cf6fc093d
CQ_TAG=localhost/codiquary-tough-proof:cycle-3-reviewed-source

test ! -e "$CQ_BUILD"
test ! -e "$CQ_BUILD_STATE"
test ! -e "$CQ_BUILD_EVIDENCE"
mkdir -p "$CQ_BUILD/context/inputs/base" \
  "$CQ_BUILD_EVIDENCE/host-only-controller"
chmod 700 "$CQ_BUILD_EVIDENCE/host-only-controller"
cq_prepare_podman_state "$CQ_BUILD_STATE"

python3 - "$CQ_BUILD/context" "$CQ_BUILD_STATE" "$CQ_ACQ_STATE" \
  "$CQ_BUILD_EVIDENCE/host-only-controller" <<'PY'
from pathlib import Path
import sys

context, build_state, acquisition_state, receipts = (
    Path(value).resolve() for value in sys.argv[1:]
)
for name, protected in (
    ("build state", build_state),
    ("acquisition state", acquisition_state),
    ("controller receipts", receipts),
):
    if context == protected or context in protected.parents or protected in context.parents:
        raise SystemExit(f"build context overlaps {name}")
PY

install -m 0644 "$CQ_PROOF/executor/Containerfile" \
  "$CQ_BUILD/context/Containerfile"
install -m 0644 "$CQ_PROOF/executor/inputs.sha256" \
  "$CQ_BUILD/context/inputs.sha256"
install -m 0644 "$CQ_PUBLIC/docker-rust-bookworm-Dockerfile" \
  "$CQ_BUILD/context/inputs/base/Dockerfile"
install -m 0644 "$CQ_PUBLIC/rust-image-metadata/bookworm/manifest.json" \
  "$CQ_BUILD/context/inputs/base/manifest.json"
install -m 0644 "$CQ_PUBLIC/rust-image-metadata/bookworm/config.json" \
  "$CQ_BUILD/context/inputs/base/config.json"

while read -r expected relative; do
  case "$relative" in
    base/Dockerfile|base/manifest.json|base/config.json) continue ;;
    base/image-inspect.json) source="$CQ_ACQ/base-image-inspect.json" ;;
    *) source="$CQ_ACQ/$relative" ;;
  esac
  test -f "$source"
  install -D -m 0644 "$source" "$CQ_BUILD/context/inputs/$relative"
done < "$CQ_PROOF/executor/inputs.sha256"

(
  cd "$CQ_BUILD/context/inputs"
  sha256sum --check ../inputs.sha256
  sha256sum --check apt-lists.sha256
  (cd debs && sha256sum --check ../debs.sha256)
  (cd rust-dist && sha256sum --check inputs.sha256)
)
awk '{print $2}' "$CQ_BUILD/context/inputs.sha256" | LC_ALL=C sort \
  > "$CQ_BUILD/expected-inputs.txt"
find "$CQ_BUILD/context/inputs" -type f -printf '%P\n' | LC_ALL=C sort \
  > "$CQ_BUILD/actual-inputs.txt"
cmp "$CQ_BUILD/expected-inputs.txt" "$CQ_BUILD/actual-inputs.txt"
sha256sum "$CQ_BUILD/context/Containerfile" \
  "$CQ_BUILD/context/inputs.sha256" \
  > "$CQ_BUILD_EVIDENCE/executor-source.sha256"

CQ_BUILD_CONTROLLER="$CQ_BUILD_EVIDENCE/host-only-controller"

cq_oci_controller "$CQ_ACQ_STATE" "$CQ_BUILD_CONTROLLER" base-save \
  save --format oci-archive \
  --output "$CQ_BUILD_EVIDENCE/base.oci.tar" "$CQ_BASE"

python3 - "$CQ_BUILD_EVIDENCE/base.oci.tar" \
  "$CQ_BUILD_EVIDENCE/base-oci-identity.json" <<'PY'
import hashlib
import json
import pathlib
import sys
import tarfile

archive = pathlib.Path(sys.argv[1])
output = pathlib.Path(sys.argv[2])
archive_hash = hashlib.sha256()
with archive.open("rb") as stream:
    while chunk := stream.read(1024 * 1024):
        archive_hash.update(chunk)
with tarfile.open(archive, "r:*") as bundle:
    index_bytes = bundle.extractfile("index.json").read()
    index = json.loads(index_bytes)
    if len(index["manifests"]) != 1:
        raise SystemExit("expected one OCI manifest descriptor")
    descriptor = index["manifests"][0]
    manifest_digest = descriptor["digest"]
    manifest_bytes = bundle.extractfile(
        "blobs/sha256/" + manifest_digest.removeprefix("sha256:")
    ).read()
    if "sha256:" + hashlib.sha256(manifest_bytes).hexdigest() != manifest_digest:
        raise SystemExit("OCI manifest digest mismatch")
    manifest = json.loads(manifest_bytes)
    config_digest = manifest["config"]["digest"]
    config_bytes = bundle.extractfile(
        "blobs/sha256/" + config_digest.removeprefix("sha256:")
    ).read()
    if "sha256:" + hashlib.sha256(config_bytes).hexdigest() != config_digest:
        raise SystemExit("OCI config digest mismatch")
    config = json.loads(config_bytes)

identity = {
    "archive_sha256": archive_hash.hexdigest(),
    "architecture": config["architecture"],
    "config_sha256": config_digest.removeprefix("sha256:"),
    "diff_ids": config["rootfs"]["diff_ids"],
    "layer_digests": [entry["digest"] for entry in manifest["layers"]],
    "manifest_sha256": manifest_digest.removeprefix("sha256:"),
    "os": config["os"],
}
output.write_text(json.dumps(identity, indent=2, sort_keys=True) + "\n")
PY

python3 -I - "$CQ_BUILD_EVIDENCE/base-oci-identity.json" <<'PY'
import json
import pathlib
import sys

identity = json.loads(pathlib.Path(sys.argv[1]).read_text())
if identity["manifest_sha256"] != "cdb2da72943ec036bf0c731ef5ac9e5fb2b1d17d3a9256a581bad64cf6fc093d":
    raise SystemExit("saved base manifest digest mismatch")
if identity["config_sha256"] != "ef460ef3675d3011ccfbd2bfd1c9239f04c044eee0cc0a3de6aecfd9f390bf26":
    raise SystemExit("saved base config digest mismatch")
if identity["architecture"] != "amd64":
    raise SystemExit("saved base architecture mismatch")
if identity["os"] != "linux":
    raise SystemExit("saved base OS mismatch")
PY

cq_oci_controller "$CQ_BUILD_STATE" "$CQ_BUILD_CONTROLLER" base-load \
  load \
  --input "$CQ_BUILD_EVIDENCE/base.oci.tar" \
  > "$CQ_BUILD_EVIDENCE/base-load.txt"
cq_oci_controller "$CQ_BUILD_STATE" "$CQ_BUILD_CONTROLLER" base-inspect \
  image inspect "$CQ_BASE" \
  > "$CQ_BUILD_EVIDENCE/base-after-load.json"
test "$(cq_oci_controller "$CQ_BUILD_STATE" "$CQ_BUILD_CONTROLLER" \
  base-digest image inspect --format '{{.Digest}}' "$CQ_BASE")" = \
  sha256:cdb2da72943ec036bf0c731ef5ac9e5fb2b1d17d3a9256a581bad64cf6fc093d

cq_oci_controller "$CQ_BUILD_STATE" "$CQ_BUILD_CONTROLLER" \
  derived-offline-build build \
  --network=none --pull=never --platform linux/amd64 \
  --file "$CQ_BUILD/context/Containerfile" \
  --tag "$CQ_TAG" "$CQ_BUILD/context" \
  > "$CQ_BUILD_EVIDENCE/offline-build.txt" 2>&1
cq_oci_controller "$CQ_BUILD_STATE" "$CQ_BUILD_CONTROLLER" \
  derived-inspect image inspect "$CQ_TAG" \
  > "$CQ_BUILD_EVIDENCE/derived-build-inspect.json"
cq_oci_controller "$CQ_BUILD_STATE" "$CQ_BUILD_CONTROLLER" derived-save \
  save --format oci-archive \
  --output "$CQ_BUILD_EVIDENCE/executor.oci.tar" "$CQ_TAG"

python3 - "$CQ_BUILD_EVIDENCE/executor.oci.tar" \
  "$CQ_BUILD_EVIDENCE/derived-oci-identity.json" <<'PY'
import hashlib
import json
import pathlib
import sys
import tarfile

archive = pathlib.Path(sys.argv[1])
output = pathlib.Path(sys.argv[2])
archive_hash = hashlib.sha256()
with archive.open("rb") as stream:
    while chunk := stream.read(1024 * 1024):
        archive_hash.update(chunk)
with tarfile.open(archive, "r:*") as bundle:
    index = json.loads(bundle.extractfile("index.json").read())
    if len(index["manifests"]) != 1:
        raise SystemExit("expected one OCI manifest descriptor")
    descriptor = index["manifests"][0]
    manifest_digest = descriptor["digest"]
    manifest_bytes = bundle.extractfile(
        "blobs/sha256/" + manifest_digest.removeprefix("sha256:")
    ).read()
    if "sha256:" + hashlib.sha256(manifest_bytes).hexdigest() != manifest_digest:
        raise SystemExit("OCI manifest digest mismatch")
    manifest = json.loads(manifest_bytes)
    config_digest = manifest["config"]["digest"]
    config_bytes = bundle.extractfile(
        "blobs/sha256/" + config_digest.removeprefix("sha256:")
    ).read()
    if "sha256:" + hashlib.sha256(config_bytes).hexdigest() != config_digest:
        raise SystemExit("OCI config digest mismatch")
    config = json.loads(config_bytes)

identity = {
    "archive_sha256": archive_hash.hexdigest(),
    "architecture": config["architecture"],
    "config_sha256": config_digest.removeprefix("sha256:"),
    "diff_ids": config["rootfs"]["diff_ids"],
    "layer_digests": [entry["digest"] for entry in manifest["layers"]],
    "manifest_sha256": manifest_digest.removeprefix("sha256:"),
    "os": config["os"],
}
if identity["architecture"] != "amd64" or identity["os"] != "linux":
    raise SystemExit("derived image is not linux/amd64")
output.write_text(json.dumps(identity, indent=2, sort_keys=True) + "\n")
PY

sha256sum \
  "$CQ_BUILD_EVIDENCE/base.oci.tar" \
  "$CQ_BUILD_EVIDENCE/base-oci-identity.json" \
  "$CQ_BUILD_EVIDENCE/base-load.txt" \
  "$CQ_BUILD_EVIDENCE/base-after-load.json" \
  "$CQ_BUILD_EVIDENCE/offline-build.txt" \
  "$CQ_BUILD_EVIDENCE/derived-build-inspect.json" \
  "$CQ_BUILD_EVIDENCE/executor.oci.tar" \
  "$CQ_BUILD_EVIDENCE/derived-oci-identity.json" \
  > "$CQ_BUILD_EVIDENCE/executor-build-evidence.sha256"
```

The tag in that procedure is only a build-time locator. It is never an
admitted executor root. Freeze the OCI archive, its parsed identity, and build
inspection together with every host-only controller receipt and the retained
build-network-denial bytes. The admission preflight below then records the
embedded package and component inventories, tool identities, and libclang
target. Freeze and review those bytes with the source bundle and Cargo
observations. The reviewed
platform-manifest digest identifies the held OCI image; its reviewed config
digest is also the local image ID used by Podman after an offline load. A tag,
an assumed output digest, or an unreviewed local image ID is insufficient.

The executing host is also fixed for the admitted Linux observation. Immediately
before every container, rehash `/usr/bin/podman`, `/usr/bin/crun`,
`/usr/bin/conmon`, and `/usr/bin/pasta` against the reviewed runtime receipt.
The current selection is Podman 6.1.0 with crun 1.29.1, task-owned VFS state,
`cgroupfs`, and the `file` event backend. A changed binary, runtime selection,
helper, image digest, or Podman state stops the run and renews review.

The Linux phase environment does not use rustup mediation. It exposes Cargo,
rustc, rustdoc, rustfmt, `cargo-fmt`, `clippy-driver`, and `cargo-clippy` only
from
`/usr/local/rustup/toolchains/1.98.1-x86_64-unknown-linux-gnu/bin`, sets the
Cargo and Rust front-end variables to those direct files, and omits
`RUSTUP_HOME` and `RUSTUP_TOOLCHAIN`. The image build and every admission
preflight require and hash those files. The `/usr/local/cargo/bin` rustup proxy
directory is not on the phase `PATH`.

No first-slice or final command may reuse the resolution cache. Before either
execution, the coordinator populates a fresh task-owned Cargo home from both
reviewed locks and verifies every cached registry archive against its lockfile
checksum. Acquisition is allowed network access; execution is not. The verified
resolution home is never mounted into an executor. Instead, construct a new
execution home containing only the crates.io `registry` subtree. Reject
configuration, credentials, symlinks, special files, alternate registries, and
every path outside the explicit cache/index/source shapes below. Normalize it
read-only. Reconstruct every `registry/src` tree from the corresponding
lock-authenticated `.crate` archive instead of copying resolution-home source
bytes; copy only Cargo's regular `.cargo-ok` completion marker. Bind every
relative path, type, mode, size, file hash, and archive-to-source tree digest in
one inventory. Review that inventory before loading or running the executor.

```sh
set -euo pipefail
CQ_RESOLUTION_CARGO_HOME=${CQ_RESOLUTION_CARGO_HOME:?set the verified resolution Cargo home}
CQ_EXECUTION_CARGO_HOME=${CQ_EXECUTION_CARGO_HOME:?set a new execution Cargo home}
CQ_CARGO_INVENTORY=${CQ_CARGO_INVENTORY:?set a new inventory JSON path}
CQ_REPO=$(git rev-parse --show-toplevel)
CQ_PROOF="$CQ_REPO/proofs/exact-target-tough"

test -d "$CQ_RESOLUTION_CARGO_HOME"
test ! -e "$CQ_EXECUTION_CARGO_HOME"
test ! -e "$CQ_CARGO_INVENTORY"
mkdir -p "$(dirname "$CQ_CARGO_INVENTORY")"
printf '%s  %s\n' \
  1d69534c34fc55d999c7cac409c5b44d2f036e6612e37a3021eb6f3d01cf57f4 \
  "$CQ_PROOF/Cargo.lock" \
  12c719f55434cde51602fc3c9da6145d509fac93831c80d529f5f8b33930b5fb \
  "$CQ_PROOF/locks/tough-workspace.Cargo.lock" \
  | sha256sum -c -

python3 - "$CQ_RESOLUTION_CARGO_HOME" "$CQ_EXECUTION_CARGO_HOME" \
  "$CQ_CARGO_INVENTORY" "$CQ_PROOF/Cargo.lock" \
  "$CQ_PROOF/locks/tough-workspace.Cargo.lock" <<'PY'
import hashlib
import json
import os
from pathlib import Path, PurePosixPath
import shutil
import stat
import sys
import tarfile
import tomllib

source_home = Path(sys.argv[1])
execution_home = Path(sys.argv[2])
inventory_path = Path(sys.argv[3])
lock_paths = [Path(sys.argv[4]), Path(sys.argv[5])]
source_id = "index.crates.io-1949cf8c6b5b557f"
crates_io_source = "registry+https://github.com/rust-lang/crates.io-index"
prohibited = {"config", "config.toml", "credentials", "credentials.toml"}

for name in prohibited:
    if (source_home / name).exists() or (source_home / name).is_symlink():
        raise SystemExit(f"prohibited resolution Cargo control file: {name}")
source_registry = source_home / "registry"
if not source_registry.is_dir() or source_registry.is_symlink():
    raise SystemExit("resolution Cargo home has no regular registry directory")

required = {}
for lock_path in lock_paths:
    lock = tomllib.loads(lock_path.read_text(encoding="utf-8"))
    for package in lock["package"]:
        source = package.get("source")
        if source is None:
            continue
        checksum = package.get("checksum")
        if source != crates_io_source or checksum is None:
            raise SystemExit(
                f"unsupported non-path lock source: {package['name']} {package['version']}"
            )
        key = (package["name"], package["version"])
        previous = required.setdefault(key, checksum)
        if previous != checksum:
            raise SystemExit(f"conflicting locked checksum: {key}")

required_names = {f"{name}-{version}" for name, version in required}

def allowed(relative: Path, kind: str) -> bool:
    parts = relative.parts
    if parts == ("registry",):
        return kind == "directory"
    if parts == ("registry", "CACHEDIR.TAG"):
        return kind == "file"
    if len(parts) < 2 or parts[0] != "registry":
        return False
    area = parts[1]
    if area == "cache":
        if len(parts) == 2:
            return kind == "directory"
        if len(parts) == 3:
            return parts[2] == source_id and kind == "directory"
        return (
            len(parts) == 4
            and parts[2] == source_id
            and parts[3].endswith(".crate")
            and kind == "file"
        )
    if area == "index":
        if len(parts) == 2:
            return kind == "directory"
        if len(parts) == 3:
            return parts[2] == source_id and kind == "directory"
        if parts[2] != source_id:
            return False
        if len(parts) == 4 and parts[3] == "config.json":
            return kind == "file"
        if parts[3] == ".cache":
            return kind in {"directory", "file"}
        return False
    if area == "src":
        if len(parts) == 2:
            return kind == "directory"
        if len(parts) == 3:
            return parts[2] == source_id and kind == "directory"
        if parts[2] != source_id:
            return False
        if len(parts) == 4:
            return kind == "directory" and parts[3] in required_names
        return kind in {"directory", "file"}
    return False

for path in [source_registry, *source_registry.rglob("*")]:
    relative = Path("registry") if path == source_registry else Path("registry") / path.relative_to(source_registry)
    if "\n" in relative.as_posix():
        raise SystemExit(f"newline in Cargo path: {relative!s}")
    mode = path.lstat().st_mode
    if stat.S_ISLNK(mode):
        raise SystemExit(f"symlink in Cargo registry: {relative!s}")
    if stat.S_ISDIR(mode):
        kind = "directory"
    elif stat.S_ISREG(mode):
        kind = "file"
    else:
        raise SystemExit(f"special Cargo registry entry: {relative!s}")
    if not allowed(relative, kind):
        raise SystemExit(f"unallowlisted Cargo registry entry: {relative!s}")

cache_root = source_registry / "cache" / source_id
index_root = source_registry / "index" / source_id
source_root = source_registry / "src" / source_id
for label, path in (("cache", cache_root), ("index", index_root), ("source", source_root)):
    if not path.is_dir() or path.is_symlink():
        raise SystemExit(f"missing regular crates.io {label} directory")
expected_archives = {f"{name}-{version}.crate" for name, version in required}
observed_archives = {path.name for path in cache_root.iterdir() if path.is_file()}
if observed_archives != expected_archives:
    raise SystemExit(
        "registry archive set mismatch: "
        f"missing={sorted(expected_archives - observed_archives)!r} "
        f"extra={sorted(observed_archives - expected_archives)!r}"
    )
observed_sources = {path.name for path in source_root.iterdir() if path.is_dir()}
if observed_sources != required_names:
    raise SystemExit(
        "registry source set mismatch: "
        f"missing={sorted(required_names - observed_sources)!r} "
        f"extra={sorted(observed_sources - required_names)!r}"
    )

execution_home.mkdir(mode=0o755)
execution_registry = execution_home / "registry"
execution_registry.mkdir()
for name in ("CACHEDIR.TAG", "cache", "index"):
    source = source_registry / name
    destination = execution_registry / name
    if source.is_dir():
        shutil.copytree(source, destination)
    elif source.is_file() and not source.is_symlink():
        shutil.copy2(source, destination)
    else:
        raise SystemExit(f"missing regular registry entry: {name}")
execution_source_root = execution_registry / "src" / source_id
execution_source_root.mkdir(parents=True)

def hash_file(path: Path) -> str:
    digest = hashlib.sha256()
    with path.open("rb") as stream:
        while chunk := stream.read(1024 * 1024):
            digest.update(chunk)
    return digest.hexdigest()

source_archive_bindings = []
for (name, version), expected_hash in sorted(required.items()):
    package_directory = f"{name}-{version}"
    archive = cache_root / f"{package_directory}.crate"
    actual_hash = hash_file(archive)
    if actual_hash != expected_hash:
        raise SystemExit(f"locked checksum mismatch: {name} {version}")

    destination_root = execution_source_root / package_directory
    destination_root.mkdir()
    records = {}
    with tarfile.open(archive, "r:gz") as bundle:
        for member in bundle.getmembers():
            raw_name = member.name[:-1] if member.name.endswith("/") else member.name
            member_path = PurePosixPath(raw_name)
            if (
                not raw_name
                or "\n" in raw_name
                or member_path.is_absolute()
                or member_path.as_posix() != raw_name
                or ".." in member_path.parts
                or member_path.parts[0] != package_directory
            ):
                raise SystemExit(f"unsafe crate member: {member.name!r}")
            relative_parts = member_path.parts[1:]
            relative = "/".join(relative_parts) if relative_parts else "."
            if relative in records:
                raise SystemExit(f"duplicate crate member: {member.name!r}")
            if relative == "." and not member.isdir():
                raise SystemExit(f"crate root is not a directory: {member.name!r}")
            if member.isdir():
                records[relative] = {
                    "bytes": 0,
                    "member": member,
                    "path": relative,
                    "sha256": None,
                    "type": "directory",
                }
            elif member.isreg():
                if relative == ".cargo-ok":
                    raise SystemExit("crate archive contains reserved .cargo-ok")
                records[relative] = {
                    "bytes": member.size,
                    "member": member,
                    "path": relative,
                    "sha256": None,
                    "type": "file",
                }
            else:
                raise SystemExit(f"non-regular crate member: {member.name!r}")

        for relative, record in sorted(
            records.items(), key=lambda item: (item[0].count("/"), item[0])
        ):
            if relative == ".":
                continue
            destination = destination_root.joinpath(*PurePosixPath(relative).parts)
            if record["type"] == "directory":
                destination.mkdir(parents=True, exist_ok=True)
                continue
            destination.parent.mkdir(parents=True, exist_ok=True)
            extracted = bundle.extractfile(record["member"])
            if extracted is None:
                raise SystemExit(f"crate member has no bytes: {relative}")
            digest = hashlib.sha256()
            written = 0
            with destination.open("xb") as output:
                while chunk := extracted.read(1024 * 1024):
                    output.write(chunk)
                    digest.update(chunk)
                    written += len(chunk)
            if written != record["bytes"]:
                raise SystemExit(f"crate member size changed: {relative}")
            record["sha256"] = digest.hexdigest()

    marker_source = source_root / package_directory / ".cargo-ok"
    marker_mode = marker_source.lstat().st_mode
    if not stat.S_ISREG(marker_mode) or marker_source.is_symlink():
        raise SystemExit(f"missing regular Cargo completion marker: {package_directory}")
    marker_destination = destination_root / ".cargo-ok"
    shutil.copyfile(marker_source, marker_destination)
    public_records = [
        {key: value for key, value in record.items() if key != "member"}
        for _, record in sorted(records.items())
    ]
    tree_bytes = json.dumps(
        public_records, separators=(",", ":"), sort_keys=True
    ).encode("utf-8")
    source_archive_bindings.append(
        {
            "archive_sha256": actual_hash,
            "cargo_ok_sha256": hash_file(marker_destination),
            "files": sum(record["type"] == "file" for record in public_records),
            "name": name,
            "source": crates_io_source,
            "source_directory": package_directory,
            "source_tree_sha256": hashlib.sha256(tree_bytes).hexdigest(),
            "version": version,
        }
    )

for path in sorted((execution_home / "registry").rglob("*"), reverse=True):
    os.chmod(path, 0o555 if path.is_dir() else 0o444)
os.chmod(execution_home / "registry", 0o555)
os.chmod(execution_home, 0o555)

entries = [
    {"bytes": 0, "mode": "0555", "path": ".", "sha256": None,
     "type": "directory"}
]
for path in [execution_home / "registry", *(execution_home / "registry").rglob("*")]:
    relative = path.relative_to(execution_home).as_posix()
    mode = path.lstat().st_mode
    if stat.S_ISDIR(mode):
        entries.append(
            {"bytes": 0, "mode": f"{stat.S_IMODE(mode):04o}", "path": relative,
             "sha256": None, "type": "directory"}
        )
    elif stat.S_ISREG(mode):
        digest = hashlib.sha256()
        with path.open("rb") as stream:
            while chunk := stream.read(1024 * 1024):
                digest.update(chunk)
        entries.append(
            {"bytes": path.stat().st_size, "mode": f"{stat.S_IMODE(mode):04o}",
             "path": relative, "sha256": digest.hexdigest(), "type": "file"}
        )
    else:
        raise SystemExit(f"copied non-regular Cargo entry: {relative}")

inventory = {
    "allowed_root_entries": ["registry"],
    "entries": sorted(entries, key=lambda entry: entry["path"]),
    "schema": "io.nisavid.codiquary.execution-cargo-home/v2",
    "source_id": source_id,
    "source_archive_bindings": source_archive_bindings,
}
inventory_path.write_text(json.dumps(inventory, indent=2, sort_keys=True) + "\n")
PY

sha256sum "$CQ_CARGO_INVENTORY" \
  > "$CQ_CARGO_INVENTORY.sha256"
```

The resolution home may contain Cargo acquisition locks or other coordinator
state, but none is copied. The execution home has exactly one top-level entry,
`registry`, and is mounted read-only. Every extracted source file now comes
from the lock-authenticated archive bytes; the resolution source tree supplies
only Cargo's recorded completion marker. The inventory is a renewed-review
input, not a self-approval receipt. Each phase receives six new task-owned
writable directories for target, temporary, home, output, fixture, and
prepared-source state. No writable directory crosses a phase boundary. No host
home, credential, socket, device, or unrelated path is mounted.

After the built OCI bundle and identity receive review, load that exact held
archive into empty execution state without network access. This step does not
start a container:

```sh
set -euo pipefail
CQ_EXECUTOR_ARCHIVE=${CQ_EXECUTOR_ARCHIVE:?set the reviewed executor.oci.tar path}
CQ_EXECUTOR_IDENTITY=${CQ_EXECUTOR_IDENTITY:?set the reviewed derived-oci-identity.json path}
CQ_EXECUTOR_IDENTITY_SHA256=${CQ_EXECUTOR_IDENTITY_SHA256:?set its reviewed SHA-256}
CQ_EXECUTION_STATE=${CQ_EXECUTION_STATE:?set a fresh task-owned Podman execution-state directory}
CQ_BOUNDARY_EVIDENCE=${CQ_BOUNDARY_EVIDENCE:?set a fresh host-only boundary-receipt directory}

test -f "$CQ_EXECUTOR_ARCHIVE"
test -f "$CQ_EXECUTOR_IDENTITY"
[[ "$CQ_EXECUTOR_IDENTITY_SHA256" =~ ^[0-9a-f]{64}$ ]]
printf '%s  %s\n' "$CQ_EXECUTOR_IDENTITY_SHA256" "$CQ_EXECUTOR_IDENTITY" \
  | sha256sum -c -
test ! -e "$CQ_EXECUTION_STATE"
test ! -e "$CQ_BOUNDARY_EVIDENCE"
mkdir -p "$CQ_BOUNDARY_EVIDENCE"
chmod 700 "$CQ_BOUNDARY_EVIDENCE"
cq_prepare_podman_state "$CQ_EXECUTION_STATE"

CQ_EXECUTOR_CONFIG_SHA256=$(python3 - "$CQ_EXECUTOR_ARCHIVE" \
  "$CQ_EXECUTOR_IDENTITY" <<'PY'
import hashlib
import json
import pathlib
import sys

archive = pathlib.Path(sys.argv[1])
identity = json.loads(pathlib.Path(sys.argv[2]).read_text())
archive_hash = hashlib.sha256()
with archive.open("rb") as stream:
    while chunk := stream.read(1024 * 1024):
        archive_hash.update(chunk)
if archive_hash.hexdigest() != identity["archive_sha256"]:
    raise SystemExit("executor OCI archive differs from reviewed identity")
if identity["architecture"] != "amd64" or identity["os"] != "linux":
    raise SystemExit("reviewed executor is not linux/amd64")
print(identity["config_sha256"])
PY
)
[[ "$CQ_EXECUTOR_CONFIG_SHA256" =~ ^[0-9a-f]{64}$ ]]
CQ_EXECUTOR_IMAGE_ID="sha256:$CQ_EXECUTOR_CONFIG_SHA256"

cq_oci_controller "$CQ_EXECUTION_STATE" "$CQ_BOUNDARY_EVIDENCE" \
  executor-load load --input "$CQ_EXECUTOR_ARCHIVE" \
  > "$CQ_BOUNDARY_EVIDENCE/load.txt"
cq_oci_controller "$CQ_EXECUTION_STATE" "$CQ_BOUNDARY_EVIDENCE" \
  executor-load-inspect image inspect \
  "$CQ_EXECUTOR_IMAGE_ID" \
  > "$CQ_BOUNDARY_EVIDENCE/load-image-inspect.json"

python3 -I - "$CQ_EXECUTOR_IDENTITY" \
  "$CQ_BOUNDARY_EVIDENCE/load-image-inspect.json" <<'PY'
import json
import pathlib
import sys

identity = json.loads(pathlib.Path(sys.argv[1]).read_text())
inspection = json.loads(pathlib.Path(sys.argv[2]).read_text())
if len(inspection) != 1:
    raise SystemExit("expected one local image inspection")
image = inspection[0]
if image["Id"].removeprefix("sha256:") != identity["config_sha256"]:
    raise SystemExit("loaded executor image ID mismatch")
if image["Architecture"] != identity["architecture"]:
    raise SystemExit("loaded executor architecture mismatch")
if image["Os"] != identity["os"]:
    raise SystemExit("loaded executor OS mismatch")
if image["RootFS"]["Layers"] != identity["diff_ids"]:
    raise SystemExit("loaded executor rootfs layers mismatch")
PY
```

Set the Linux boundary from a shell with no ambient Podman variables:

```sh
set -euo pipefail
CQ_REPO=$(git rev-parse --show-toplevel)
CQ_PROOF="$CQ_REPO/proofs/exact-target-tough"
CQ_INPUTS=${CQ_INPUTS:?set the verified public-source directory}
CQ_CARGO_HOME=${CQ_CARGO_HOME:?set the reviewed registry-only execution Cargo home}
CQ_CARGO_INVENTORY=${CQ_CARGO_INVENTORY:?set the reviewed Cargo-home inventory JSON}
CQ_CARGO_INVENTORY_SHA256=${CQ_CARGO_INVENTORY_SHA256:?set its reviewed SHA-256}
CQ_PHASE_STATE_ROOT=${CQ_PHASE_STATE_ROOT:?set a fresh empty task-owned phase-state directory}
CQ_EXECUTION_STATE=${CQ_EXECUTION_STATE:?set the admitted Podman execution-state directory}
CQ_BOUNDARY_EVIDENCE=${CQ_BOUNDARY_EVIDENCE:?set the host-only boundary-receipt directory}
CQ_EXECUTOR_IDENTITY=${CQ_EXECUTOR_IDENTITY:?set the reviewed derived-oci-identity.json path}
CQ_EXECUTOR_IDENTITY_SHA256=${CQ_EXECUTOR_IDENTITY_SHA256:?set its reviewed SHA-256}

for directory in \
  "$CQ_REPO" "$CQ_INPUTS" "$CQ_CARGO_HOME" "$CQ_PHASE_STATE_ROOT" \
  "$CQ_EXECUTION_STATE" "$CQ_BOUNDARY_EVIDENCE"; do
  [[ "$directory" = /* && "$directory" != *,* && \
    "$directory" != *$'\n'* && -d "$directory" ]]
done
[[ "$CQ_EXECUTOR_IDENTITY" = /* && "$CQ_EXECUTOR_IDENTITY" != *,* && \
  "$CQ_EXECUTOR_IDENTITY" != *$'\n'* && -f "$CQ_EXECUTOR_IDENTITY" ]]
[[ "$CQ_CARGO_INVENTORY" = /* && "$CQ_CARGO_INVENTORY" != *,* && \
  "$CQ_CARGO_INVENTORY" != *$'\n'* && -f "$CQ_CARGO_INVENTORY" ]]
[[ "$CQ_EXECUTOR_IDENTITY_SHA256" =~ ^[0-9a-f]{64}$ ]]
[[ "$CQ_CARGO_INVENTORY_SHA256" =~ ^[0-9a-f]{64}$ ]]
printf '%s  %s\n' \
  "$CQ_EXECUTOR_IDENTITY_SHA256" "$CQ_EXECUTOR_IDENTITY" \
  "$CQ_CARGO_INVENTORY_SHA256" "$CQ_CARGO_INVENTORY" \
  | sha256sum -c -
test -z "$(find "$CQ_PHASE_STATE_ROOT" -mindepth 1 -print -quit)"

python3 - \
  "$CQ_REPO" "$CQ_INPUTS" "$CQ_CARGO_HOME" "$CQ_PHASE_STATE_ROOT" \
  "$CQ_EXECUTION_STATE" \
  "$CQ_BOUNDARY_EVIDENCE" "$CQ_EXECUTOR_IDENTITY" \
  "$CQ_CARGO_INVENTORY" <<'PY'
from pathlib import Path
import sys

names = (
    "repo", "inputs", "cargo", "phase_state", "execution_state",
    "boundary_evidence", "executor_identity", "cargo_inventory",
)
paths = {name: Path(value).resolve() for name, value in zip(names, sys.argv[1:], strict=True)}
mounted = {name: paths[name] for name in names[:4]}
protected = {name: paths[name] for name in names[4:]}

def overlaps(left: Path, right: Path) -> bool:
    return left == right or left in right.parents or right in left.parents

for protected_name, protected_path in protected.items():
    for mounted_name, mounted_path in mounted.items():
        if overlaps(protected_path, mounted_path):
            raise SystemExit(
                f"host-only {protected_name} overlaps mounted {mounted_name}"
            )

mounted_items = list(mounted.items())
for index, (left_name, left_path) in enumerate(mounted_items):
    for right_name, right_path in mounted_items[index + 1:]:
        if overlaps(left_path, right_path):
            raise SystemExit(f"mounted path alias: {left_name} and {right_name}")
PY

CQ_EXECUTOR_MANIFEST_SHA256=$(python3 -c \
  'import json,sys; print(json.load(open(sys.argv[1]))["manifest_sha256"])' \
  "$CQ_EXECUTOR_IDENTITY")
CQ_EXECUTOR_CONFIG_SHA256=$(python3 -c \
  'import json,sys; print(json.load(open(sys.argv[1]))["config_sha256"])' \
  "$CQ_EXECUTOR_IDENTITY")
[[ "$CQ_EXECUTOR_MANIFEST_SHA256" =~ ^[0-9a-f]{64}$ ]]
[[ "$CQ_EXECUTOR_CONFIG_SHA256" =~ ^[0-9a-f]{64}$ ]]
CQ_EXECUTOR_IMAGE_ID="sha256:$CQ_EXECUTOR_CONFIG_SHA256"

CQ_PROOF_LOCK_SHA256=1d69534c34fc55d999c7cac409c5b44d2f036e6612e37a3021eb6f3d01cf57f4
CQ_TOUGH_LOCK_SHA256=12c719f55434cde51602fc3c9da6145d509fac93831c80d529f5f8b33930b5fb
CQ_RUST_TOOLCHAIN_BIN=/usr/local/rustup/toolchains/1.98.1-x86_64-unknown-linux-gnu/bin
CQ_NATIVE_PATH=$CQ_RUST_TOOLCHAIN_BIN:/opt/codiquary/bin:/usr/local/sbin:/usr/local/bin:/usr/sbin:/usr/bin:/sbin:/bin
CQ_LIBCLANG_PATH=/opt/codiquary/lib/libclang.so
CQ_RUST_TARGET=x86_64-unknown-linux-gnu
readonly CQ_EXECUTOR_IDENTITY CQ_EXECUTOR_MANIFEST_SHA256
readonly CQ_EXECUTOR_CONFIG_SHA256 CQ_EXECUTOR_IMAGE_ID
readonly CQ_EXECUTOR_IDENTITY_SHA256
readonly CQ_CARGO_HOME CQ_CARGO_INVENTORY CQ_CARGO_INVENTORY_SHA256
readonly CQ_PHASE_STATE_ROOT
readonly CQ_EXECUTION_STATE CQ_BOUNDARY_EVIDENCE
readonly CQ_PROOF_LOCK_SHA256 CQ_TOUGH_LOCK_SHA256
readonly CQ_RUST_TOOLCHAIN_BIN CQ_NATIVE_PATH CQ_LIBCLANG_PATH CQ_RUST_TARGET
```

Define the one Linux execution entry point. Its Podman options are part of the
evidence contract. `--network=none` creates an unconfigured network namespace;
the same container proves empty IPv4 and IPv6 route tables and requires numeric
TEST-NET connections to fail with `ENETUNREACH` before it executes the phase
command. The image root and all externally sourced input mounts are read-only.
Only the six externally sourced mounts in a new phase-state directory are
writable, and no writable directory is mounted into another phase. The
controller first verifies the exact ten declared host binds, then initializes
the retained container without starting its process. A second inspection must
expose the task-state `OCIConfigPath`. Admission copies those authoritative
bytes, rejects every other host bind, and records task-state runtime binds and
the closed runtime pseudo-filesystem allowlist separately.

Admission also reads the initialized process's mount namespace before start.
[Podman 6.1.0 initialization](https://github.com/containers/podman/blob/v6.1.0/libpod/container_internal.go#L1024-L1136)
performs OCI create, and its
[conmon runtime records the created process PID](https://github.com/containers/podman/blob/v6.1.0/libpod/oci_conmon_common.go#L1233-L1248)
before the state is saved; admission requires that positive inspected PID and a
stable process identity around the mountinfo read.
For the image root, every declared bind, and every task-state bind, it projects
the exact admitted source through the host mount table and requires the same
mountinfo device, root, source, filesystem, and access. The reviewed image and
task-owned Podman state control the image root; coordinator-selected paths
control declared binds; the closed Podman root and runroot control task-state
binds. Runtime pseudo-filesystems are accepted only at exact destinations in
the admitted OCI mount list. Security submounts are accepted only when the
specification's complete `maskedPaths` and `readonlyPaths` sets equal the
[Podman 6.1.0 generator](https://github.com/containers/podman/blob/v6.1.0/pkg/specgen/generate/config_linux.go#L99-L126)
defaults from [`go.podman.io/common` 0.69.1](https://github.com/podman-container-tools/container-libs/blob/common/v0.69.1/common/pkg/config/default.go#L38-L86),
including only the kernel-owned thermal-throttle paths matched by that tagged
implementation. Each observed security submount must then occupy one of those
exact destinations with a source equivalent to the kernel `/dev/null` selected
by crun, the
[crun 1.29.1 shared empty directory](https://github.com/containers/crun/blob/1.29.1/src/libcrun/status.c#L107-L125)
under the task's closed runtime state, crun's read-only tmpfs fallback, or a
read-only remount of the admitted parent pseudo-filesystem. In a rootless user
namespace, only crun's
[six required device destinations](https://github.com/containers/crun/blob/1.29.1/src/libcrun/linux.c#L2030-L2036)
may add mounts below `/dev`; each must project from the correspondingly numbered
kernel device on the host. The selected crun and the kernel control those
mounts.

The host retains that complete canonical projection and passes its exact bytes
to the attached start on standard input. The controller verifies those bytes
before and after start. Before the phase command can run, the in-container
preflight requires `/proc/self/mountinfo` to match the projection exactly and
emits only the container destinations, access, filesystem classes, source
control classes, and projection digest. A different start-time bind changes
the exact mount set or its device, root, source, filesystem, or access and
therefore stops before candidate execution. A path alias with the same device,
root, and mount source exposes the same effective source rather than different
bytes or authority.

Neither this function nor the OCI controller changes the caller's shell
options. The controller creates and retains a named container, validates its
declared mounts, initializes it, admits the effective OCI specification, and
only then starts it attached. It validates every completed controller receipt,
inspects a clean exited state, obtains the printed exit code through a
separately successful `podman wait`, proves that the effective specification
did not change across attached start, and removes the container through another
controller action. The function requires the inspected, waited, and
attached-start statuses to agree, and returns that status only after the
preflight, complete phase-state inventory, cleanup, and atomic boundary receipt
succeed. A boundary, controller, Podman, or evidence failure returns status
`125` without a completed `boundary.sha256`; a child that returns `125` has a
completed exit-state and boundary receipt. Callers expecting a nonzero child
status must use an `if` condition to observe it without changing their own
`errexit` policy.

```sh
cq_linux_run() {
  if [[ -z ${CQ_PHASE:-} ]]; then
    printf '%s\n' 'CQ_PHASE must name a receipt-safe phase' >&2
    return 125
  fi
  [[ "$CQ_PHASE" =~ ^[a-z0-9][a-z0-9-]*$ ]] || return 125
  test "${#CQ_PHASE}" -le 48 || return 125
  local phase_state="$CQ_PHASE_STATE_ROOT/$CQ_PHASE"
  local phase_target="$phase_state/target"
  local phase_tmp="$phase_state/tmp"
  local phase_home="$phase_state/home"
  local phase_output="$phase_state/output"
  local phase_fixtures="$phase_state/fixtures"
  local phase_work="$phase_state/work"
  local phase_boundary="$CQ_BOUNDARY_EVIDENCE/phases/$CQ_PHASE"
  local container_name="codiquary-$CQ_PHASE"
  test ! -e "$phase_state" || return 125
  test ! -e "$phase_boundary" || return 125
  mkdir -p \
    "$phase_target" "$phase_tmp" "$phase_home" "$phase_output" \
    "$phase_fixtures" "$phase_work" "$phase_boundary" || return 125
  chmod 700 \
    "$phase_state" "$phase_target" "$phase_tmp" "$phase_home" \
    "$phase_output" "$phase_fixtures" "$phase_work" \
    "$phase_boundary" || return 125

  local phase_command_stdin="$phase_boundary/phase-command-stdin.bin"
  if [[ -t 0 ]]; then
    : > "$phase_command_stdin" || return 125
  else
    /usr/bin/python3 -c \
      'import pathlib,sys; output=pathlib.Path(sys.argv[1]); total=0; stream=output.open("xb");
while chunk := sys.stdin.buffer.read(1024 * 1024):
 total += len(chunk)
 if total > 4 * 1024 * 1024: raise SystemExit("phase stdin exceeds 4 MiB")
 stream.write(chunk)
stream.close()' \
      "$phase_command_stdin" || return 125
  fi
  chmod 600 "$phase_command_stdin" || return 125

  printf '%s  %s\n' \
    "$CQ_EXECUTOR_IDENTITY_SHA256" "$CQ_EXECUTOR_IDENTITY" \
    "$CQ_CARGO_INVENTORY_SHA256" "$CQ_CARGO_INVENTORY" \
    > "$phase_boundary/reviewed-inputs.sha256" || return 125
  /usr/bin/sha256sum --check --strict \
    "$phase_boundary/reviewed-inputs.sha256" \
    > "$phase_boundary/reviewed-inputs-check.txt" \
    || return 125

  /usr/bin/python3 - "$CQ_CARGO_HOME" "$CQ_CARGO_INVENTORY" \
    "$phase_boundary/observed-cargo-home-inventory.json" <<'PY_INVENTORY' \
    || return 125
import hashlib
import json
from pathlib import Path
import stat
import sys

root = Path(sys.argv[1])
expected_path = Path(sys.argv[2])
output_path = Path(sys.argv[3])
if {path.name for path in root.iterdir()} != {"registry"}:
    raise SystemExit("execution Cargo home top-level allowlist changed")

entries = [
    {"bytes": 0, "mode": f"{stat.S_IMODE(root.lstat().st_mode):04o}",
     "path": ".", "sha256": None, "type": "directory"}
]
registry = root / "registry"
for path in [registry, *registry.rglob("*")]:
    relative = path.relative_to(root).as_posix()
    mode = path.lstat().st_mode
    if stat.S_ISLNK(mode):
        raise SystemExit(f"symlink in execution Cargo home: {relative}")
    if stat.S_ISDIR(mode):
        entry = {"bytes": 0, "mode": f"{stat.S_IMODE(mode):04o}",
                 "path": relative, "sha256": None, "type": "directory"}
    elif stat.S_ISREG(mode):
        digest = hashlib.sha256()
        with path.open("rb") as stream:
            while chunk := stream.read(1024 * 1024):
                digest.update(chunk)
        entry = {"bytes": path.stat().st_size,
                 "mode": f"{stat.S_IMODE(mode):04o}", "path": relative,
                 "sha256": digest.hexdigest(), "type": "file"}
    else:
        raise SystemExit(f"special execution Cargo entry: {relative}")
    entries.append(entry)

expected = json.loads(expected_path.read_text())
if (
    expected.get("schema") != "io.nisavid.codiquary.execution-cargo-home/v2"
    or not isinstance(expected.get("source_archive_bindings"), list)
):
    raise SystemExit("reviewed Cargo-home inventory has the wrong schema")
observed = {
    "allowed_root_entries": ["registry"],
    "entries": sorted(entries, key=lambda entry: entry["path"]),
    "schema": "io.nisavid.codiquary.execution-cargo-home/v2",
    "source_id": "index.crates.io-1949cf8c6b5b557f",
    "source_archive_bindings": expected["source_archive_bindings"],
}
if observed != expected:
    raise SystemExit("execution Cargo home differs from reviewed inventory")
output_path.write_text(json.dumps(observed, indent=2, sort_keys=True) + "\n")
PY_INVENTORY

  local image_receipt="$phase_boundary/image-inspect.json"
  local image_stderr="$phase_boundary/image-inspect-stderr.txt"
  local image_call_status
  if cq_oci_controller "$CQ_EXECUTION_STATE" "$CQ_BOUNDARY_EVIDENCE" \
    "$CQ_PHASE-image-inspect" image inspect "$CQ_EXECUTOR_IMAGE_ID" \
    > "$image_receipt" 2> "$image_stderr"; then
    image_call_status=0
  else
    image_call_status=$?
  fi
  local image_controller_status
  image_controller_status=$(cq_controller_receipt_status \
    "$CQ_BOUNDARY_EVIDENCE/controller/$CQ_PHASE-image-inspect") \
    || return 125
  test "$image_call_status" -eq "$image_controller_status" || return 125
  test "$image_controller_status" -eq 0 || return 125
  /usr/bin/python3 - "$CQ_EXECUTOR_IDENTITY" "$image_receipt" <<'PY_IMAGE' \
    || return 125
import json
from pathlib import Path
import sys

identity = json.loads(Path(sys.argv[1]).read_text())
inspection = json.loads(Path(sys.argv[2]).read_text())
if len(inspection) != 1:
    raise SystemExit("expected one local image inspection")
image = inspection[0]
if image["Id"].removeprefix("sha256:") != identity["config_sha256"]:
    raise SystemExit("local image ID does not match reviewed OCI config")
if image["Architecture"] != identity["architecture"] or image["Os"] != identity["os"]:
    raise SystemExit("local image platform changed")
if image["RootFS"]["Layers"] != identity["diff_ids"]:
    raise SystemExit("local image rootfs differs from reviewed OCI config")
PY_IMAGE

  local -a create_arguments=(
    create --name "$container_name" --interactive
    --pull=never
    --network=none
    --no-hosts
    --cap-drop=all
    --security-opt=no-new-privileges
    --read-only
    --read-only-tmpfs=false
    --ipc=private
    --pid=private
    --uts=private
    --pids-limit=2048
    --user=0:0
    --mount "type=bind,src=$CQ_REPO,dst=/repo,ro=true"
    --mount "type=bind,src=$CQ_INPUTS,dst=/inputs,ro=true"
    --mount "type=bind,src=$CQ_CARGO_HOME,dst=/cargo,ro=true"
    --mount "type=bind,src=$CQ_CARGO_INVENTORY,dst=/execution-cargo-home-inventory.json,ro=true"
    --mount "type=bind,src=$phase_target,dst=/phase/target"
    --mount "type=bind,src=$phase_tmp,dst=/phase/tmp"
    --mount "type=bind,src=$phase_home,dst=/phase/home"
    --mount "type=bind,src=$phase_output,dst=/phase/output"
    --mount "type=bind,src=$phase_fixtures,dst=/phase/fixtures"
    --mount "type=bind,src=$phase_work,dst=/repo/proofs/exact-target-tough/.work"
    "$CQ_EXECUTOR_IMAGE_ID"
    /usr/bin/env -i
    "HOME=/phase/home"
    "PATH=$CQ_NATIVE_PATH"
    "LC_ALL=C"
    "LANG=C"
    "CARGO_HOME=/cargo"
    "CARGO_TARGET_DIR=/phase/target"
    "CARGO_NET_OFFLINE=true"
    "CARGO_INCREMENTAL=0"
    "CARGO_TERM_COLOR=never"
    "TMPDIR=/phase/tmp"
    "CQ_RUST_TOOLCHAIN_BIN=$CQ_RUST_TOOLCHAIN_BIN"
    "CARGO=$CQ_RUST_TOOLCHAIN_BIN/cargo"
    "RUSTC=$CQ_RUST_TOOLCHAIN_BIN/rustc"
    "RUSTDOC=$CQ_RUST_TOOLCHAIN_BIN/rustdoc"
    "RUSTFMT=$CQ_RUST_TOOLCHAIN_BIN/rustfmt"
    "CC=/usr/bin/gcc"
    "CXX=/usr/bin/g++"
    "LD=/usr/bin/ld"
    "AR=/usr/bin/ar"
    "RANLIB=/usr/bin/ranlib"
    "CMAKE=/usr/bin/cmake"
    "CMAKE_GENERATOR=Unix Makefiles"
    "CMAKE_MAKE_PROGRAM=/usr/bin/make"
    "MAKE=/usr/bin/make"
    "PERL=/usr/bin/perl"
    "LIBCLANG_PATH=$CQ_LIBCLANG_PATH"
    "CQ_EXECUTOR_MANIFEST_SHA256=$CQ_EXECUTOR_MANIFEST_SHA256"
    "CQ_EXECUTOR_CONFIG_SHA256=$CQ_EXECUTOR_CONFIG_SHA256"
    "CQ_EXECUTOR_IMAGE_ID=$CQ_EXECUTOR_IMAGE_ID"
    "OPENSSL_NO_VENDOR=0"
    "AWS_LC_SYS_USE_SYSTEM=0"
    /bin/bash -c '
      set -euo pipefail
      printf "%s\n" CODIQUARY_EXECUTOR_PREFLIGHT_V1
      test -f /opt/codiquary/lib/libclang.so
      test "$(readlink -f /opt/codiquary/lib/libclang.so)" = \
        /usr/lib/x86_64-linux-gnu/libclang-14.so.14.0.6
      test ! -w /repo
      test ! -w /inputs
      test ! -w /cargo
      test ! -w /execution-cargo-home-inventory.json
      test "$CQ_RUST_TOOLCHAIN_BIN" = \
        /usr/local/rustup/toolchains/1.98.1-x86_64-unknown-linux-gnu/bin
      for tool in cargo rustc rustdoc rustfmt cargo-fmt clippy-driver cargo-clippy; do
        test -f "$CQ_RUST_TOOLCHAIN_BIN/$tool"
        test -x "$CQ_RUST_TOOLCHAIN_BIN/$tool"
      done
      printf "%s  %s\n" \
        1d69534c34fc55d999c7cac409c5b44d2f036e6612e37a3021eb6f3d01cf57f4 \
        /repo/proofs/exact-target-tough/Cargo.lock \
        12c719f55434cde51602fc3c9da6145d509fac93831c80d529f5f8b33930b5fb \
        /repo/proofs/exact-target-tough/locks/tough-workspace.Cargo.lock \
        | sha256sum -c -
      if touch /codiquary-root-write-probe 2>/dev/null; then
        rm -f /codiquary-root-write-probe
        echo "executor root is writable" >&2
        exit 1
      fi
      for directory in /phase/target /phase/tmp /phase/home /phase/output \
        /phase/fixtures /repo/proofs/exact-target-tough/.work; do
        touch "$directory/.codiquary-write-probe"
        rm "$directory/.codiquary-write-probe"
      done
      printf "executor_manifest_sha256=%s\n" "$CQ_EXECUTOR_MANIFEST_SHA256"
      printf "executor_config_sha256=%s\n" "$CQ_EXECUTOR_CONFIG_SHA256"
      printf "executor_image_id=%s\n" "$CQ_EXECUTOR_IMAGE_ID"
      printf "libclang_path=%s\n" "$LIBCLANG_PATH"
      readlink -f "$LIBCLANG_PATH"
      sha256sum \
        /opt/codiquary/executor/inputs.sha256 \
        /opt/codiquary/executor/added-debian-packages.tsv \
        /opt/codiquary/executor/added-rust-components.json \
        /opt/codiquary/executor/packages.tsv \
        /opt/codiquary/executor/rust-components.txt \
        /opt/codiquary/executor/libclang-target.txt \
        /opt/codiquary/executor/build-network-denial.txt \
        /execution-cargo-home-inventory.json \
        "$CQ_RUST_TOOLCHAIN_BIN/cargo" "$CQ_RUST_TOOLCHAIN_BIN/rustc" \
        "$CQ_RUST_TOOLCHAIN_BIN/rustdoc" "$CQ_RUST_TOOLCHAIN_BIN/rustfmt" \
        "$CQ_RUST_TOOLCHAIN_BIN/cargo-fmt" \
        "$CQ_RUST_TOOLCHAIN_BIN/clippy-driver" \
        "$CQ_RUST_TOOLCHAIN_BIN/cargo-clippy" /usr/bin/gcc /usr/bin/g++ \
        /usr/bin/ld /usr/bin/ar /usr/bin/ranlib /usr/bin/cmake \
        /usr/bin/make /usr/bin/perl /usr/bin/python3 /bin/bash \
        /usr/bin/git /usr/bin/patch /usr/bin/tar /usr/bin/sha256sum \
        /usr/bin/sha512sum /usr/sbin/ip "$LIBCLANG_PATH"
      cat /opt/codiquary/executor/build-network-denial.txt
      cat /opt/codiquary/executor/packages.tsv
      cat /opt/codiquary/executor/rust-components.txt
      cat /opt/codiquary/executor/libclang-target.txt
      "$CARGO" -Vv
      "$RUSTC" -Vv
      "$RUSTDOC" --version
      "$RUSTFMT" --version
      "$CQ_RUST_TOOLCHAIN_BIN/cargo-fmt" --version
      "$CQ_RUST_TOOLCHAIN_BIN/clippy-driver" --version
      "$CQ_RUST_TOOLCHAIN_BIN/cargo-clippy" --version
      gcc --version
      g++ --version
      cmake --version
      make --version
      perl -V:version
      python3 --version
      ip -o link show
      test -z "$(ip -o -4 route show)"
      test -z "$(ip -o -6 route show)"
      grep -E "^(Cap(Inh|Prm|Eff|Bnd|Amb)|NoNewPrivs):" /proc/self/status
      test "$(grep -Ec "^Cap(Inh|Prm|Eff|Bnd|Amb):[[:space:]]+0+$" \
        /proc/self/status)" -eq 5
      grep -Eq "^NoNewPrivs:[[:space:]]+1$" /proc/self/status
      readlink /proc/self/ns/net
      mount_provenance_path=/phase/tmp/.codiquary-pre-start-mount-provenance.json
      exec 3<&0
      python3 - "$mount_provenance_path" <<"PY_MOUNT_INPUT"
import os
from pathlib import Path
import sys

output_path = Path(sys.argv[1])
record = bytearray()
while True:
    byte = os.read(3, 1)
    if not byte:
        raise SystemExit("controller stdin ended before mount provenance")
    if byte == b"\n":
        break
    record.extend(byte)
    if len(record) > 1024 * 1024:
        raise SystemExit("mount provenance exceeds 1 MiB")
if not record:
    raise SystemExit("mount provenance is empty")
with output_path.open("xb") as output:
    output.write(record)
PY_MOUNT_INPUT
      exec 3<&-
      CQ_MOUNT_PROVENANCE_PATH=$mount_provenance_path \
        python3 - <<"PY_MOUNTS"
import hashlib
import json
import os
from pathlib import Path
import re

declared_host_binds = {
    "/cargo": "ro",
    "/execution-cargo-home-inventory.json": "ro",
    "/inputs": "ro",
    "/phase/fixtures": "rw",
    "/phase/home": "rw",
    "/phase/output": "rw",
    "/phase/target": "rw",
    "/phase/tmp": "rw",
    "/repo": "ro",
    "/repo/proofs/exact-target-tough/.work": "rw",
}
runtime_state_destinations = {
    "/dev/shm",
    "/etc/hostname",
    "/etc/resolv.conf",
    "/run/.containerenv",
}
runtime_pseudo_destinations = {
    "/dev",
    "/dev/mqueue",
    "/dev/pts",
    "/dev/shm",
    "/proc",
    "/sys",
    "/sys/fs/cgroup",
}
runtime_device_destinations = {
    "/dev/full",
    "/dev/null",
    "/dev/random",
    "/dev/tty",
    "/dev/urandom",
    "/dev/zero",
}
source_controls = {
    "declared-host-bind",
    "oci-crun-host-device",
    "oci-masked-container-dev-null",
    "oci-masked-crun-empty-directory",
    "oci-masked-host-dev-null",
    "oci-masked-tmpfs",
    "oci-pseudo-filesystem",
    "oci-readonly-remount",
    "reviewed-image-root",
    "task-state-bind",
}

provenance_text = Path(os.environ["CQ_MOUNT_PROVENANCE_PATH"]).read_text(
    encoding="utf-8"
)
provenance = json.loads(provenance_text)
canonical_provenance = json.dumps(
    provenance, separators=(",", ":"), sort_keys=True
)
if canonical_provenance != provenance_text:
    raise SystemExit("mount provenance is not canonical single-line JSON")
if (
    not isinstance(provenance, dict)
    or set(provenance)
    != {"container_id", "masked_paths", "mounts", "readonly_paths", "schema"}
    or provenance.get("schema")
    != "io.nisavid.codiquary.pre-start-mount-provenance/v1"
    or not isinstance(provenance.get("container_id"), str)
    or not re.fullmatch(r"[0-9a-f]{64}", provenance["container_id"])
    or not isinstance(provenance.get("masked_paths"), list)
    or not isinstance(provenance.get("readonly_paths"), list)
    or not isinstance(provenance.get("mounts"), list)
):
    raise SystemExit("mount provenance has the wrong schema")

def exact_paths(values: list[object], label: str) -> set[str]:
    if any(
        not isinstance(value, str)
        or not value.startswith("/")
        or "\n" in value
        for value in values
    ):
        raise SystemExit(f"invalid {label} destination")
    if len(set(values)) != len(values):
        raise SystemExit(f"duplicate {label} destination")
    return set(values)

masked_paths = exact_paths(provenance["masked_paths"], "masked path")
readonly_paths = exact_paths(provenance["readonly_paths"], "readonly path")
expected_mounts = {}
required_record_keys = {
    "access",
    "destination",
    "filesystem",
    "major_minor",
    "root",
    "source",
    "source_control",
}
for record in provenance["mounts"]:
    if not isinstance(record, dict) or set(record) != required_record_keys:
        raise SystemExit("malformed mount provenance record")
    if any(
        not isinstance(record[key], str) or "\n" in record[key]
        for key in required_record_keys
    ):
        raise SystemExit("non-string or newline-bearing mount provenance field")
    destination = record["destination"]
    if (
        not destination.startswith("/")
        or destination in expected_mounts
        or record["access"] not in {"ro", "rw"}
        or not record["filesystem"]
        or not re.fullmatch(r"[0-9]+:[0-9]+", record["major_minor"])
        or not record["root"].startswith("/")
        or not record["source"]
        or record["source_control"] not in source_controls
    ):
        raise SystemExit(f"invalid mount provenance record: {destination!r}")
    expected_mounts[destination] = record

if expected_mounts.get("/", {}).get("source_control") != "reviewed-image-root":
    raise SystemExit("mount provenance has no reviewed image root")
observed_declared_policy = {
    destination: record["access"]
    for destination, record in expected_mounts.items()
    if record["source_control"] == "declared-host-bind"
}
if observed_declared_policy != declared_host_binds:
    raise SystemExit("declared bind provenance policy changed")
for destination, record in expected_mounts.items():
    control = record["source_control"]
    if control == "reviewed-image-root" and destination != "/":
        raise SystemExit(f"unexpected image-root destination: {destination!r}")
    if control == "task-state-bind" and destination not in runtime_state_destinations:
        raise SystemExit(f"unallowlisted task-state bind: {destination!r}")
    if control == "oci-pseudo-filesystem" and destination not in runtime_pseudo_destinations:
        raise SystemExit(f"unallowlisted OCI pseudo-filesystem: {destination!r}")
    if control == "oci-crun-host-device" and destination not in runtime_device_destinations:
        raise SystemExit(f"unallowlisted crun device mount: {destination!r}")
    if control.startswith("oci-masked-") and destination not in masked_paths:
        raise SystemExit(f"unadmitted masked destination: {destination!r}")
    if control == "oci-readonly-remount" and destination not in readonly_paths:
        raise SystemExit(f"unadmitted read-only destination: {destination!r}")

def decode_mountinfo(value: str) -> str:
    return re.sub(
        r"\\([0-7]{3})",
        lambda match: chr(int(match.group(1), 8)),
        value,
    )

observed_mounts = {}
for line in Path("/proc/self/mountinfo").read_text().splitlines():
    fields = line.split()
    try:
        separator = fields.index("-")
    except ValueError as error:
        raise SystemExit("mountinfo record has no separator") from error
    if separator < 6 or len(fields) <= separator + 3:
        raise SystemExit("mountinfo record is incomplete")
    root = decode_mountinfo(fields[3])
    mountpoint = decode_mountinfo(fields[4])
    filesystem = fields[separator + 1]
    source = decode_mountinfo(fields[separator + 2])
    mount_options = set(fields[5].split(","))
    if "ro" in mount_options:
        access = "ro"
    elif "rw" in mount_options:
        access = "rw"
    else:
        raise SystemExit(f"mount has no access mode: {mountpoint!r}")
    if mountpoint in observed_mounts:
        raise SystemExit(f"stacked mount rejected: {mountpoint!r}")
    observed_mounts[mountpoint] = {
        "access": access,
        "destination": mountpoint,
        "filesystem": filesystem,
        "major_minor": fields[2],
        "root": root,
        "source": source,
    }

if set(observed_mounts) != set(expected_mounts):
    raise SystemExit(
        "post-start mount set differs from pre-start provenance: "
        f"missing={sorted(set(expected_mounts) - set(observed_mounts))!r} "
        f"extra={sorted(set(observed_mounts) - set(expected_mounts))!r}"
    )
for destination, observed in observed_mounts.items():
    expected = {
        key: value
        for key, value in expected_mounts[destination].items()
        if key != "source_control"
    }
    if observed != expected:
        raise SystemExit(
            f"post-start mount source provenance changed: {destination!r}"
        )

provenance_sha256 = hashlib.sha256(
    (canonical_provenance + "\n").encode("utf-8")
).hexdigest()
print(
    json.dumps(
        {
            "declared_host_binds": dict(sorted(declared_host_binds.items())),
            "image_root": {
                "access": expected_mounts["/"]["access"],
                "filesystem": expected_mounts["/"]["filesystem"],
                "source_control": expected_mounts["/"]["source_control"],
            },
            "mount_provenance_sha256": provenance_sha256,
            "runtime_mounts": {
                destination: {
                    "access": record["access"],
                    "filesystem": record["filesystem"],
                    "source_control": record["source_control"],
                }
                for destination, record in sorted(expected_mounts.items())
                if destination not in declared_host_binds and destination != "/"
            },
            "schema": "io.nisavid.codiquary.container-mount-projection/v4",
        },
        sort_keys=True,
    )
)
PY_MOUNTS
      rm -f "$mount_provenance_path"
      python3 - <<"PY_NETWORK"
import errno
import socket

probes = (
    (socket.AF_INET, ("192.0.2.1", 9), "ipv4"),
    (socket.AF_INET6, ("2001:db8::1", 9, 0, 0), "ipv6"),
)
for family, address, label in probes:
    sock = socket.socket(family, socket.SOCK_STREAM)
    sock.settimeout(1)
    try:
        sock.connect(address)
    except OSError as error:
        if error.errno != errno.ENETUNREACH:
            raise SystemExit(f"{label} denial was not ENETUNREACH: {error!r}")
        print(f"{label}_test_net=ENETUNREACH")
    else:
        raise SystemExit(f"{label} TEST-NET connection unexpectedly succeeded")
    finally:
        sock.close()
PY_NETWORK
      printf "%s\n" CODIQUARY_EXECUTOR_PREFLIGHT_COMPLETE
      exec "$@" > /phase/output/stdout.txt 2> /phase/output/stderr.txt
    ' cq-phase "$@"
  )

  local create_stdout="$phase_boundary/container-id.txt"
  local create_stderr="$phase_boundary/container-create-stderr.txt"
  local create_call_status
  if cq_oci_controller "$CQ_EXECUTION_STATE" "$CQ_BOUNDARY_EVIDENCE" \
    "$CQ_PHASE-create" "${create_arguments[@]}" \
    > "$create_stdout" 2> "$create_stderr"; then
    create_call_status=0
  else
    create_call_status=$?
  fi
  local create_controller_status
  create_controller_status=$(cq_controller_receipt_status \
    "$CQ_BOUNDARY_EVIDENCE/controller/$CQ_PHASE-create") || return 125
  test "$create_call_status" -eq "$create_controller_status" || return 125
  test "$create_controller_status" -eq 0 || return 125
  local -a container_ids=()
  mapfile -t container_ids < "$create_stdout" || return 125
  test "${#container_ids[@]}" -eq 1 || return 125
  local container_id=${container_ids[0]}
  [[ "$container_id" =~ ^[0-9a-f]{64}$ ]] || return 125

  local created_inspect="$phase_boundary/container-created-inspect.json"
  local created_inspect_stderr="$phase_boundary/container-created-inspect-stderr.txt"
  local created_inspect_call_status
  if cq_oci_controller "$CQ_EXECUTION_STATE" "$CQ_BOUNDARY_EVIDENCE" \
    "$CQ_PHASE-created-inspect" container inspect "$container_id" \
    > "$created_inspect" 2> "$created_inspect_stderr"; then
    created_inspect_call_status=0
  else
    created_inspect_call_status=$?
  fi
  local created_inspect_controller_status
  created_inspect_controller_status=$(cq_controller_receipt_status \
    "$CQ_BOUNDARY_EVIDENCE/controller/$CQ_PHASE-created-inspect") \
    || return 125
  test "$created_inspect_call_status" -eq \
    "$created_inspect_controller_status" || return 125
  test "$created_inspect_controller_status" -eq 0 || return 125

  /usr/bin/python3 - \
    "$created_inspect" "$phase_boundary/declared-mounts.json" \
    "$container_id" "$container_name" "$CQ_EXECUTOR_CONFIG_SHA256" \
    "$CQ_REPO" /repo ro \
    "$CQ_INPUTS" /inputs ro \
    "$CQ_CARGO_HOME" /cargo ro \
    "$CQ_CARGO_INVENTORY" /execution-cargo-home-inventory.json ro \
    "$phase_target" /phase/target rw \
    "$phase_tmp" /phase/tmp rw \
    "$phase_home" /phase/home rw \
    "$phase_output" /phase/output rw \
    "$phase_fixtures" /phase/fixtures rw \
    "$phase_work" /repo/proofs/exact-target-tough/.work rw \
    <<'PY_DECLARED_MOUNTS' || return 125
import json
from pathlib import Path
import sys

inspection_path = Path(sys.argv[1])
output_path = Path(sys.argv[2])
container_id = sys.argv[3]
container_name = sys.argv[4]
config_sha256 = sys.argv[5]
mount_arguments = sys.argv[6:]
if len(mount_arguments) % 3:
    raise SystemExit("mount expectation arguments are incomplete")
expected = {}
for index in range(0, len(mount_arguments), 3):
    source, destination, access = mount_arguments[index:index + 3]
    if destination in expected or access not in {"ro", "rw"}:
        raise SystemExit("invalid duplicate mount expectation")
    expected[destination] = {
        "access": access,
        "source": Path(source).resolve(strict=True),
    }

inspection = json.loads(inspection_path.read_text(encoding="utf-8"))
if len(inspection) != 1:
    raise SystemExit("expected one created-container inspection")
container = inspection[0]
if container["Id"] != container_id:
    raise SystemExit("created container ID changed")
if container["Name"].removeprefix("/") != container_name:
    raise SystemExit("created container name changed")
if container["Image"].removeprefix("sha256:") != config_sha256:
    raise SystemExit("created container image changed")
if container["HostConfig"].get("AutoRemove") is not False:
    raise SystemExit("created container must be retained")

observed = {}
for mount in container["Mounts"]:
    destination = mount.get("Destination")
    if destination not in expected or destination in observed:
        raise SystemExit(f"undeclared or duplicate bind mount: {destination!r}")
    if mount.get("Type") != "bind" or type(mount.get("RW")) is not bool:
        raise SystemExit(f"mount is not a classified bind: {destination!r}")
    access = "rw" if mount["RW"] else "ro"
    source = Path(mount["Source"]).resolve(strict=True)
    if access != expected[destination]["access"]:
        raise SystemExit(f"mount access changed: {destination!r}")
    if source != expected[destination]["source"]:
        raise SystemExit(f"mount source changed: {destination!r}")
    observed[destination] = access
if set(observed) != set(expected):
    raise SystemExit(
        "declared bind-mount set changed: "
        f"missing={sorted(set(expected) - set(observed))!r}"
    )
output_path.write_text(
    json.dumps(
        {
            "mounts": dict(sorted(observed.items())),
            "schema": "io.nisavid.codiquary.declared-container-mounts/v1",
        },
        indent=2,
        sort_keys=True,
    )
    + "\n",
    encoding="utf-8",
)
PY_DECLARED_MOUNTS

  local init_stdout="$phase_boundary/container-init.txt"
  local init_stderr="$phase_boundary/container-init-stderr.txt"
  local init_call_status
  if cq_oci_controller "$CQ_EXECUTION_STATE" "$CQ_BOUNDARY_EVIDENCE" \
    "$CQ_PHASE-init" container init "$container_id" \
    > "$init_stdout" 2> "$init_stderr"; then
    init_call_status=0
  else
    init_call_status=$?
  fi
  local init_controller_status
  init_controller_status=$(cq_controller_receipt_status \
    "$CQ_BOUNDARY_EVIDENCE/controller/$CQ_PHASE-init") || return 125
  test "$init_call_status" -eq "$init_controller_status" || return 125
  test "$init_controller_status" -eq 0 || return 125

  local initialized_inspect="$phase_boundary/container-initialized-inspect.json"
  local initialized_inspect_stderr="$phase_boundary/container-initialized-inspect-stderr.txt"
  local initialized_inspect_call_status
  if cq_oci_controller "$CQ_EXECUTION_STATE" "$CQ_BOUNDARY_EVIDENCE" \
    "$CQ_PHASE-initialized-inspect" container inspect "$container_id" \
    > "$initialized_inspect" 2> "$initialized_inspect_stderr"; then
    initialized_inspect_call_status=0
  else
    initialized_inspect_call_status=$?
  fi
  local initialized_inspect_controller_status
  initialized_inspect_controller_status=$(cq_controller_receipt_status \
    "$CQ_BOUNDARY_EVIDENCE/controller/$CQ_PHASE-initialized-inspect") \
    || return 125
  test "$initialized_inspect_call_status" -eq \
    "$initialized_inspect_controller_status" || return 125
  test "$initialized_inspect_controller_status" -eq 0 || return 125

  /usr/bin/python3 - \
    "$initialized_inspect" "$phase_boundary/effective-oci-config.json" \
    "$phase_boundary/effective-mounts.json" \
    "$phase_boundary/pre-start-mount-provenance.json" \
    "$container_id" "$container_name" "$CQ_EXECUTOR_CONFIG_SHA256" \
    "$CQ_EXECUTION_STATE" \
    "$CQ_REPO" /repo ro \
    "$CQ_INPUTS" /inputs ro \
    "$CQ_CARGO_HOME" /cargo ro \
    "$CQ_CARGO_INVENTORY" /execution-cargo-home-inventory.json ro \
    "$phase_target" /phase/target rw \
    "$phase_tmp" /phase/tmp rw \
    "$phase_home" /phase/home rw \
    "$phase_output" /phase/output rw \
    "$phase_fixtures" /phase/fixtures rw \
    "$phase_work" /repo/proofs/exact-target-tough/.work rw \
    <<'PY_EFFECTIVE_MOUNTS' || return 125
import glob
import hashlib
import json
import os
from pathlib import Path, PurePosixPath
import re
import stat
import sys

inspection_path = Path(sys.argv[1])
spec_copy_path = Path(sys.argv[2])
output_path = Path(sys.argv[3])
provenance_path = Path(sys.argv[4])
container_id = sys.argv[5]
container_name = sys.argv[6]
config_sha256 = sys.argv[7]
execution_state = Path(sys.argv[8]).resolve(strict=True)
mount_arguments = sys.argv[9:]
if len(mount_arguments) % 3:
    raise SystemExit("mount expectation arguments are incomplete")

expected_host_binds = {}
for index in range(0, len(mount_arguments), 3):
    source, destination, access = mount_arguments[index:index + 3]
    if destination in expected_host_binds or access not in {"ro", "rw"}:
        raise SystemExit("invalid duplicate mount expectation")
    expected_host_binds[destination] = {
        "access": access,
        "source": Path(source).resolve(strict=True),
    }

inspection = json.loads(inspection_path.read_text(encoding="utf-8"))
if len(inspection) != 1:
    raise SystemExit("expected one initialized-container inspection")
container = inspection[0]
if container["Id"] != container_id:
    raise SystemExit("initialized container ID changed")
if container["Name"].removeprefix("/") != container_name:
    raise SystemExit("initialized container name changed")
if container["Image"].removeprefix("sha256:") != config_sha256:
    raise SystemExit("initialized container image changed")
if container["HostConfig"].get("AutoRemove") is not False:
    raise SystemExit("initialized container must be retained")
state = container["State"]
if (
    state.get("Status") != "created"
    or state.get("Running") is not False
    or state.get("Paused") is not False
    or state.get("Restarting") is not False
    or state.get("OOMKilled") is not False
    or state.get("Dead") is not False
    or state.get("Error") != ""
):
    raise SystemExit("container did not reach a clean initialized state")
container_pid = state.get("Pid")
if type(container_pid) is not int or container_pid <= 0:
    raise SystemExit("initialized container has no positive process PID")

raw_oci_config_path = container.get("OCIConfigPath")
if (
    not isinstance(raw_oci_config_path, str)
    or not raw_oci_config_path.startswith("/")
    or "\n" in raw_oci_config_path
):
    raise SystemExit("initialized inspection has no absolute OCIConfigPath")
oci_config_path = Path(raw_oci_config_path)
resolved_oci_config_path = oci_config_path.resolve(strict=True)
if oci_config_path != resolved_oci_config_path:
    raise SystemExit("OCIConfigPath traverses a symlink")
if execution_state not in resolved_oci_config_path.parents:
    raise SystemExit("OCIConfigPath escaped task-owned Podman state")
oci_mode = resolved_oci_config_path.lstat().st_mode
if not stat.S_ISREG(oci_mode) or resolved_oci_config_path.is_symlink():
    raise SystemExit("OCIConfigPath is not a regular file")

spec_bytes = resolved_oci_config_path.read_bytes()
spec = json.loads(spec_bytes)
if not isinstance(spec, dict) or spec.get("root", {}).get("readonly") is not True:
    raise SystemExit("effective OCI specification has a writable image root")
mounts = spec.get("mounts")
if not isinstance(mounts, list):
    raise SystemExit("effective OCI specification has no mount list")

def container_path(value: object) -> str:
    if not isinstance(value, str) or not value.startswith("/") or "\n" in value:
        raise SystemExit(f"invalid OCI mount destination: {value!r}")
    normalized = PurePosixPath(value).as_posix()
    if normalized != value or ".." in PurePosixPath(value).parts:
        raise SystemExit(f"non-canonical OCI mount destination: {value!r}")
    return value

def access_from(options: set[str]) -> str:
    if "ro" in options and "rw" in options:
        raise SystemExit("OCI mount declares both ro and rw")
    return "ro" if "ro" in options else "rw"

runtime_state_policy = {
    "/dev/shm": ("directory", {"rw"}),
    "/etc/hostname": ("file", {"ro", "rw"}),
    "/etc/resolv.conf": ("file", {"ro", "rw"}),
    "/run/.containerenv": ("file", {"ro", "rw"}),
}
runtime_pseudo_policy = {
    "/dev": {("tmpfs", "tmpfs", "rw")},
    "/dev/mqueue": {("mqueue", "mqueue", "rw")},
    "/dev/pts": {("devpts", "devpts", "rw")},
    "/dev/shm": {
        ("tmpfs", "shm", "rw"),
        ("tmpfs", "tmpfs", "rw"),
    },
    "/proc": {("proc", "proc", "rw")},
    "/sys": {("sysfs", "sysfs", "ro")},
    "/sys/fs/cgroup": {
        ("cgroup", "cgroup", "ro"),
        ("cgroup", "cgroup2", "ro"),
        ("cgroup2", "cgroup", "ro"),
        ("cgroup2", "cgroup2", "ro"),
    },
}
required_runtime_destinations = {
    "/dev",
    "/dev/mqueue",
    "/dev/pts",
    "/dev/shm",
    "/proc",
    "/sys",
    "/sys/fs/cgroup",
}
runtime_source_roots = [
    (execution_state / name).resolve(strict=True) for name in ("root", "runroot")
]
crun_empty_directory = (
    execution_state / "runtime" / "crun" / ".empty-directory"
)

root_config = spec.get("root")
raw_root_path = root_config.get("path") if isinstance(root_config, dict) else None
if (
    not isinstance(raw_root_path, str)
    or not raw_root_path
    or "\n" in raw_root_path
    or ".." in PurePosixPath(raw_root_path).parts
):
    raise SystemExit("effective OCI specification has no safe root path")
root_candidate = Path(raw_root_path)
if not root_candidate.is_absolute():
    root_candidate = resolved_oci_config_path.parent / root_candidate
root_source = root_candidate.resolve(strict=True)
root_mode = root_source.lstat().st_mode
if not stat.S_ISDIR(root_mode) or root_source.is_symlink():
    raise SystemExit("effective OCI root path is not a regular directory")
if not any(
    source_root == root_source or source_root in root_source.parents
    for source_root in runtime_source_roots
):
    raise SystemExit("effective OCI root escaped task-owned Podman state")

linux_config = spec.get("linux")
if not isinstance(linux_config, dict):
    raise SystemExit("effective OCI specification has no Linux policy")

def security_paths(field: str) -> set[str]:
    raw_paths = linux_config.get(field)
    if not isinstance(raw_paths, list):
        raise SystemExit(f"effective OCI specification has no {field}")
    paths = [container_path(value) for value in raw_paths]
    if len(set(paths)) != len(paths):
        raise SystemExit(f"effective OCI specification repeats {field}")
    for path in paths:
        if not any(path.startswith(root + "/") for root in ("/proc", "/sys")):
            raise SystemExit(f"{field} escaped the runtime security trees: {path!r}")
    return set(paths)

expected_masked_paths = {
    "/proc/acpi",
    "/proc/interrupts",
    "/proc/kcore",
    "/proc/keys",
    "/proc/latency_stats",
    "/proc/sched_debug",
    "/proc/scsi",
    "/proc/timer_list",
    "/proc/timer_stats",
    "/sys/devices/virtual/powercap",
    "/sys/firmware",
    "/sys/fs/selinux",
}
thermal_throttle_paths = {
    container_path(path)
    for path in glob.glob(
        "/sys/devices/system/cpu/cpu*/thermal_throttle"
    )
}
expected_masked_paths.update(thermal_throttle_paths)
expected_readonly_paths = {
    "/proc/asound",
    "/proc/bus",
    "/proc/fs",
    "/proc/irq",
    "/proc/sys",
    "/proc/sysrq-trigger",
}

masked_paths = security_paths("maskedPaths")
readonly_paths = security_paths("readonlyPaths")
if masked_paths != expected_masked_paths:
    raise SystemExit(
        "effective OCI maskedPaths differ from the tagged Podman defaults: "
        f"missing={sorted(expected_masked_paths - masked_paths)!r} "
        f"extra={sorted(masked_paths - expected_masked_paths)!r}"
    )
if readonly_paths != expected_readonly_paths:
    raise SystemExit(
        "effective OCI readonlyPaths differ from the tagged Podman defaults: "
        f"missing={sorted(expected_readonly_paths - readonly_paths)!r} "
        f"extra={sorted(readonly_paths - expected_readonly_paths)!r}"
    )
if masked_paths & readonly_paths:
    raise SystemExit("effective OCI mask and read-only paths overlap")

declared_host_binds = []
runtime_state_binds = []
runtime_pseudo_filesystems = []
seen_destinations = set()
for mount in mounts:
    if not isinstance(mount, dict):
        raise SystemExit("effective OCI mount is not an object")
    destination = container_path(mount.get("destination"))
    if destination in seen_destinations:
        raise SystemExit(f"duplicate effective OCI mount: {destination!r}")
    seen_destinations.add(destination)
    mount_type = mount.get("type")
    source = mount.get("source")
    raw_options = mount.get("options", [])
    if (
        not isinstance(mount_type, str)
        or not isinstance(source, str)
        or "\n" in source
        or not isinstance(raw_options, list)
        or any(not isinstance(option, str) for option in raw_options)
        or len(set(raw_options)) != len(raw_options)
    ):
        raise SystemExit(f"malformed effective OCI mount: {destination!r}")
    options = set(raw_options)
    access = access_from(options)
    is_bind = mount_type == "bind" or bool({"bind", "rbind"} & options)
    record = {
        "access": access,
        "destination": destination,
        "options": sorted(options),
        "source": source,
        "type": mount_type,
    }

    if destination in expected_host_binds:
        if not is_bind or not source.startswith("/"):
            raise SystemExit(f"declared host mount is not a bind: {destination!r}")
        resolved_source = Path(source).resolve(strict=True)
        if resolved_source != expected_host_binds[destination]["source"]:
            raise SystemExit(f"declared host mount source changed: {destination!r}")
        if access != expected_host_binds[destination]["access"]:
            raise SystemExit(f"declared host mount access changed: {destination!r}")
        record["source"] = str(resolved_source)
        declared_host_binds.append(record)
        continue

    if is_bind:
        if destination not in runtime_state_policy or not source.startswith("/"):
            raise SystemExit(f"unallowlisted effective host mount: {destination!r}")
        resolved_source = Path(source).resolve(strict=True)
        if not any(
            root == resolved_source or root in resolved_source.parents
            for root in runtime_source_roots
        ):
            raise SystemExit(f"runtime bind escaped task-owned state: {destination!r}")
        expected_kind, allowed_access = runtime_state_policy[destination]
        source_mode = resolved_source.lstat().st_mode
        observed_kind = (
            "directory" if stat.S_ISDIR(source_mode)
            else "file" if stat.S_ISREG(source_mode)
            else "other"
        )
        if observed_kind != expected_kind or access not in allowed_access:
            raise SystemExit(f"runtime bind shape changed: {destination!r}")
        record["source"] = str(resolved_source)
        runtime_state_binds.append(record)
        continue

    allowed_pseudo = runtime_pseudo_policy.get(destination, set())
    if (mount_type, source, access) not in allowed_pseudo:
        raise SystemExit(f"unallowlisted runtime pseudo-filesystem: {destination!r}")
    runtime_pseudo_filesystems.append(record)

observed_host_destinations = {
    record["destination"] for record in declared_host_binds
}
if observed_host_destinations != set(expected_host_binds):
    raise SystemExit(
        "effective host-bind set changed: "
        f"missing={sorted(set(expected_host_binds) - observed_host_destinations)!r}"
    )
observed_runtime_destinations = {
    record["destination"]
    for record in [*runtime_state_binds, *runtime_pseudo_filesystems]
}
if not required_runtime_destinations <= observed_runtime_destinations:
    raise SystemExit(
        "required runtime pseudo-filesystem is absent: "
        f"{sorted(required_runtime_destinations - observed_runtime_destinations)!r}"
    )

def decode_mountinfo(value: str) -> str:
    decoded = re.sub(
        r"\\([0-7]{3})",
        lambda match: chr(int(match.group(1), 8)),
        value,
    )
    if "\n" in decoded:
        raise SystemExit("newline in mountinfo field")
    return decoded

def parse_mountinfo(text: str, label: str) -> dict[str, dict[str, str]]:
    records = {}
    for line in text.splitlines():
        fields = line.split()
        try:
            separator = fields.index("-")
        except ValueError as error:
            raise SystemExit(f"{label} mountinfo record has no separator") from error
        if separator < 6 or len(fields) <= separator + 3:
            raise SystemExit(f"{label} mountinfo record is incomplete")
        root = decode_mountinfo(fields[3])
        destination = decode_mountinfo(fields[4])
        source = decode_mountinfo(fields[separator + 2])
        if (
            not root.startswith("/")
            or not destination.startswith("/")
            or not source
            or destination in records
        ):
            raise SystemExit(f"{label} has an invalid or stacked mount: {destination!r}")
        mount_options = set(fields[5].split(","))
        if "ro" in mount_options:
            access = "ro"
        elif "rw" in mount_options:
            access = "rw"
        else:
            raise SystemExit(f"{label} mount has no access mode: {destination!r}")
        records[destination] = {
            "access": access,
            "destination": destination,
            "filesystem": fields[separator + 1],
            "major_minor": fields[2],
            "root": root,
            "source": source,
        }
    return records

process_root = Path("/proc") / str(container_pid)

def process_identity() -> tuple[int, str]:
    process_stat = process_root.stat()
    raw_stat = (process_root / "stat").read_text(encoding="ascii")
    close_paren = raw_stat.rfind(")")
    remaining = raw_stat[close_paren + 2:].split() if close_paren >= 0 else []
    if len(remaining) <= 19:
        raise SystemExit("initialized process stat is incomplete")
    return process_stat.st_ino, remaining[19]

process_identity_before = process_identity()
target_mountinfo = (process_root / "mountinfo").read_text(encoding="utf-8")
process_identity_after = process_identity()
if process_identity_after != process_identity_before:
    raise SystemExit("initialized container process changed during mount observation")

target_mounts = parse_mountinfo(target_mountinfo, "initialized container")
host_mounts = parse_mountinfo(
    Path("/proc/self/mountinfo").read_text(encoding="utf-8"),
    "controller host",
)

def projected_source(source: Path) -> dict[str, str]:
    candidates = [
        record
        for destination, record in host_mounts.items()
        if source == Path(destination) or Path(destination) in source.parents
    ]
    if not candidates:
        raise SystemExit(f"no host mount owns admitted source: {source!s}")
    deepest = max(len(PurePosixPath(record["destination"]).parts) for record in candidates)
    candidates = [
        record
        for record in candidates
        if len(PurePosixPath(record["destination"]).parts) == deepest
    ]
    if len(candidates) != 1:
        raise SystemExit(f"stacked host source mount rejected: {source!s}")
    host_mount = candidates[0]
    relative = source.relative_to(Path(host_mount["destination"]))
    projected_root = PurePosixPath(host_mount["root"]).joinpath(
        *PurePosixPath(relative.as_posix()).parts
    ).as_posix()
    return {
        "filesystem": host_mount["filesystem"],
        "major_minor": host_mount["major_minor"],
        "root": projected_root,
        "source": host_mount["source"],
    }

remaining_mounts = dict(target_mounts)
provenance_records = {}

def admit_exact(
    destination: str,
    expected: dict[str, str],
    source_control: str,
) -> None:
    observed = remaining_mounts.pop(destination, None)
    if observed != expected:
        raise SystemExit(f"initialized mount provenance mismatch: {destination!r}")
    provenance_records[destination] = {
        **observed,
        "source_control": source_control,
    }

root_projection = projected_source(root_source)
admit_exact(
    "/",
    {"access": "ro", "destination": "/", **root_projection},
    "reviewed-image-root",
)

for record in declared_host_binds:
    destination = record["destination"]
    projection = projected_source(Path(record["source"]))
    admit_exact(
        destination,
        {"access": record["access"], "destination": destination, **projection},
        "declared-host-bind",
    )

for record in runtime_state_binds:
    destination = record["destination"]
    projection = projected_source(Path(record["source"]))
    admit_exact(
        destination,
        {"access": record["access"], "destination": destination, **projection},
        "task-state-bind",
    )

post_start_pseudo_policy = {
    "/dev": {("tmpfs", "tmpfs", "/", "rw")},
    "/dev/mqueue": {("mqueue", "mqueue", "/", "rw")},
    "/dev/pts": {("devpts", "devpts", "/", "rw")},
    "/dev/shm": {
        ("tmpfs", "shm", "/", "rw"),
        ("tmpfs", "tmpfs", "/", "rw"),
    },
    "/proc": {("proc", "proc", "/", "rw")},
    "/sys": {("sysfs", "sysfs", "/", "ro")},
    "/sys/fs/cgroup": {
        ("cgroup", "cgroup", "/", "ro"),
        ("cgroup", "cgroup2", "/", "ro"),
        ("cgroup2", "cgroup", "/", "ro"),
        ("cgroup2", "cgroup2", "/", "ro"),
    },
}
for record in runtime_pseudo_filesystems:
    destination = record["destination"]
    observed = remaining_mounts.get(destination)
    if observed is None:
        raise SystemExit(f"initialized pseudo-filesystem is absent: {destination!r}")
    observed_tuple = (
        observed["filesystem"],
        observed["source"],
        observed["root"],
        observed["access"],
    )
    if observed_tuple not in post_start_pseudo_policy[destination]:
        raise SystemExit(
            f"initialized pseudo-filesystem source changed: {destination!r}"
        )
    admit_exact(destination, observed, "oci-pseudo-filesystem")

host_device_policy = {
    "/dev/full": (1, 7),
    "/dev/null": (1, 3),
    "/dev/random": (1, 8),
    "/dev/tty": (5, 0),
    "/dev/urandom": (1, 9),
    "/dev/zero": (1, 5),
}
for destination, (expected_major, expected_minor) in host_device_policy.items():
    observed = remaining_mounts.get(destination)
    if observed is None:
        continue
    source_path = Path(destination)
    source_mode = source_path.lstat()
    if (
        source_path.resolve(strict=True) != source_path
        or not stat.S_ISCHR(source_mode.st_mode)
        or os.major(source_mode.st_rdev) != expected_major
        or os.minor(source_mode.st_rdev) != expected_minor
    ):
        raise SystemExit(f"host source is not the required device: {destination!r}")
    projection = projected_source(source_path)
    admit_exact(
        destination,
        {"access": "rw", "destination": destination, **projection},
        "oci-crun-host-device",
    )

def projected_child_root(
    parent: dict[str, str], parent_path: str, child_path: str
) -> str:
    relative = PurePosixPath(child_path).relative_to(PurePosixPath(parent_path))
    return PurePosixPath(parent["root"]).joinpath(*relative.parts).as_posix()

for destination in sorted(readonly_paths):
    observed = remaining_mounts.get(destination)
    if observed is None:
        continue
    parents = [
        (path, record)
        for path, record in provenance_records.items()
        if path != destination and PurePosixPath(path) in PurePosixPath(destination).parents
    ]
    if not parents:
        raise SystemExit(f"read-only mount has no admitted parent: {destination!r}")
    parent_path, parent = max(
        parents, key=lambda item: len(PurePosixPath(item[0]).parts)
    )
    expected = {
        "access": "ro",
        "destination": destination,
        "filesystem": parent["filesystem"],
        "major_minor": parent["major_minor"],
        "root": projected_child_root(parent, parent_path, destination),
        "source": parent["source"],
    }
    admit_exact(destination, expected, "oci-readonly-remount")

dev_mount = provenance_records.get("/dev")
if dev_mount is None:
    raise SystemExit("admitted /dev mount is absent")
dev_null_root = projected_child_root(dev_mount, "/dev", "/dev/null")
host_dev_null = Path("/dev/null")
host_dev_null_mode = host_dev_null.lstat()
if (
    host_dev_null.resolve(strict=True) != host_dev_null
    or not stat.S_ISCHR(host_dev_null_mode.st_mode)
    or os.major(host_dev_null_mode.st_rdev) != 1
    or os.minor(host_dev_null_mode.st_rdev) != 3
):
    raise SystemExit("host /dev/null is not the kernel null device")
host_dev_null_projection = projected_source(host_dev_null)
crun_empty_projection = None
try:
    crun_empty_mode = crun_empty_directory.lstat()
except FileNotFoundError:
    pass
else:
    if (
        crun_empty_directory.resolve(strict=True) != crun_empty_directory
        or not stat.S_ISDIR(crun_empty_mode.st_mode)
        or stat.S_IMODE(crun_empty_mode.st_mode) != 0o555
        or crun_empty_mode.st_uid != os.getuid()
        or crun_empty_mode.st_gid != os.getgid()
        or any(crun_empty_directory.iterdir())
    ):
        raise SystemExit("crun mask directory is not the closed empty directory")
    crun_empty_projection = projected_source(crun_empty_directory)
for destination in sorted(masked_paths):
    observed = remaining_mounts.get(destination)
    if observed is None:
        continue
    container_dev_null_mask = (
        observed["filesystem"] == dev_mount["filesystem"]
        and observed["major_minor"] == dev_mount["major_minor"]
        and observed["root"] == dev_null_root
        and observed["source"] == dev_mount["source"]
        and observed["access"] == "ro"
    )
    host_dev_null_mask = observed == {
        "access": "ro",
        "destination": destination,
        **host_dev_null_projection,
    }
    crun_empty_mask = (
        crun_empty_projection is not None
        and observed
        == {
            "access": "ro",
            "destination": destination,
            **crun_empty_projection,
        }
    )
    isolated_tmpfs_mask = (
        observed["filesystem"] == "tmpfs"
        and observed["major_minor"] != dev_mount["major_minor"]
        and observed["root"] == "/"
        and observed["source"] == "tmpfs"
        and observed["access"] == "ro"
    )
    if container_dev_null_mask:
        source_control = "oci-masked-container-dev-null"
    elif host_dev_null_mask:
        source_control = "oci-masked-host-dev-null"
    elif crun_empty_mask:
        source_control = "oci-masked-crun-empty-directory"
    elif isolated_tmpfs_mask:
        source_control = "oci-masked-tmpfs"
    else:
        raise SystemExit(f"masked mount source changed: {destination!r}")
    admit_exact(destination, observed, source_control)

if remaining_mounts:
    raise SystemExit(
        "initialized mount namespace has unadmitted destinations: "
        f"{sorted(remaining_mounts)!r}"
    )

spec_sha256 = hashlib.sha256(spec_bytes).hexdigest()
with spec_copy_path.open("xb") as output:
    output.write(spec_bytes)
output_path.write_text(
    json.dumps(
        {
            "container_id": container_id,
            "declared_host_binds": sorted(
                declared_host_binds, key=lambda record: record["destination"]
            ),
            "oci_config_path": str(resolved_oci_config_path),
            "oci_config_sha256": spec_sha256,
            "masked_paths": sorted(masked_paths),
            "readonly_paths": sorted(readonly_paths),
            "root_source": str(root_source),
            "runtime_pseudo_filesystems": sorted(
                runtime_pseudo_filesystems,
                key=lambda record: record["destination"],
            ),
            "runtime_state_binds": sorted(
                runtime_state_binds, key=lambda record: record["destination"]
            ),
            "schema": "io.nisavid.codiquary.effective-mount-admission/v2",
        },
        indent=2,
        sort_keys=True,
    )
    + "\n",
    encoding="utf-8",
)
provenance = {
    "container_id": container_id,
    "masked_paths": sorted(masked_paths),
    "mounts": sorted(
        provenance_records.values(), key=lambda record: record["destination"]
    ),
    "readonly_paths": sorted(readonly_paths),
    "schema": "io.nisavid.codiquary.pre-start-mount-provenance/v1",
}
with provenance_path.open("x", encoding="utf-8") as output:
    output.write(json.dumps(provenance, separators=(",", ":"), sort_keys=True) + "\n")
PY_EFFECTIVE_MOUNTS

  local mount_provenance="$phase_boundary/pre-start-mount-provenance.json"
  local mount_provenance_sha256
  mount_provenance_sha256=$(/usr/bin/sha256sum "$mount_provenance") \
    || return 125
  mount_provenance_sha256=${mount_provenance_sha256%% *}
  [[ "$mount_provenance_sha256" =~ ^[0-9a-f]{64}$ ]] || return 125
  local start_stdin="$phase_boundary/start-stdin.bin"
  /usr/bin/python3 - "$mount_provenance" "$phase_command_stdin" \
    "$start_stdin" <<'PY_START_STDIN' || return 125
from pathlib import Path
import sys

provenance_path = Path(sys.argv[1])
phase_input_path = Path(sys.argv[2])
output_path = Path(sys.argv[3])
provenance = provenance_path.read_bytes()
if not provenance.endswith(b"\n") or b"\n" in provenance[:-1]:
    raise SystemExit("mount provenance must be one newline-terminated record")
with output_path.open("xb") as output:
    output.write(provenance)
    output.write(phase_input_path.read_bytes())
PY_START_STDIN
  chmod 600 "$start_stdin" || return 125
  local start_stdin_sha256
  start_stdin_sha256=$(/usr/bin/sha256sum "$start_stdin") || return 125
  start_stdin_sha256=${start_stdin_sha256%% *}
  [[ "$start_stdin_sha256" =~ ^[0-9a-f]{64}$ ]] || return 125

  local preflight_receipt="$phase_boundary/preflight.txt"
  local preflight_stderr="$phase_boundary/preflight-stderr.txt"
  local start_call_status
  if cq_oci_controller "$CQ_EXECUTION_STATE" "$CQ_BOUNDARY_EVIDENCE" \
    "$CQ_PHASE-start" \
    --controller-stdin "$start_stdin" "$start_stdin_sha256" \
    start --attach --interactive "$container_id" \
    > "$preflight_receipt" 2> "$preflight_stderr"; then
    start_call_status=0
  else
    start_call_status=$?
  fi

  local start_controller_status
  start_controller_status=$(cq_controller_receipt_status \
    "$CQ_BOUNDARY_EVIDENCE/controller/$CQ_PHASE-start") || return 125
  test "$start_call_status" -eq "$start_controller_status" || return 125

  local exit_inspect="$phase_boundary/container-exit-inspect.json"
  local exit_inspect_stderr="$phase_boundary/container-exit-inspect-stderr.txt"
  local exit_inspect_call_status
  if cq_oci_controller "$CQ_EXECUTION_STATE" "$CQ_BOUNDARY_EVIDENCE" \
    "$CQ_PHASE-exit-inspect" container inspect "$container_id" \
    > "$exit_inspect" 2> "$exit_inspect_stderr"; then
    exit_inspect_call_status=0
  else
    exit_inspect_call_status=$?
  fi
  local exit_inspect_controller_status
  exit_inspect_controller_status=$(cq_controller_receipt_status \
    "$CQ_BOUNDARY_EVIDENCE/controller/$CQ_PHASE-exit-inspect") \
    || return 125
  test "$exit_inspect_call_status" -eq \
    "$exit_inspect_controller_status" || return 125
  test "$exit_inspect_controller_status" -eq 0 || return 125

  /usr/bin/python3 - \
    "$exit_inspect" "$phase_boundary/container-exit-state.json" \
    "$phase_boundary/child-status.txt" "$container_id" "$container_name" \
    "$CQ_EXECUTOR_CONFIG_SHA256" <<'PY_EXIT_STATE' || return 125
import json
from pathlib import Path
import sys

inspection_path = Path(sys.argv[1])
state_output_path = Path(sys.argv[2])
status_output_path = Path(sys.argv[3])
container_id = sys.argv[4]
container_name = sys.argv[5]
config_sha256 = sys.argv[6]
inspection = json.loads(inspection_path.read_text(encoding="utf-8"))
if len(inspection) != 1:
    raise SystemExit("expected one exited-container inspection")
container = inspection[0]
if container["Id"] != container_id:
    raise SystemExit("exited container ID changed")
if container["Name"].removeprefix("/") != container_name:
    raise SystemExit("exited container name changed")
if container["Image"].removeprefix("sha256:") != config_sha256:
    raise SystemExit("exited container image changed")
state = container["State"]
exit_code = state.get("ExitCode")
if type(exit_code) is not int or not 0 <= exit_code <= 255:
    raise SystemExit("container exit code is not an unsigned byte")
if (
    state.get("Status") != "exited"
    or state.get("Running") is not False
    or state.get("Paused") is not False
    or state.get("Restarting") is not False
    or state.get("OOMKilled") is not False
    or state.get("Dead") is not False
    or state.get("Error") != ""
):
    raise SystemExit("container did not reach a clean exited state")
for field in ("StartedAt", "FinishedAt"):
    value = state.get(field)
    if not isinstance(value, str) or not value or value.startswith("0001-"):
        raise SystemExit(f"container state has no valid {field}")
state_output_path.write_text(
    json.dumps(
        {
            "error": state["Error"],
            "exit_code": exit_code,
            "finished_at": state["FinishedAt"],
            "oom_killed": state["OOMKilled"],
            "schema": "io.nisavid.codiquary.container-exit-state/v1",
            "started_at": state["StartedAt"],
            "status": state["Status"],
        },
        indent=2,
        sort_keys=True,
    )
    + "\n",
    encoding="utf-8",
)
status_output_path.write_text(f"{exit_code}\n", encoding="ascii")
PY_EXIT_STATE

  local child_status
  child_status=$(/usr/bin/python3 -c \
    'import pathlib,sys; print(int(pathlib.Path(sys.argv[1]).read_text()))' \
    "$phase_boundary/child-status.txt") || return 125

  local wait_stdout="$phase_boundary/container-wait.txt"
  local wait_stderr="$phase_boundary/container-wait-stderr.txt"
  local wait_call_status
  if cq_oci_controller "$CQ_EXECUTION_STATE" "$CQ_BOUNDARY_EVIDENCE" \
    "$CQ_PHASE-wait" wait --condition=exited "$container_id" \
    > "$wait_stdout" 2> "$wait_stderr"; then
    wait_call_status=0
  else
    wait_call_status=$?
  fi
  local wait_controller_status
  wait_controller_status=$(cq_controller_receipt_status \
    "$CQ_BOUNDARY_EVIDENCE/controller/$CQ_PHASE-wait") || return 125
  test "$wait_call_status" -eq "$wait_controller_status" || return 125
  test "$wait_controller_status" -eq 0 || return 125
  local -a waited_statuses=()
  mapfile -t waited_statuses < "$wait_stdout" || return 125
  test "${#waited_statuses[@]}" -eq 1 || return 125
  [[ "${waited_statuses[0]}" =~ ^(0|[1-9][0-9]{0,2})$ ]] || return 125
  test "$((10#${waited_statuses[0]}))" -le 255 || return 125
  test "${waited_statuses[0]}" -eq "$child_status" || return 125
  test "$start_controller_status" -eq "$child_status" || return 125
  test "$(/usr/bin/head -n 1 "$preflight_receipt")" = \
    CODIQUARY_EXECUTOR_PREFLIGHT_V1 || return 125
  test "$(/usr/bin/tail -n 1 "$preflight_receipt")" = \
    CODIQUARY_EXECUTOR_PREFLIGHT_COMPLETE || return 125
  /usr/bin/python3 - "$preflight_receipt" "$mount_provenance_sha256" \
    "$phase_boundary/post-start-mount-projection.json" \
    <<'PY_POST_START_MOUNTS' || return 125
import json
from pathlib import Path
import sys

preflight_path = Path(sys.argv[1])
expected_sha256 = sys.argv[2]
output_path = Path(sys.argv[3])
projections = []
for line in preflight_path.read_text(encoding="utf-8").splitlines():
    try:
        candidate = json.loads(line)
    except json.JSONDecodeError:
        continue
    if (
        isinstance(candidate, dict)
        and candidate.get("schema")
        == "io.nisavid.codiquary.container-mount-projection/v4"
    ):
        projections.append(candidate)
if len(projections) != 1:
    raise SystemExit("preflight did not emit one post-start mount projection")
projection = projections[0]
if projection.get("mount_provenance_sha256") != expected_sha256:
    raise SystemExit("post-start mount projection used different provenance")
output_path.write_text(
    json.dumps(projection, indent=2, sort_keys=True) + "\n",
    encoding="utf-8",
)
PY_POST_START_MOUNTS

  /usr/bin/python3 - \
    "$initialized_inspect" "$phase_boundary/effective-oci-config.json" \
    "$phase_boundary/effective-oci-config-recheck.json" "$container_id" \
    <<'PY_EFFECTIVE_RECHECK' || return 125
import hashlib
import json
from pathlib import Path
import sys

inspection = json.loads(Path(sys.argv[1]).read_text(encoding="utf-8"))
retained_path = Path(sys.argv[2])
output_path = Path(sys.argv[3])
container_id = sys.argv[4]
if len(inspection) != 1 or inspection[0].get("Id") != container_id:
    raise SystemExit("initialized inspection no longer binds the retained container")
oci_config_path = Path(inspection[0]["OCIConfigPath"])
effective_bytes = oci_config_path.read_bytes()
retained_bytes = retained_path.read_bytes()
if effective_bytes != retained_bytes:
    raise SystemExit("effective OCI specification changed across attached start")
output_path.write_text(
    json.dumps(
        {
            "container_id": container_id,
            "oci_config_sha256": hashlib.sha256(retained_bytes).hexdigest(),
            "schema": "io.nisavid.codiquary.effective-oci-spec-recheck/v1",
        },
        indent=2,
        sort_keys=True,
    )
    + "\n",
    encoding="utf-8",
)
PY_EFFECTIVE_RECHECK

  local cleanup_stdout="$phase_boundary/container-remove.txt"
  local cleanup_stderr="$phase_boundary/container-remove-stderr.txt"
  local cleanup_call_status
  if cq_oci_controller "$CQ_EXECUTION_STATE" "$CQ_BOUNDARY_EVIDENCE" \
    "$CQ_PHASE-remove" container rm "$container_id" \
    > "$cleanup_stdout" 2> "$cleanup_stderr"; then
    cleanup_call_status=0
  else
    cleanup_call_status=$?
  fi
  local cleanup_controller_status
  cleanup_controller_status=$(cq_controller_receipt_status \
    "$CQ_BOUNDARY_EVIDENCE/controller/$CQ_PHASE-remove") || return 125
  test "$cleanup_call_status" -eq "$cleanup_controller_status" || return 125
  test "$cleanup_controller_status" -eq 0 || return 125
  local -a removed_ids=()
  mapfile -t removed_ids < "$cleanup_stdout" || return 125
  test "${#removed_ids[@]}" -eq 1 || return 125
  test "${removed_ids[0]}" = "$container_id" || return 125

  /usr/bin/python3 - "$phase_state" \
    "$phase_boundary/phase-state-inventory.json" <<'PY_OUTPUT' \
    || return 125
import hashlib
import json
import os
from pathlib import Path
import stat
import sys

root = Path(sys.argv[1])
output = Path(sys.argv[2])
state_names = {"fixtures", "home", "output", "target", "tmp", "work"}
if {path.name for path in root.iterdir()} != state_names:
    raise SystemExit("phase state has an unexpected top-level entry")
entries = []
for state_name in sorted(state_names):
    state_root = root / state_name
    for path in [state_root, *state_root.rglob("*")]:
        relative = path.relative_to(root).as_posix()
        mode = path.lstat().st_mode
        if stat.S_ISDIR(mode):
            entry = {"bytes": 0, "mode": f"{stat.S_IMODE(mode):04o}",
                     "path": relative, "sha256": None, "type": "directory"}
        elif stat.S_ISREG(mode):
            digest = hashlib.sha256()
            with path.open("rb") as stream:
                while chunk := stream.read(1024 * 1024):
                    digest.update(chunk)
            entry = {"bytes": path.stat().st_size,
                     "mode": f"{stat.S_IMODE(mode):04o}", "path": relative,
                     "sha256": digest.hexdigest(), "type": "file"}
        elif stat.S_ISLNK(mode):
            if state_name in {"fixtures", "output"}:
                raise SystemExit(f"published phase symlink rejected: {relative}")
            target = os.readlink(path)
            target_bytes = os.fsencode(target)
            entry = {
                "bytes": len(target_bytes),
                "mode": f"{stat.S_IMODE(mode):04o}",
                "path": relative,
                "sha256": hashlib.sha256(target_bytes).hexdigest(),
                "target": target,
                "type": "symlink",
            }
        else:
            raise SystemExit(f"phase-state special file rejected: {relative}")
        entries.append(entry)
output.write_text(
    json.dumps({"entries": sorted(entries, key=lambda entry: entry["path"]),
                "schema": "io.nisavid.codiquary.phase-state/v1"},
               indent=2, sort_keys=True) + "\n"
)
PY_OUTPUT

  /usr/bin/sha256sum --check --strict \
    "$phase_boundary/reviewed-inputs.sha256" >/dev/null || return 125
  test "$(cq_controller_receipt_status \
    "$CQ_BOUNDARY_EVIDENCE/controller/$CQ_PHASE-image-inspect")" -eq 0 \
    || return 125
  test "$(cq_controller_receipt_status \
    "$CQ_BOUNDARY_EVIDENCE/controller/$CQ_PHASE-create")" -eq 0 \
    || return 125
  test "$(cq_controller_receipt_status \
    "$CQ_BOUNDARY_EVIDENCE/controller/$CQ_PHASE-created-inspect")" -eq 0 \
    || return 125
  test "$(cq_controller_receipt_status \
    "$CQ_BOUNDARY_EVIDENCE/controller/$CQ_PHASE-init")" -eq 0 \
    || return 125
  test "$(cq_controller_receipt_status \
    "$CQ_BOUNDARY_EVIDENCE/controller/$CQ_PHASE-initialized-inspect")" -eq 0 \
    || return 125
  test "$(cq_controller_receipt_status \
    "$CQ_BOUNDARY_EVIDENCE/controller/$CQ_PHASE-start")" \
    -eq "$child_status" || return 125
  test "$(cq_controller_receipt_status \
    "$CQ_BOUNDARY_EVIDENCE/controller/$CQ_PHASE-exit-inspect")" -eq 0 \
    || return 125
  test "$(cq_controller_receipt_status \
    "$CQ_BOUNDARY_EVIDENCE/controller/$CQ_PHASE-wait")" -eq 0 \
    || return 125
  test "$(cq_controller_receipt_status \
    "$CQ_BOUNDARY_EVIDENCE/controller/$CQ_PHASE-remove")" -eq 0 \
    || return 125

  local boundary_relative="phases/$CQ_PHASE"
  local -a boundary_entries=(
    "$boundary_relative/reviewed-inputs.sha256"
    "$boundary_relative/reviewed-inputs-check.txt"
    "$boundary_relative/phase-command-stdin.bin"
    "$boundary_relative/observed-cargo-home-inventory.json"
    "$boundary_relative/image-inspect.json"
    "$boundary_relative/image-inspect-stderr.txt"
    "$boundary_relative/container-id.txt"
    "$boundary_relative/container-create-stderr.txt"
    "$boundary_relative/container-created-inspect.json"
    "$boundary_relative/container-created-inspect-stderr.txt"
    "$boundary_relative/declared-mounts.json"
    "$boundary_relative/container-init.txt"
    "$boundary_relative/container-init-stderr.txt"
    "$boundary_relative/container-initialized-inspect.json"
    "$boundary_relative/container-initialized-inspect-stderr.txt"
    "$boundary_relative/effective-oci-config.json"
    "$boundary_relative/effective-mounts.json"
    "$boundary_relative/pre-start-mount-provenance.json"
    "$boundary_relative/start-stdin.bin"
    "$boundary_relative/preflight.txt"
    "$boundary_relative/preflight-stderr.txt"
    "$boundary_relative/post-start-mount-projection.json"
    "$boundary_relative/container-exit-inspect.json"
    "$boundary_relative/container-exit-inspect-stderr.txt"
    "$boundary_relative/container-exit-state.json"
    "$boundary_relative/child-status.txt"
    "$boundary_relative/container-wait.txt"
    "$boundary_relative/container-wait-stderr.txt"
    "$boundary_relative/effective-oci-config-recheck.json"
    "$boundary_relative/container-remove.txt"
    "$boundary_relative/container-remove-stderr.txt"
    "$boundary_relative/phase-state-inventory.json"
    "controller/$CQ_PHASE-image-inspect/receipt.sha256"
    "controller/$CQ_PHASE-create/receipt.sha256"
    "controller/$CQ_PHASE-created-inspect/receipt.sha256"
    "controller/$CQ_PHASE-init/receipt.sha256"
    "controller/$CQ_PHASE-initialized-inspect/receipt.sha256"
    "controller/$CQ_PHASE-start/receipt.sha256"
    "controller/$CQ_PHASE-exit-inspect/receipt.sha256"
    "controller/$CQ_PHASE-wait/receipt.sha256"
    "controller/$CQ_PHASE-remove/receipt.sha256"
  )
  local boundary_tmp boundary_tmp_name boundary_tmp_relative
  boundary_tmp=$(/usr/bin/mktemp \
    "$phase_boundary/.boundary.sha256.XXXXXX") || return 125
  boundary_tmp_name=${boundary_tmp##*/}
  boundary_tmp_relative="$boundary_relative/$boundary_tmp_name"
  if (
    cd "$CQ_BOUNDARY_EVIDENCE" &&
      /usr/bin/sha256sum "${boundary_entries[@]}" > "$boundary_tmp"
  ); then
    :
  else
    return 125
  fi
  cq_validate_sha256_manifest "$CQ_BOUNDARY_EVIDENCE" \
    "$boundary_tmp_relative" "${boundary_entries[@]}" || return 125
  /usr/bin/mv --no-target-directory \
    "$boundary_tmp" "$phase_boundary/boundary.sha256" || return 125
  cq_validate_sha256_manifest "$CQ_BOUNDARY_EVIDENCE" \
    "$boundary_relative/boundary.sha256" "${boundary_entries[@]}" \
    || return 125
  return "$child_status"
}
```

The first container after the offline load is only an admission observation.
Run it after the source/input review and before any Cargo metadata or candidate
command:

```sh
CQ_PHASE=executor-admission
cq_linux_run /usr/bin/true
```

That command must return zero and its receipts must show the reviewed local
config/image ID, matching rootfs diff IDs, exact executor inventory, selected
libclang target, closed Podman configuration before and after every action, a
clean initialized state, the retained effective OCI specification, exactly ten
declared host binds, only allowlisted task-state runtime binds and pseudo
filesystems, the exact initialized namespace and security-submount set, matching
post-start device/root/source/filesystem/access provenance, dropped
capabilities, `NoNewPrivs`, empty route tables, and both `ENETUNREACH` results.
A failure releases no fallback command; correct or replace the boundary and
renew review.

The controller, image inspection, initialized-container inspection, retained
OCI specification, effective-mount admission, preflight, status, and boundary
hashes remain in `CQ_BOUNDARY_EVIDENCE`, which is never mounted. The preflight
receives the hash-bound pre-start mount projection as the first record on
standard input. `cq_linux_run` preserves up to 4 MiB of finite caller input
after that record; terminal input is normalized to an empty phase input. The
provenance check consumes exactly the first record, and the phase command then
receives the preserved caller bytes or EOF. The preflight writes to the
container's stdout only until its completion marker, then
redirects the phase command's stdout and stderr into that phase's fresh
candidate-output mount. The phase command inherits no descriptor for the
host-only receipt directory. The host validates every controller manifest's
exact five
entries and their checksums before reading its status. It accepts child status
only from a retained container whose controller-owned inspection reports
`Status=exited`, an empty operational `Error`, no OOM kill, and the same exit
code returned by attached start and printed by a separately successful wait
action. It also requires the live `OCIConfigPath` bytes to equal the admitted
copy after the child exits. It then removes the stopped container, inventories
all six writable directories, validates the exact boundary-manifest shape and
checksums in a private same-directory temporary file, and atomically renames
that file to `boundary.sha256`. Candidate and fixture output reject symlinks;
the other fresh, non-reused phase directories record regular files,
directories, and symlinks, and reject special files. Candidate bytes can affect
their isolated phase state but cannot replace controller, effective-spec,
preflight, or completion evidence.

The reviewed OCI manifest plus the matching local config/image ID binds the
complete in-container executable search path, compiler support programs, shell
utilities, headers, and libraries. The per-command receipt rechecks the
procedure-selected front ends and exact libclang file at use. `env -i` prevents
host variables, including OpenSSL,
AWS-LC, pkg-config, bindgen, compiler, linker, Rust flag, and credential values,
from entering the container. Only the two explicit vendored-backend controls
are added.

The dependency review must reconcile the final executor inventory with every
reachable build script and proc macro. A newly required executable, library,
SDK, generator, environment input, Cargo-home entry, or writable mount stops
execution and renews that review.

Hosted macOS arm64 is a separate later admission. Linux-generated macOS target
metadata is not a hosted observation. Before hosted fixture-independent replay,
bind Bash with `mapfile`, Python 3.11 or newer, GNU-compatible preparation and
checksum tools, Rust/Cargo 1.98.1 with rustfmt and Clippy, exact Xcode and SDK
identities, one exact libclang file, a fresh verified Cargo home, and an
OS-enforced offline boundary. Do not reuse `cq_linux_run` or report Linux
executor evidence as macOS qualification.

The coordinator populates the resolution Cargo home in a separate networked
acquisition boundary with exactly two `cargo fetch --locked` commands, one for
each reviewed root. It runs no build command. Construct and review the
registry-only execution home above, then verify its complete inventory and
locked contents again inside the admitted network-denied executor:

```sh
CQ_PHASE=cache-verification
cq_linux_run /usr/bin/python3 - \
  /cargo \
  /repo/proofs/exact-target-tough/Cargo.lock \
  /repo/proofs/exact-target-tough/locks/tough-workspace.Cargo.lock \
  /execution-cargo-home-inventory.json \
  /phase/output/locked-cache-verification.json <<'PY'
import hashlib
import json
from pathlib import Path, PurePosixPath
import stat
import sys
import tarfile
import tomllib

cargo_home = Path(sys.argv[1])
lock_paths = [Path(sys.argv[2]), Path(sys.argv[3])]
inventory_path = Path(sys.argv[4])
output_path = Path(sys.argv[5])

if {path.name for path in cargo_home.iterdir()} != {"registry"}:
    raise SystemExit("execution Cargo home top-level allowlist changed")
inventory_entries = [
    {"bytes": 0, "mode": f"{stat.S_IMODE(cargo_home.lstat().st_mode):04o}",
     "path": ".", "sha256": None, "type": "directory"}
]
registry = cargo_home / "registry"
for path in [registry, *registry.rglob("*")]:
    relative = path.relative_to(cargo_home).as_posix()
    mode = path.lstat().st_mode
    if stat.S_ISLNK(mode):
        raise SystemExit(f"symlink in execution Cargo home: {relative}")
    if stat.S_ISDIR(mode):
        entry = {"bytes": 0, "mode": f"{stat.S_IMODE(mode):04o}",
                 "path": relative, "sha256": None, "type": "directory"}
    elif stat.S_ISREG(mode):
        digest = hashlib.sha256()
        with path.open("rb") as stream:
            while chunk := stream.read(1024 * 1024):
                digest.update(chunk)
        entry = {"bytes": path.stat().st_size,
                 "mode": f"{stat.S_IMODE(mode):04o}", "path": relative,
                 "sha256": digest.hexdigest(), "type": "file"}
    else:
        raise SystemExit(f"special execution Cargo entry: {relative}")
    inventory_entries.append(entry)
reviewed_inventory = json.loads(inventory_path.read_text())
source_id = "index.crates.io-1949cf8c6b5b557f"
crates_io_source = "registry+https://github.com/rust-lang/crates.io-index"
if (
    reviewed_inventory.get("schema")
    != "io.nisavid.codiquary.execution-cargo-home/v2"
    or reviewed_inventory.get("source_id") != source_id
):
    raise SystemExit("reviewed Cargo-home inventory has the wrong schema")

required = {}
lock_digests = {}
for label, lock_path in zip(("proof", "tough-workspace"), lock_paths, strict=True):
    lock_bytes = lock_path.read_bytes()
    lock_digests[label] = hashlib.sha256(lock_bytes).hexdigest()
    lock = tomllib.loads(lock_bytes.decode("utf-8"))
    for package in lock["package"]:
        source = package.get("source")
        checksum = package.get("checksum")
        if source is None:
            continue
        if source != crates_io_source or checksum is None:
            raise SystemExit(
                f"unsupported non-path lock source: {package['name']} {package['version']}"
            )
        key = (package["name"], package["version"])
        previous = required.setdefault(key, checksum)
        if previous != checksum:
            raise SystemExit(f"conflicting locked checksum: {key}")

cache_root = cargo_home / "registry" / "cache" / source_id
source_root = cargo_home / "registry" / "src" / source_id
for label, path in (("cache", cache_root), ("source", source_root)):
    if not path.is_dir() or path.is_symlink():
        raise SystemExit(f"missing regular crates.io {label} directory")
observed_archives = set(cache_root.glob("*.crate"))
verified = []
required_archives = set()
source_archive_bindings = []
required_sources = {f"{name}-{version}" for name, version in required}
observed_sources = {path.name for path in source_root.iterdir() if path.is_dir()}
if observed_sources != required_sources:
    missing = sorted(required_sources - observed_sources)
    extra = sorted(observed_sources - required_sources)
    raise SystemExit(f"registry source set mismatch: missing={missing!r} extra={extra!r}")

def hash_file(path: Path) -> str:
    digest = hashlib.sha256()
    with path.open("rb") as stream:
        while chunk := stream.read(1024 * 1024):
            digest.update(chunk)
    return digest.hexdigest()

for (name, version), expected in sorted(required.items()):
    package_directory = f"{name}-{version}"
    archive = cache_root / f"{package_directory}.crate"
    if not archive.is_file() or archive.is_symlink():
        raise SystemExit(f"missing regular cached archive: {package_directory}")
    actual = hash_file(archive)
    if actual != expected:
        raise SystemExit(f"locked checksum mismatch: {name} {version}")
    required_archives.add(archive)
    verified.append(
        {
            "name": name,
            "sha256": actual,
            "source": crates_io_source,
            "version": version,
        }
    )

    source_directory = source_root / package_directory
    marker = source_directory / ".cargo-ok"
    marker_mode = marker.lstat().st_mode
    if not stat.S_ISREG(marker_mode) or marker.is_symlink():
        raise SystemExit(f"missing regular Cargo completion marker: {package_directory}")

    actual_directories = {"."}
    actual_files = set()
    for path in source_directory.rglob("*"):
        relative = path.relative_to(source_directory).as_posix()
        mode = path.lstat().st_mode
        if stat.S_ISDIR(mode):
            actual_directories.add(relative)
        elif stat.S_ISREG(mode):
            if relative != ".cargo-ok":
                actual_files.add(relative)
        else:
            raise SystemExit(f"non-regular extracted source entry: {relative}")

    records = {}
    with tarfile.open(archive, "r:gz") as bundle:
        for member in bundle.getmembers():
            raw_name = member.name[:-1] if member.name.endswith("/") else member.name
            member_path = PurePosixPath(raw_name)
            if (
                not raw_name
                or "\n" in raw_name
                or member_path.is_absolute()
                or member_path.as_posix() != raw_name
                or ".." in member_path.parts
                or member_path.parts[0] != package_directory
            ):
                raise SystemExit(f"unsafe crate member: {member.name!r}")
            relative_parts = member_path.parts[1:]
            relative = "/".join(relative_parts) if relative_parts else "."
            if relative in records:
                raise SystemExit(f"duplicate crate member: {member.name!r}")
            if relative == "." and not member.isdir():
                raise SystemExit(f"crate root is not a directory: {member.name!r}")
            if member.isdir():
                records[relative] = {
                    "bytes": 0,
                    "path": relative,
                    "sha256": None,
                    "type": "directory",
                }
                continue
            if not member.isreg() or relative == ".cargo-ok":
                raise SystemExit(f"non-regular or reserved crate member: {member.name!r}")
            extracted = bundle.extractfile(member)
            if extracted is None:
                raise SystemExit(f"crate member has no bytes: {relative}")
            digest = hashlib.sha256()
            observed_bytes = 0
            while chunk := extracted.read(1024 * 1024):
                digest.update(chunk)
                observed_bytes += len(chunk)
            if observed_bytes != member.size:
                raise SystemExit(f"crate member size changed: {relative}")
            source_file = source_directory.joinpath(*PurePosixPath(relative).parts)
            source_mode = source_file.lstat().st_mode
            if not stat.S_ISREG(source_mode) or source_file.is_symlink():
                raise SystemExit(f"extracted source file is not regular: {relative}")
            source_hash = hash_file(source_file)
            if source_file.stat().st_size != member.size or source_hash != digest.hexdigest():
                raise SystemExit(f"extracted source differs from archive: {relative}")
            records[relative] = {
                "bytes": member.size,
                "path": relative,
                "sha256": source_hash,
                "type": "file",
            }

    expected_directories = {"."}
    expected_files = set()
    for relative, record in records.items():
        if relative == ".":
            continue
        parts = PurePosixPath(relative).parts
        expected_directories.update(
            "/".join(parts[:length]) for length in range(1, len(parts))
        )
        if record["type"] == "directory":
            expected_directories.add(relative)
        else:
            expected_files.add(relative)
    if actual_directories != expected_directories or actual_files != expected_files:
        raise SystemExit(f"extracted source shape differs from archive: {package_directory}")

    public_records = [record for _, record in sorted(records.items())]
    tree_bytes = json.dumps(
        public_records, separators=(",", ":"), sort_keys=True
    ).encode("utf-8")
    source_archive_bindings.append(
        {
            "archive_sha256": actual,
            "cargo_ok_sha256": hash_file(marker),
            "files": sum(record["type"] == "file" for record in public_records),
            "name": name,
            "source": crates_io_source,
            "source_directory": package_directory,
            "source_tree_sha256": hashlib.sha256(tree_bytes).hexdigest(),
            "version": version,
        }
    )

extras = sorted(str(path.relative_to(cargo_home)) for path in observed_archives - required_archives)
if extras:
    raise SystemExit("unreviewed registry archives in execution cache: " + ", ".join(extras))

observed_inventory = {
    "allowed_root_entries": ["registry"],
    "entries": sorted(inventory_entries, key=lambda entry: entry["path"]),
    "schema": "io.nisavid.codiquary.execution-cargo-home/v2",
    "source_archive_bindings": source_archive_bindings,
    "source_id": source_id,
}
if observed_inventory != reviewed_inventory:
    raise SystemExit("mounted Cargo home differs from reviewed inventory")

output_path.write_text(
    json.dumps(
        {
            "cargo_home_inventory_sha256": hashlib.sha256(
                inventory_path.read_bytes()
            ).hexdigest(),
            "inventory_entries": len(inventory_entries),
            "locks": lock_digests,
            "registry_archives": verified,
            "registry_source_bindings": source_archive_bindings,
        },
        indent=2,
        sort_keys=True,
    )
    + "\n",
    encoding="utf-8",
)
PY
```

Only after that exact cache verification and its executor-boundary receipt
succeed may the coordinator collect the two read-only Cargo metadata
observations from the admitted executor. This command regenerates no lock and
runs no build script, proc macro, library, fixture author, or test:

```sh
CQ_PHASE=executor-cargo-metadata
cq_linux_run /bin/bash -c '
  set -euo pipefail
  /bin/bash /repo/proofs/exact-target-tough/scripts/prepare-sources.sh \
    --dependency-checkpoint /inputs \
    /repo/proofs/exact-target-tough/.work/tough
  cp /repo/proofs/exact-target-tough/locks/tough-workspace.Cargo.lock \
    /repo/proofs/exact-target-tough/.work/tough/Cargo.lock
  printf "%s  %s\n" \
    1d69534c34fc55d999c7cac409c5b44d2f036e6612e37a3021eb6f3d01cf57f4 \
    /repo/proofs/exact-target-tough/Cargo.lock \
    12c719f55434cde51602fc3c9da6145d509fac93831c80d529f5f8b33930b5fb \
    /repo/proofs/exact-target-tough/.work/tough/Cargo.lock \
    | sha256sum -c -
  "$CARGO" metadata \
    --manifest-path /repo/proofs/exact-target-tough/Cargo.toml \
    --locked --offline --format-version 1 \
    --filter-platform x86_64-unknown-linux-gnu \
    >/phase/output/admitted-proof-metadata-linux-x86_64.json
  "$CARGO" metadata \
    --manifest-path /repo/proofs/exact-target-tough/.work/tough/Cargo.toml \
    --locked --offline --format-version 1 \
    --filter-platform x86_64-unknown-linux-gnu \
    >/phase/output/admitted-tough-workspace-metadata-linux-x86_64.json
'
```

Freeze and independently review the source/input bundle, complete native
controller receipts, build-denial observation, OCI identity and inventory,
Cargo-home inventory, host-only admission receipts, public preflight, cache
verification, complete phase-state inventories, and these metadata files as one
renewed dependency/execution checkpoint. No candidate command is admitted
until that review is clean. The same reviewed cache then remains read-only for
fixture authoring, red, green, and final Linux replay.

## Operational observation gates

The corrected source admits no operation by itself. Preserve these observations
at each transition; a changed source, input, controller, image, cache, phase
boundary, or evidence dependency invalidates every affected later gate.

1. **Dependency checkpoint:** before resolution or fetching, require the clean
   reviewed source revision, exact two public source archives, fresh task-owned
   input, Cargo-home, and evidence directories, Rust/Cargo 1.98.1, and an empty
   index. Observe both generated locks, both hosted-target graphs, exact `-p
   tough` reachability, build scripts, proc macros, licenses, checksums, and the
   prepared path-source digest before any compilation.
2. **Offline OCI construction:** before staging or build, require the reviewed
   executor source/input bundle and acquired-file manifests. Observe valid
   atomic controller receipts for base save/load/inspection, build, derived
   inspection, and save, each bound to the closed empty configuration files;
   exact base and derived OCI identities; all held Debian and Rust-component
   installations; and retained IPv4/IPv6 build-denial bytes.
3. **Registry-only execution home:** before copying cache state, require both
   reviewed locks and a fresh resolution home populated only by their two
   fetches. Observe the exact crates.io archive set, every lock checksum, source
   reconstruction from those archive members, only regular `.cargo-ok` markers
   from resolution state, no unallowlisted entry, and the complete version-2
   inventory. Review its digest before loading or execution.
4. **OCI load:** before load, require the reviewed OCI archive, identity digest,
   build receipts, and registry-home inventory. Observe a valid load receipt
   bound to the same closed configuration and a local inspection matching the
   reviewed config, platform, and rootfs diff IDs. Loading grants no container
   execution.
5. **Harmless Linux admission:** before `executor-admission`, require fresh
   phase state and the clean source review covering the lifecycle controller.
   Observe exact ten-bind create inspection with only six writable phase
   destinations; successful initialization under the closed configuration; an
   `OCIConfigPath` inside task-owned state; exact effective-spec host binds;
   separately admitted runtime-state binds, pseudo-filesystems, masks, and
   read-only paths; a stable initialized process identity; host-to-namespace
   source projections for the image root and every bind; and an exact
   post-start match on destinations, devices, roots, sources, filesystems, and
   access before the phase command. Also observe direct tool identities, empty
   routes, both `ENETUNREACH` probes, an unchanged effective specification after
   exit, clean exited state, successful wait and removal, the complete
   six-directory state inventory, and an atomically published boundary manifest
   whose entries and checksums validate.
6. **Locked cache verification:** before `cache-verification`, require the clean
   admission boundary. Observe every archive checksum and every archive-member
   path, type, length, and digest against its reconstructed `registry/src`
   counterpart, exact source and archive sets, both lock digests, the reviewed
   inventory digest, fresh phase state, and another valid lifecycle boundary.
7. **Read-only metadata:** before `executor-cargo-metadata`, require clean cache
   verification. Observe both exact offline target-filtered metadata graphs,
   unchanged locks and prepared source, fresh phase state, and a valid lifecycle
   boundary; then obtain the renewed combined provenance/dependency and
   execution review.
8. **Fixture author, red, and green:** before fixture authoring, require that
   renewed review. Before red, require reviewed public fixture integration and
   the one accepted test. Observe public-only fixture output; then Cargo status
   `101`, the exact failed-test line, and one behavioral sentinel from a fresh
   red phase. Before green, require the source correction under review and a
   new phase; observe the named passing test with no red sentinel. Neither phase
   may consume another phase's writable state.
9. **Final replay and hosted macOS:** before final Linux replay, require one
   clean whole-proof review on the complete final revision and every preceding
   Linux boundary. Observe the complete typed replay and evidence checksums on
   that revision. Hosted macOS requires its separate native tool, SDK, libclang,
   cache, offline-boundary, replay, and checksum observations before any
   cross-platform conclusion.

This correction adds no synthetic child-`125` proof case. The retained-state,
successful-wait, and inspected-error branch is available for source review; a
dedicated runtime observation of child `125` would require separate scope
because the accepted case matrix does not contain it.

## TDD writing gate

After the coordinator returns a clean renewed dependency review, prepare the
single positive fixture and write only
`exact_match_reaches_observation_helper_once`. It must traverse Tough's real
`RepositoryLoader`, authenticate separately signed root, timestamp, snapshot,
and top-level targets metadata, match the closed six-field application intent,
consume and verify the inert target through EOF, create one held allocation,
and call the observation helper once. The counting transport, observation
helper, and proof-only clock are the only substitutions. Do not mock Tough,
Sequoia, OpenSSL, digest verification, or canonicalization.

The fixture author runs through `cq_linux_run` and writes only to the empty
fixture-output mount. This command is exact; no fixture may come from a host
toolchain or another image:

```sh
CQ_PHASE=fixture-author
cq_linux_run /bin/bash -c '
  set -euo pipefail
  /bin/bash /repo/proofs/exact-target-tough/scripts/prepare-sources.sh \
    --through-0002 /inputs /repo/proofs/exact-target-tough/.work/tough
  cp /repo/proofs/exact-target-tough/locks/tough-workspace.Cargo.lock \
    /repo/proofs/exact-target-tough/.work/tough/Cargo.lock
  "$CARGO" run \
    --manifest-path /repo/proofs/exact-target-tough/Cargo.toml \
    --locked --offline --target x86_64-unknown-linux-gnu \
    --bin author-fixtures -- --output /phase/fixtures
  cd /phase/fixtures
  find . -type f -print | LC_ALL=C sort \
    | while IFS= read -r path; do sha256sum "$path"; done \
    >/phase/output/fixture-author-output.sha256
'
```

The coordinator checks that receipt, integrates only the public outputs from
`$CQ_PHASE_STATE_ROOT/fixture-author/fixtures` into the fixture paths, and
proves that no secret key packet or disposable signing material was serialized.
The first-slice source worker then writes the one accepted red test against
those held public bytes.

For the first Linux slice, bind the reviewed source revision and both unchanged
locks. The red phase prepares its own Tough tree through patch 0002, installs
the reviewed upstream lock, preserves the listing, and runs the test without
sharing writable state with any other phase:

```sh
CQ_REVISION=${CQ_REVISION:?set the reviewed dependency-checkpoint revision}
test "$(git rev-parse HEAD)" = "$CQ_REVISION"
printf '%s  %s\n' \
  "$CQ_PROOF_LOCK_SHA256" "$CQ_PROOF/Cargo.lock" \
  "$CQ_TOUGH_LOCK_SHA256" "$CQ_PROOF/locks/tough-workspace.Cargo.lock" \
  | /usr/bin/sha256sum -c -

```

The test must emit the unique behavioral sentinel below only after the named
test has started and observed zero helper calls where one is required. Cargo
status `101`, the exact failed-test line, and exactly one sentinel are all
required. A compile, link, executor, or infrastructure failure cannot satisfy
red.

```sh
CQ_PHASE=exact-match-red
red_status=0
if cq_linux_run /bin/bash -c '
  set -euo pipefail
  /bin/bash /repo/proofs/exact-target-tough/scripts/prepare-sources.sh \
    --through-0002 /inputs /repo/proofs/exact-target-tough/.work/tough
  cp /repo/proofs/exact-target-tough/locks/tough-workspace.Cargo.lock \
    /repo/proofs/exact-target-tough/.work/tough/Cargo.lock
  sha256sum /repo/proofs/exact-target-tough/.work/tough/.codiquary-prepared-tree \
    >/phase/output/red-prepared-tree.sha256
  printf "%s  %s\n" \
    12c719f55434cde51602fc3c9da6145d509fac93831c80d529f5f8b33930b5fb \
    /repo/proofs/exact-target-tough/.work/tough/Cargo.lock \
    | sha256sum -c -
  if "$CARGO" test \
      --manifest-path /repo/proofs/exact-target-tough/Cargo.toml \
      --locked --offline --target x86_64-unknown-linux-gnu --test exact_target \
      exact_match_reaches_observation_helper_once \
      -- --exact --nocapture \
      2>&1 | /usr/bin/tee /phase/output/exact-match-red.log; then
    test_status=0
  else
    test_status=$?
  fi
  exit "$test_status"
'; then
  red_status=0
else
  red_status=$?
fi
test "$red_status" -eq 101
grep -Fx 'test exact_match_reaches_observation_helper_once ... FAILED' \
  "$CQ_PHASE_STATE_ROOT/exact-match-red/output/exact-match-red.log"
test "$(grep -Fxc \
  'EXACT_TARGET_BEHAVIOR_RED: helper_calls=0 expected=1' \
  "$CQ_PHASE_STATE_ROOT/exact-match-red/output/exact-match-red.log")" -eq 1
```

Add only the behavior portions of patches 0001 and 0002 and the minimum verifier
implementation. The green phase starts with new writable state, prepares
through 0002, reinstalls the unchanged reviewed lock, preserves the new
listing, and then runs green through the same reviewed executor image:

```sh
printf '%s  %s\n' \
  "$CQ_PROOF_LOCK_SHA256" "$CQ_PROOF/Cargo.lock" \
  "$CQ_TOUGH_LOCK_SHA256" "$CQ_PROOF/locks/tough-workspace.Cargo.lock" \
  | /usr/bin/sha256sum -c -

CQ_PHASE=exact-match-green
cq_linux_run /bin/bash -c '
  set -euo pipefail
  /bin/bash /repo/proofs/exact-target-tough/scripts/prepare-sources.sh \
    --through-0002 /inputs /repo/proofs/exact-target-tough/.work/tough
  cp /repo/proofs/exact-target-tough/locks/tough-workspace.Cargo.lock \
    /repo/proofs/exact-target-tough/.work/tough/Cargo.lock
  sha256sum /repo/proofs/exact-target-tough/.work/tough/.codiquary-prepared-tree \
    >/phase/output/green-prepared-tree.sha256
  printf "%s  %s\n" \
    12c719f55434cde51602fc3c9da6145d509fac93831c80d529f5f8b33930b5fb \
    /repo/proofs/exact-target-tough/.work/tough/Cargo.lock \
    | sha256sum -c -
  "$CARGO" test \
    --manifest-path /repo/proofs/exact-target-tough/Cargo.toml \
    --locked --offline --target x86_64-unknown-linux-gnu --test exact_target \
    exact_match_reaches_observation_helper_once \
    -- --exact --nocapture \
    2>&1 | /usr/bin/tee /phase/output/exact-match-green.log
'
grep -Fx 'test exact_match_reaches_observation_helper_once ... ok' \
  "$CQ_PHASE_STATE_ROOT/exact-match-green/output/exact-match-green.log"
! grep -Fq 'EXACT_TARGET_BEHAVIOR_RED:' \
  "$CQ_PHASE_STATE_ROOT/exact-match-green/output/exact-match-green.log"
```

Pipeline failure stops the shell. Add no hostile case in this slice. The
committed fixtures contain only public certificates and signed public bytes;
disposable signing material exists only in the fixture author's process memory
and is never serialized.

## Final replay gate

Do not run this section until the implementation, fixtures, three complete
patches, `prepared-tree.sha256`, both lockfiles, fixture oracle, replay script,
workflow, executor source, executor image, and this procedure have a clean
review on the same revision. The Linux final uses the same reviewed image
digest as fixture authoring, red, and green. Hosted macOS uses the same reviewed
source and locks but waits for its separate native admission.

```sh
CQ_REVISION=${CQ_REVISION:?set the clean reviewed revision}
test "$(git rev-parse HEAD)" = "$CQ_REVISION"
test -z "$(git status --porcelain=v1)"
printf '%s  %s\n' \
  "$CQ_PROOF_LOCK_SHA256" "$CQ_PROOF/Cargo.lock" \
  "$CQ_TOUGH_LOCK_SHA256" "$CQ_PROOF/locks/tough-workspace.Cargo.lock" \
  | /usr/bin/sha256sum -c -

CQ_PHASE=final-linux-replay
cq_linux_run /bin/bash -c '
  set -euo pipefail
  repo=/repo
  proof=/repo/proofs/exact-target-tough
  target=x86_64-unknown-linux-gnu
  evidence=/phase/output

  test "$(git -C "$repo" rev-parse HEAD)" = "'$CQ_REVISION'"
  test -z "$(git -C "$repo" status --porcelain=v1)"
  (cd /inputs && sha256sum -c "$proof/source-archives.sha256")
  /bin/bash "$proof/scripts/prepare-sources.sh" \
    --all /inputs "$proof/.work/tough"
  (cd "$proof/.work/tough" && sha256sum -c "$proof/prepared-tree.sha256")
  python3 "$proof/scripts/verify-fixtures.py" \
    "$repo/fixtures/conformance/exact-target-v1/manifest.json"

  "$CARGO" metadata --manifest-path "$proof/Cargo.toml" \
    --locked --offline --format-version 1 --filter-platform "$target" \
    >"$evidence/proof-metadata.json"
  "$CARGO" metadata --manifest-path "$proof/.work/tough/Cargo.toml" \
    --locked --offline --format-version 1 --filter-platform "$target" \
    >"$evidence/tough-metadata.json"
  "$CARGO" tree --manifest-path "$proof/Cargo.toml" \
    --locked --offline --target "$target" -e features \
    >"$evidence/proof-features.txt"
  "$CARGO" tree --manifest-path "$proof/.work/tough/Cargo.toml" \
    --locked --offline -p tough --target "$target" -e features \
    >"$evidence/tough-package-features.txt"

  "$CARGO" fmt --manifest-path "$proof/Cargo.toml" --all -- --check
  "$CARGO" clippy --manifest-path "$proof/Cargo.toml" \
    --locked --offline --target "$target" --all-targets -- -D warnings
  "$CARGO" test --manifest-path "$proof/Cargo.toml" \
    --locked --offline --target "$target" --all-targets -- --nocapture \
    2>&1 | tee "$evidence/proof-tests.log"
  "$CARGO" test --manifest-path "$proof/.work/tough/Cargo.toml" \
    --locked --offline -p tough --target "$target" --lib \
    2>&1 | tee "$evidence/tough-lib-tests.log"
  "$CARGO" test --manifest-path "$proof/.work/tough/Cargo.toml" \
    --locked --offline -p tough --target "$target" \
    --test raw_metadata_verification --test transport \
    --test expiration_enforcement --test interop \
    --test rotated_root --test target_path_safety \
    2>&1 | tee "$evidence/tough-relevant-tests.log"

  "$proof/scripts/replay.sh" \
    --manifest "$repo/fixtures/conformance/exact-target-v1/manifest.json" \
    --platform linux-x86_64 --output "$evidence"
  /bin/bash "$repo/scripts/check-repository.sh"
  git -C "$repo" diff --check
  test -z "$(git -C "$repo" status --porcelain=v1)"
  (
    cd "$evidence"
    find . -maxdepth 1 -type f ! -name evidence.sha256 -print \
      | LC_ALL=C sort \
      | while IFS= read -r path; do sha256sum "$path"; done \
      >evidence.sha256
  )
'
```

`replay.sh` must write `environment.json`, `source-and-locks.json`, and
`cases.jsonl`; `environment.json` must record
`CQ_EXECUTOR_MANIFEST_SHA256`, `CQ_EXECUTOR_CONFIG_SHA256`, and
`CQ_EXECUTOR_IMAGE_ID`, plus the SHA-256 of
`/execution-cargo-home-inventory.json`, the direct Rust toolchain directory,
and the admission hashes for Cargo, rustc, rustdoc, rustfmt, `cargo-fmt`,
`clippy-driver`, and `cargo-clippy`. The fixture manifest's typed
expectations—not exact command text—are the result oracle.
Review the procedure before the Linux run. After the separate hosted-macOS
admission and replay, review both platforms' checksums and typed results before
any dependent replay.
