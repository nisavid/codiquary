# Adapters

Adapters occupy narrow, role-specific seams such as custody, signer
authorization, canonical publication, artifact hosting or mirroring, external
evidence, and verifier distribution. Selection is explicit and
capability-based. The core never scans ambient executable paths, lets a provider select
itself, or silently falls back after an indeterminate mutation.

An adapter surface is added only after real variation and conformance evidence
justify it. A secret-free fake may exercise an interface before a provider is
qualified. No adapter implementation exists yet.
