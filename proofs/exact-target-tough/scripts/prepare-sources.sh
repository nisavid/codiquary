#!/usr/bin/env bash

set -euo pipefail

usage() {
    cat >&2 <<'EOF'
usage: prepare-sources.sh (--dependency-checkpoint|--through-0002|--all) INPUTS OUTPUT

INPUTS is the public-source directory whose relative layout matches
source-archives.sha256. OUTPUT must be proofs/exact-target-tough/.work/tough.
EOF
    exit 2
}

fail() {
    echo "prepare-sources.sh: $*" >&2
    exit 1
}

[[ $# -eq 3 ]] || usage
mode=$1
inputs=$2
output=$3
while [[ "$output" != / && "$output" == */ ]]; do
    output=${output%/}
done

script_dir=$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" && pwd -P)
proof_dir=$(cd -- "$script_dir/.." && pwd -P)
work_dir="$proof_dir/.work"
series_file="$proof_dir/patches/series"
source_manifest="$proof_dir/source-archives.sha256"

require_safe_destination() {
    local resolved_work_dir output_parent

    [[ ! -L "$work_dir" ]] || fail "refusing to use a symbolic-link work directory"
    resolved_work_dir=$(cd -- "$work_dir" && pwd -P)
    [[ "$resolved_work_dir" == "$work_dir" ]] ||
        fail "work directory must resolve to its expected physical repository location"
    output_parent=$(cd -- "$(dirname -- "$output")" && pwd -P)
    [[ "$output_parent" == "$work_dir" && "$(basename -- "$output")" == tough ]] ||
        fail "output must be $work_dir/tough"
    [[ ! -L "$output" ]] || fail "refusing to replace a symbolic-link output"
}

case "$mode" in
    --dependency-checkpoint | --through-0002)
        patch_limit=2
        ;;
    --all)
        patch_limit=all
        ;;
    *)
        usage
        ;;
esac

[[ -d "$inputs" ]] || fail "input directory does not exist: $inputs"
[[ ! -L "$work_dir" ]] || fail "refusing to use a symbolic-link work directory"
mkdir -p -- "$work_dir"
require_safe_destination

(cd -- "$inputs" && sha256sum -c "$source_manifest")

archive="$inputs/github-awslabs-tough/98d8eb8b2ce63515d9b4981c938ef6453c5b5771.tar.gz"
tmp=$(mktemp -d "$work_dir/.prepare.XXXXXX")
trap 'rm -rf -- "$tmp"' EXIT
mkdir -- "$tmp/unpacked"
tar -xzf "$archive" -C "$tmp/unpacked"

mapfile -t roots < <(find "$tmp/unpacked" -mindepth 1 -maxdepth 1 -type d -print)
[[ ${#roots[@]} -eq 1 ]] || fail "Tough archive must contain exactly one root directory"
mv -- "${roots[0]}" "$tmp/tree"

mapfile -t patches < <(sed -e '/^[[:space:]]*#/d' -e '/^[[:space:]]*$/d' "$series_file")
[[ ${#patches[@]} -eq 3 ]] || fail "patch series must name exactly three patches"

if [[ "$patch_limit" == all ]]; then
    apply_count=${#patches[@]}
else
    apply_count=$patch_limit
fi

for ((index = 0; index < apply_count; index++)); do
    patch_file="$proof_dir/patches/${patches[$index]}"
    [[ -f "$patch_file" ]] || fail "required patch is unfinished: ${patches[$index]}"
    patch_log="$tmp/patch-$index.log"
    if ! LC_ALL=C patch --directory="$tmp/tree" --strip=1 --fuzz=0 --forward \
        --batch --reject-file=- <"$patch_file" >"$patch_log" 2>&1; then
        sed -n '1,160p' "$patch_log" >&2
        fail "patch did not apply: ${patches[$index]}"
    fi
    if grep -Eiq 'offset|fuzz|reversed|previously applied' "$patch_log"; then
        sed -n '1,160p' "$patch_log" >&2
        fail "patch applied with a forbidden adjustment: ${patches[$index]}"
    fi
done

if [[ "$mode" == --all ]]; then
    lock="$proof_dir/locks/tough-workspace.Cargo.lock"
    tree_digest="$proof_dir/prepared-tree.sha256"
    [[ -f "$lock" ]] || fail "reviewed Tough workspace lockfile is missing"
    [[ -f "$tree_digest" ]] || fail "reviewed prepared-tree digest is missing"
    cp -- "$lock" "$tmp/tree/Cargo.lock"
fi

python3 - "$tmp/tree" >"$tmp/tree/.codiquary-prepared-tree" <<'PY'
import hashlib
import os
from pathlib import Path
import stat
import sys

root = Path(sys.argv[1])
for path in sorted(root.rglob("*"), key=lambda item: item.as_posix().encode()):
    relative = path.relative_to(root).as_posix()
    if relative == ".codiquary-prepared-tree":
        continue
    metadata = path.lstat()
    mode = f"{stat.S_IMODE(metadata.st_mode):04o}"
    if path.is_symlink():
        kind = "link"
        payload = os.readlink(path).encode()
    elif path.is_dir():
        kind = "directory"
        payload = b""
    elif path.is_file():
        kind = "file"
        payload = path.read_bytes()
    else:
        raise SystemExit(f"unsupported source-tree entry: {relative}")
    digest = hashlib.sha256(payload).hexdigest()
    print(f"{mode}\t{kind}\t{digest}\t{relative}")
PY

if [[ "$mode" == --all ]]; then
    (cd -- "$tmp/tree" && sha256sum -c "$proof_dir/prepared-tree.sha256")
else
    echo "dependency-checkpoint prepared-tree listing:" >&2
    sha256sum "$tmp/tree/.codiquary-prepared-tree" >&2
fi

require_safe_destination
if [[ -e "$output" ]]; then
    rm -rf -- "$output"
fi
mv -- "$tmp/tree" "$output"
echo "prepared $output through ${patches[$((apply_count - 1))]}" >&2
