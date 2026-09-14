# Conformance

Conformance suites will test normative behavior independently from ownership,
qualification, support, and production acceptance. They must cover positive,
negative, interoperability, interruption, possible-effect, reconciliation, and
recovery paths.

The first dependency-free corpus is exposed by `cophax conformance`. Each line
is `case-id<TAB>result<TAB>description`, making the output suitable for scripts
and review logs. The cases are intentionally descriptive until the owning wire
and command contracts freeze their serialized shapes.

Run it with:

```sh
cargo run --frozen -- conformance
```

The corpus covers an accepted exact-byte release, rollback rejection, a
blocking suspension, and an indeterminate incomplete history. It is public,
deterministic, and contains no provider, credential, or production data.

Documentation must refer to this command's output rather than restating
expected results. Qualification records and release authority remain separate
from this conformance runner.
