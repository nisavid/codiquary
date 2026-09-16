# Conformance

Conformance suites will test normative behavior independently from ownership,
qualification, support, and production acceptance. They must cover positive,
negative, interoperability, interruption, possible-effect, reconciliation, and
recovery paths.

The current scaffold exposes a predefined expected-case inventory through
`cophax conformance`. It prints each case as a tab-separated identifier,
expected-result label, and description. This describes the current output; it
does not freeze command grammar or a wire schema.

Run it with:

```sh
cargo run --frozen -- conformance
```

The inventory names expected outcomes for an exact-byte release, rollback,
suspension, and incomplete history. It does not evaluate inputs, invoke an
enforcing component, or produce observed conformance or qualification evidence.
It is public, deterministic, and contains no provider, credential, or
production data.

Documentation may consume this printed inventory as scaffold input without
treating it as an executable result. Qualification records, release authority,
and protected-consumer acceptance remain separate.
