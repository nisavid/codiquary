# Core implementation

This directory contains the single primary distribution's build placeholders:
an empty library in `lib.rs` and a non-operational `cophax` entrypoint in
`main.rs`. There is no public library API or command grammar yet.

The [substrate decision](https://github.com/nisavid/codiquary/issues/2) selects
one root Rust 2024 package and library named `codiquary`. Module layout and
the public API remain design work; add executable adapter modules only when
their owning contracts settle them. The top-level `adapters/` directory retains
its separate contract and conformance role.
