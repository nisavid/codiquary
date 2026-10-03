#!/usr/bin/env python3
"""Check the replay controller lifecycle with inert external adapters."""

from __future__ import annotations

import hashlib
import json
import os
from pathlib import Path
import shlex
import signal
import subprocess
import sys


def fail(message: str) -> None:
    raise SystemExit(message)


def digest(path: Path) -> str:
    return hashlib.sha256(path.read_bytes()).hexdigest()


def replace_all(source: str, old: str, new: str, label: str) -> tuple[str, int]:
    count = source.count(old)
    if count == 0:
        fail(f"controller adaptation target is absent: {label}")
    return source.replace(old, new), count


def write_executable(path: Path, source: str) -> None:
    path.write_text(source.rstrip("\n") + "\n", encoding="utf-8", newline="\n")
    path.chmod(0o700)


def run_owned(command: list[str], work: Path) -> subprocess.CompletedProcess[bytes]:
    process = subprocess.Popen(
        command,
        cwd=work,
        env={
            "LC_ALL": "C",
            "PATH": "/usr/bin:/bin",
            "PYTHONDONTWRITEBYTECODE": "1",
            "TMPDIR": str(work),
        },
        start_new_session=True,
        stdout=subprocess.PIPE,
        stderr=subprocess.PIPE,
    )
    try:
        stdout, stderr = process.communicate(timeout=30)
    except subprocess.TimeoutExpired:
        os.killpg(process.pid, signal.SIGKILL)
        stdout, stderr = process.communicate()
        (work / "timeout.stdout").write_bytes(stdout)
        (work / "timeout.stderr").write_bytes(stderr)
        fail("owned inert controller harness timed out and was reaped")
    return subprocess.CompletedProcess(command, process.returncode, stdout, stderr)


if len(sys.argv) != 3:
    fail("usage: check-replay-lifecycle.py GUIDE NEW-WORK-DIRECTORY")

guide_path = Path(sys.argv[1]).resolve()
work = Path(sys.argv[2]).resolve()
if not guide_path.is_file():
    fail(f"guide is not a regular file: {guide_path}")
if work.exists():
    fail(f"work directory must be new: {work}")
work.mkdir(parents=True, mode=0o700)

guide = guide_path.read_text(encoding="utf-8")
if "\r" in guide:
    fail("guide must use LF line endings")
lifecycle_start = guide.find("cq_linux_run() (")
lifecycle_end = guide.find("\n```", lifecycle_start)
if lifecycle_start < 0 or lifecycle_end < 0:
    fail("lifecycle code block is absent or malformed")
lifecycle = guide[lifecycle_start:lifecycle_end]
identity_fragments = (
    'local start_controller_identity="$phase_boundary/start-controller-process.json"',
    '"schema": "io.nisavid.codiquary.start-controller-process/v1"',
    'if exe != "/usr/bin/bash":',
    'if read_starttime() != starttime:',
    '"$boundary_relative/start-controller-process.json"',
)
for fragment in identity_fragments:
    if lifecycle.count(fragment) != 1:
        fail(f"start-controller process identity contract changed: {fragment}")
if lifecycle.find('exec {start_stream_fd}<>"$start_stdin_fifo"') > lifecycle.find(
    '/usr/bin/python3 - "$start_controller_identity"'
):
    fail("start-controller identity is captured before FIFO deadlock protection")
start = guide.find("CQ_PODMAN_SHA256=")
end = guide.find("\n```", start)
if start < 0 or end < 0:
    fail("controller code block is absent or malformed")
controller = guide[start:end]
received_controller_sha256 = hashlib.sha256(controller.encode()).hexdigest()
accepted_controllers = {
    "fe30cf9ed9067070532dcc043548cdb624a7fc66e5ddfd49a7b53b260ef43fe6",
    "f9326942c9e416f11bdb02b7323e203838fd83fe516af44693ce11b339e83272",
}
if received_controller_sha256 not in accepted_controllers:
    fail("controller bytes differ from the reviewed final or no-pause-removed form")

for forbidden in ("/proc/", "/usr/bin/kill", "systemctl", "system migrate"):
    if forbidden in controller:
        fail(f"controller gained an unapproved actuation boundary: {forbidden}")

calls_path = work / "podman-calls.jsonl"
podman_stub = work / "podman-adapter.py"
tool_stub = work / "tool-adapter.py"
write_executable(
    podman_stub,
    f"""#!/usr/bin/python3
import json
import os
from pathlib import Path
import sys

record = {{"argv": sys.argv[1:], "environment": dict(os.environ)}}
with Path({str(calls_path)!r}).open("a", encoding="utf-8") as output:
    output.write(json.dumps(record, sort_keys=True) + "\\n")
if "mutate-input" in sys.argv:
    target = Path(sys.argv[sys.argv.index("mutate-input") + 1])
    target.write_text("changed\\n", encoding="utf-8")
if "version" in sys.argv:
    print("podman version 6.1.0")
if "synthetic-failure" in sys.argv:
    raise SystemExit(37)
""",
)
write_executable(
    tool_stub,
    """#!/usr/bin/python3
from pathlib import Path
import sys

if "id-adapter" in Path(sys.argv[0]).name:
    print("1000")
else:
    print(Path(sys.argv[0]).name + " synthetic version")
""",
)

adapters = {
    "podman": podman_stub,
    "crun": work / "crun-adapter.py",
    "conmon": work / "conmon-adapter.py",
    "pasta": work / "pasta-adapter.py",
    "id": work / "id-adapter.py",
}
for name in ("crun", "conmon", "pasta", "id"):
    adapters[name].write_bytes(tool_stub.read_bytes())
    adapters[name].chmod(0o700)

adaptations: list[dict[str, object]] = []
for name, production in (
    ("podman", "/usr/bin/podman"),
    ("crun", "/usr/bin/crun"),
    ("conmon", "/usr/bin/conmon"),
    ("pasta", "/usr/bin/pasta"),
    ("id", "/usr/bin/id"),
):
    replacement = shlex.quote(str(adapters[name]))
    controller, count = replace_all(controller, production, replacement, name)
    adaptations.append(
        {
            "occurrences": count,
            "production": production,
            "synthetic": str(adapters[name]),
            "synthetic_sha256": digest(adapters[name]),
        }
    )

for name, variable in (
    ("podman", "CQ_PODMAN_SHA256"),
    ("crun", "CQ_CRUN_SHA256"),
    ("conmon", "CQ_CONMON_SHA256"),
    ("pasta", "CQ_PASTA_SHA256"),
):
    prefix = f"{variable}="
    line = next((item for item in controller.splitlines() if item.startswith(prefix)), None)
    if line is None:
        fail(f"controller digest binding is absent: {variable}")
    controller = controller.replace(line, prefix + digest(adapters[name]), 1)

for production in (
    "/usr/bin/podman",
    "/usr/bin/crun",
    "/usr/bin/conmon",
    "/usr/bin/pasta",
    "/usr/bin/id",
):
    if production in controller:
        fail(f"external actuation boundary survived adaptation: {production}")

state = work / "state"
receipts = work / "receipts"
input_path = work / "controller-input.txt"
harness = work / "controller-harness.bash"
harness_source = controller + f"""

set -u
state={shlex.quote(str(state))}
receipts={shlex.quote(str(receipts))}
input_path={shlex.quote(str(input_path))}
cq_prepare_podman_state "$state"
mkdir -p "$receipts"

cq_oci_controller "$state" "$receipts" success image exists synthetic

failure_status=0
if cq_oci_controller "$state" "$receipts" action-failure \\
    image exists synthetic-failure; then
  failure_status=0
else
  failure_status=$?
fi
test "$failure_status" -eq 37

printf '%s\\n' original > "$input_path"
input_sha256=$(/usr/bin/sha256sum "$input_path")
input_sha256=${{input_sha256%% *}}
mutation_status=0
if cq_oci_controller "$state" "$receipts" changed-input \\
    --controller-input "$input_path" "$input_sha256" \\
    debug mutate-input "$input_path"; then
  mutation_status=0
else
  mutation_status=$?
fi
test "$mutation_status" -eq 125
test ! -e "$receipts/controller/changed-input/receipt.sha256"

cp -a "$receipts/controller/success" "$receipts/controller/omitted-evidence"
rm "$receipts/controller/omitted-evidence/status.txt"
if cq_controller_receipt_status "$receipts/controller/omitted-evidence"; then
  exit 91
fi

chmod 700 "$state/config"
mode_status=0
if cq_oci_controller "$state" "$receipts" changed-mode image exists synthetic; then
  mode_status=0
else
  mode_status=$?
fi
test "$mode_status" -eq 125
test ! -e "$receipts/controller/changed-mode"
"""
write_executable(harness, harness_source)

completed = run_owned(["/usr/bin/bash", "--noprofile", "--norc", str(harness)], work)
(work / "harness.stdout").write_bytes(completed.stdout)
(work / "harness.stderr").write_bytes(completed.stderr)
if completed.returncode != 0:
    fail(
        "controller harness failed: "
        f"status={completed.returncode} stderr={completed.stderr.decode(errors='replace').strip()}"
    )

calls = [json.loads(line) for line in calls_path.read_text().splitlines()]
if len(calls) != 6:
    fail(f"expected six version/action adapter calls, observed {len(calls)}")

expected_environment = {
    "CONTAINERS_CONF": str(state / "config/containers/containers.conf"),
    "HOME": str(state / "home"),
    "PATH": "/usr/bin:/bin",
    "PODMAN_NO_PAUSE_PROCESS": "1",
    "XDG_CONFIG_HOME": str(state / "config"),
    "XDG_RUNTIME_DIR": str(state / "runtime"),
}
global_argv = [
    "--root", str(state / "root"),
    "--runroot", str(state / "runroot"),
    "--storage-driver", "vfs",
    "--runtime", str(adapters["crun"]),
    "--conmon", str(adapters["conmon"]),
    "--cgroup-manager", "cgroupfs",
    "--events-backend", "file",
    "--hooks-dir", str(state / "hooks"),
]

case_actions = {
    "success": ["image", "exists", "synthetic"],
    "action-failure": ["image", "exists", "synthetic-failure"],
    "changed-input": ["debug", "mutate-input", str(input_path)],
}
for index, (name, action) in enumerate(case_actions.items()):
    version_call, action_call = calls[index * 2 : index * 2 + 2]
    version_environment = dict(version_call["environment"])
    action_environment = dict(action_call["environment"])
    if version_environment.pop("LC_CTYPE", None) != "C.UTF-8":
        fail(f"{name} version adapter locale initialization changed")
    if action_environment.pop("LC_CTYPE", None) != "C.UTF-8":
        fail(f"{name} action adapter locale initialization changed")
    if version_environment != expected_environment:
        fail(f"{name} version call did not receive the exact no-pause environment")
    if action_environment != expected_environment:
        fail(f"{name} action call did not receive the exact no-pause environment")
    if version_call["argv"] != global_argv + ["version"]:
        fail(f"{name} version adapter argv differs from the controller argv")
    if action_call["argv"] != global_argv + action:
        fail(f"{name} action adapter argv differs from the controller argv")

    receipt = receipts / "controller" / name
    version_receipt = shlex.split((receipt / "version-argv.txt").read_text())
    action_receipt = shlex.split((receipt / "argv.txt").read_text())
    environment_argv = [
        "/usr/bin/env", "-i",
        f"HOME={state / 'home'}",
        f"XDG_CONFIG_HOME={state / 'config'}",
        f"XDG_RUNTIME_DIR={state / 'runtime'}",
        f"CONTAINERS_CONF={state / 'config/containers/containers.conf'}",
        "PODMAN_NO_PAUSE_PROCESS=1",
        "PATH=/usr/bin:/bin",
    ]
    command_argv = [str(adapters["podman"]), *global_argv]
    if version_receipt != environment_argv + command_argv + ["version"]:
        fail(f"{name} version receipt does not bind the executed environment and argv")
    if action_receipt != environment_argv + command_argv + action:
        fail(f"{name} action receipt does not bind the executed environment and argv")

    if name != "changed-input":
        manifest = (receipt / "receipt.sha256").read_text().splitlines()
        expected_files = [
            "controller.sha256", "versions.txt", "version-argv.txt",
            "argv.txt", "status.txt",
        ]
        if [line.split("  ", 1)[1] for line in manifest] != expected_files:
            fail(f"{name} receipt manifest shape changed")
        for line in manifest:
            expected_digest, relative = line.split("  ", 1)
            if digest(receipt / relative) != expected_digest:
                fail(f"{name} receipt digest mismatch: {relative}")

result = {
    "adaptations": adaptations,
    "cases": {
        "action-failure": {"recorded_status": 37, "receipt_complete": True},
        "changed-input": {"status": 125, "receipt_complete": False},
        "changed-mode": {"status": 125, "controller_invoked": False},
        "omitted-evidence": {"receipt_accepted": False},
        "success": {"recorded_status": 0, "receipt_complete": True},
    },
    "adapted_controller_sha256": hashlib.sha256(controller.encode()).hexdigest(),
    "guide_sha256": digest(guide_path),
    "process_identity_contract": "separate FIFO and live Bash PID/starttime/UID/exe receipts",
    "received_controller_sha256": received_controller_sha256,
    "result": "passed",
    "scope": (
        "real controller gates and receipts with inert external executable "
        "adapters; Python adds only LC_CTYPE=C.UTF-8 after exec"
    ),
}
(work / "result.json").write_text(
    json.dumps(result, indent=2, sort_keys=True) + "\n", encoding="utf-8"
)
print(json.dumps(result, sort_keys=True))
