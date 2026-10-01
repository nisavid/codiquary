# Core implementation

This directory contains the current descriptive scaffold for the single primary
distribution. `lib.rs` exposes public, value-free types and a predefined
expected-case inventory. `main.rs` prints that inventory for the current
`cophax conformance` invocation. The command does not evaluate inputs or produce
observed conformance, and its grammar is not selected.

The [substrate decision](https://github.com/nisavid/codiquary/issues/2) selects
one root Rust 2024 package and library named `codiquary`. Further module layout
and the normative public API remain design work; add executable adapter modules
only when their owning contracts settle them. The top-level `adapters/`
directory retains its separate contract and conformance role.
