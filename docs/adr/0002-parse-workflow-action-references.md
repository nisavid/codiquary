# ADR 0002: Parse workflow action references structurally

- Status: Proposed for repository review; implementation authorized by the maintainer
- Decision scope: repository action-pin enforcement

## Context

The existing line selector overlooks executable references written as
sequence-first, quoted, or flow mappings. Its unanchored value expression can
also accept a pin located in a comment. A successful check therefore does not
consistently establish the stated full-SHA policy.

## Decision

Replace line matching with safe YAML representation parsing at the existing
repository-policy entrypoint. Inspect step actions and job-level reusable
workflows and actions in parallel groups, and match their entire decoded reference values. Reject ambiguous
or unsupported YAML and propagate parsing and discovery failures.

Use PyYAML 6.0.3 as an isolated, wheel-hash-locked tooling dependency. Compose
nodes without object constructors, then inspect the graph without repeatedly
expanding aliases. The detailed contract and threat-model review are in
[workflow action pins](../repository-controls/workflow-action-pins.md).

## Authority and compatibility

This repairs enforcement of the external full-SHA-only rule. It grants no
new reference exemptions or release authority. Existing pinned workflows and
version comments remain supported. Unsupported local/Docker references stay
unsupported. Inert script and input text is no longer mistaken for an action.
Contributors must activate the parser environment before local checks or hooks;
CI provisions the same dependency. No release contracts, adapters, deployment
locks, or runtime qualification records change.

## Alternatives and consequences

Extending the grep expression cannot reliably cover YAML parsing, comments,
aliases, and duplicate keys. A handwritten YAML parser would add substantial
security-sensitive code. A dedicated Rust YAML tooling package would expand
the currently dependency-free build scaffold for a repository-only concern.
The Python helper keeps this control in `scripts/`, at the cost of a documented
tooling prerequisite and maintenance of its pinned parser.

The conformance evidence is the focused Python suite, including malicious
representation variants, legitimate current workflows, and the real shell
entrypoint's failure propagation. Parser replacement must retain these cases;
rollback must not restore the defective line matcher.
