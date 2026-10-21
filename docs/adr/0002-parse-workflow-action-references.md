# ADR 0002: Parse workflow action references before checking pins

- Status: Proposed; implementation and testing authorized, acceptance pending review
- Decision scope: repository contribution validation
- Owning remediation: [IDV-1989](https://linear.app/nisavid/issue/IDV-1989)

## Context and threat review

The repository's line-oriented action-pin check can omit executable YAML
mappings or accept a pinned reference inside a comment. A contributor can
therefore obtain a false successful policy result for a mutable external
reference. Exploitation additionally requires approval/admission of that
workflow and attacker influence over the external reference. Existing checked-in
actions are pinned; the finding does not establish production authority.

## Decision

Parse workflow representation nodes with PyYAML's `BaseLoader` and validate
all step and job `uses` values, including actions inside `parallel` groups. Do not construct arbitrary objects. Inspect the
representation graph before lookup, rejecting ambiguous or unsupported input.
Resolve nonrecursive aliases without repeatedly expanding shared nodes, reject
cycles and merge keys, and handle property names without ASCII case. Require
exact full lowercase commit SHAs for external references.

Keep this policy under `scripts/`, outside the non-operational Rust release
implementation. Provision the pinned parser dependency explicitly in CI and
local contributor environments. Treat missing inputs, dependencies, and parse
errors as failures. The existing shell entrypoint remains shared by CI,
pre-push, and manual validation.

## Compatibility and conformance

Existing pinned actions remain valid. Equivalent quoted, flow, explicit-key,
multiline, and alias forms receive the same reference check. Script text and
comments cease to cause false positives. Malformed or ambiguous workflows
previously missed by the text scan now fail. Local, Docker, and dynamic
references remain unsupported; this change adds no exemption from the prior
external-SHA requirement.

The new Python/PyYAML setup is required for the local pre-push hook as well as
manual checks. `scripts/test_workflow_pins.py` provides security and compatibility
regressions, including parallel/background steps and the actual shell entrypoint. No runtime contract,
adapter, deployment lock, or release qualification changes.

## Authority effect and remaining risks

This closes a representation bypass in repository validation. It cannot
establish action trust, commit existence, transitive dependency immutability,
or hosted review enforcement. Repository maintainers must review changes to
the checker and workflow together. PyYAML is a new development dependency;
its exact version is declared and dependency updates are monitored. Parser
or hosted interpretation changes require reevaluating this policy subset.
No release or production authority is added.

## Alternatives

Expanding the grep pattern would still confuse YAML syntax, aliases, and
scalar content. A custom YAML parser would add unnecessary maintenance and
parser-differential risk. Implementing this in the Rust runtime would couple
repository tooling to the unrelated release implementation and build toolchain.

The [workflow action-pin contract](../repository-controls/workflow-action-pins.md)
documents supported inputs and the precise limits of validation.
