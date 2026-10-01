# Tests

Repository-policy checks live under `scripts/` during the language-neutral
genesis stage. Implementation tests will be colocated according to the selected
substrate while preserving clear ownership by contract, core, adapter,
conformance, and qualification surface.

A test describes only the behavior and externally oriented contract it proves.
The current library test checks that every predefined case has a nonempty
identifier and description and that identifiers are unique. It does not run the
`cophax` executable, evaluate a case, or establish conformance, qualification,
or production evidence.
