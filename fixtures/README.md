# Fixtures

This directory will contain disposable, value-free canonical and hostile
fixtures for contracts, lifecycle transitions, adapter outcomes, publication,
verification, withdrawal, freeze, recovery, and protected-consumer handoffs.

Fixtures cannot contain production identifiers, secrets, provider payloads, or
captured private evidence. The predefined expected-case inventory in
`src/lib.rs` is printed by the current `cophax conformance` command. Its labels
are not accepted fixtures or observed outcomes; serialized hostile vectors will
be added when the owning contracts settle their wire shapes.
