# Domain documentation

This is a single-context repository. Read root `CONTEXT.md` for the canonical
model and invariants. System-wide architectural decisions live under
`docs/adr/`; reusable contract sources will live under `contracts/`.

Do not introduce a second bounded context merely to mirror an adapter or
directory. Record a real ownership, privilege, platform, dependency, or release
boundary first.
