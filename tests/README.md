# Tests

Repository-policy checks live under `scripts/` during the language-neutral
genesis stage. Implementation tests will be colocated according to the selected
substrate while preserving clear ownership by contract, core, adapter,
conformance, and qualification surface.

A test describes only the behavior and externally oriented contract it proves.
The library test asserts that every fixture has a unique identifier and an
explicit result. The executable is checked through the same public corpus, so
documentation and review logs can consume deterministic output without
duplicating fixtures.
