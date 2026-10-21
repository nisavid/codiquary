"""Validate workflow action references without constructing YAML objects."""

import re
import sys
from pathlib import Path

try:
    import yaml
    from yaml.nodes import MappingNode, ScalarNode, SequenceNode
except ImportError:
    sys.exit("workflow validation requires PyYAML: install scripts/requirements-policy.txt")


class PolicyError(ValueError):
    """A workflow cannot satisfy the action-reference policy."""


REFERENCE = re.compile(
    r"[A-Za-z0-9][A-Za-z0-9-]*/[A-Za-z0-9_.-]+(?:/[A-Za-z0-9_.-]+)*@[0-9a-f]{40}"
)
TAGS = {
    MappingNode: "tag:yaml.org,2002:map",
    SequenceNode: "tag:yaml.org,2002:seq",
    ScalarNode: "tag:yaml.org,2002:str",
}


def inspect_nodes(node, active=None, checked=None):
    """Reject ambiguous input; visit shared alias targets only once."""
    if active is None:
        active, checked = set(), set()
    identity = id(node)
    if identity in active:
        raise PolicyError("recursive YAML aliases are unsupported")
    if identity in checked:
        return
    if node is None or node.tag != TAGS.get(type(node)):
        raise PolicyError("empty document or unsupported YAML tag")
    active.add(identity)
    if isinstance(node, MappingNode):
        keys = set()
        for key, value in node.value:
            if not isinstance(key, ScalarNode) or key.tag != TAGS[ScalarNode]:
                raise PolicyError("mapping keys must be strings")
            name = key.value.lower()
            if name == "<<":
                raise PolicyError("YAML merge keys are unsupported")
            if name in keys:
                raise PolicyError(f"duplicate YAML key {key.value!r}")
            keys.add(name)
            inspect_nodes(value, active, checked)
    elif isinstance(node, SequenceNode):
        for value in node.value:
            inspect_nodes(value, active, checked)
    active.remove(identity)
    checked.add(identity)


def mapping(node, location):
    if not isinstance(node, MappingNode):
        raise PolicyError(f"{location} must be a mapping")
    # Actions' template reader compares property names without ASCII case.
    return {key.value.lower(): value for key, value in node.value}


def check_reference(node, location):
    if not isinstance(node, ScalarNode) or not REFERENCE.fullmatch(node.value):
        raise PolicyError(
            f"{location}: expected an external repository reference pinned to "
            "a full 40-character lowercase commit SHA; "
            "local, Docker, and dynamic references are unsupported"
        )


def check_steps(node, location, in_parallel=False):
    if not isinstance(node, SequenceNode):
        raise PolicyError(f"{location} must be a sequence")
    for index, node in enumerate(node.value):
        step_location = f"{location}[{index}]"
        step = mapping(node, step_location)
        if "uses" in step:
            check_reference(step["uses"], f"{step_location}.uses")
        if "parallel" in step:
            if in_parallel:
                raise PolicyError("nested parallel groups are unsupported")
            check_steps(step["parallel"], f"{step_location}.parallel", in_parallel=True)


def check_workflow(source):
    # Compose preserves decoded scalar text (including `on`) and resolves
    # aliases, but never invokes YAML object constructors.
    root = yaml.compose(source, Loader=yaml.BaseLoader)
    inspect_nodes(root)
    workflow = mapping(root, "workflow")
    jobs = mapping(workflow.get("jobs"), "jobs")
    if not jobs:
        raise PolicyError("jobs must not be empty")
    for name, node in jobs.items():
        location = f"jobs.{name}"
        job = mapping(node, location)
        if "uses" in job:
            check_reference(job["uses"], f"{location}.uses")
        if "steps" in job:
            check_steps(job["steps"], f"{location}.steps")
        elif "uses" not in job:
            raise PolicyError(f"{location} must contain steps or uses")


def main(directory):
    try:
        directory = Path(directory)
        if directory.is_symlink():
            raise PolicyError("workflow directory symlinks are unsupported")
        paths = sorted(path for path in directory.iterdir()
                       if path.suffix in {".yaml", ".yml"})
        if not paths:
            raise PolicyError("no workflow YAML files found")
    except (OSError, PolicyError) as error:
        print(f"{directory}: {error}", file=sys.stderr)
        return 1
    failed = False
    for path in paths:
        try:
            if path.is_symlink():
                raise PolicyError("workflow file symlinks are unsupported")
            check_workflow(path.read_text(encoding="utf-8"))
        except (OSError, UnicodeError, yaml.YAMLError, PolicyError, RecursionError) as error:
            print(f"{path}: {error}", file=sys.stderr)
            failed = True
    return int(failed)


if __name__ == "__main__":
    if len(sys.argv) != 2:
        sys.exit("usage: check_workflow_pins.py WORKFLOW_DIRECTORY")
    sys.exit(main(sys.argv[1]))
