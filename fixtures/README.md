# Fixtures

This directory will contain disposable, value-free canonical and hostile
fixtures for contracts, lifecycle transitions, adapter outcomes, publication,
verification, withdrawal, freeze, recovery, and protected-consumer handoffs.

Fixtures cannot contain production identifiers, secrets, provider payloads, or
captured private evidence. The accepted disposable corpus is defined in
`src/lib.rs` and exercised with `cophax conformance`; serialized hostile
vectors will be added when the owning contracts settle their wire shapes.
