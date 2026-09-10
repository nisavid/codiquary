# Consumer and transport findings

The consumer/transport experiment establishes a narrow pre-privilege composition result: for the supplied synthetic fixtures, TUF acquisition can retain verified target bytes, apply a policy decision, preserve representative native signature checks, and call a fake consumer only after the selected checks succeed. The makepkg path also demonstrates six-field application-selection substitution rejection before acquisition.

It does not decide whether Codiquary should adopt TUF or define the production contract that an adoption would require.

## The boundary under test

The experiment follows this order:

```text
application selection → TUF metadata and target verification → held bytes
→ retained policy gate → native package checks → fake consumer
```

The application selection contains product, version, channel, purpose, policy profile, and target path before acquisition. The makepkg substitution test compares each field with an independently accepted selection and rejects every mismatch before acquisition. The pacman composition uses its six-field selection to choose the target, but its tests pass the same selection into the acquisition expectations and therefore do not independently demonstrate pacman selection-substitution rejection.

Tough authenticates the eligible target description and target bytes. The experiment keeps those bytes in memory through a fake retained-policy gate and the relevant native check before passing them once to a fake consumer.

The gate and consumer demonstrate ordering and byte retention only. The gate is not a production Codiquary policy implementation, and the consumer does not install, execute, or otherwise act on the fixture.

## What the transports show

The file-backed cases use Tough's filesystem transport. The HTTP cases use a custom Tough `Transport` implementation that invokes `curl` against an isolated `127.0.0.1` fixture server and retains the observed request and response bytes.

This shows that the tested boundary is not limited to direct file reads. It does not qualify Tough's feature-gated `HttpTransport`, TLS, redirects, remote networking, authentication, mirrors, retries, proxies, or a production transport adapter. The shared acquisition negatives and integrated native failures are selected boundary cases, not an exhaustive transport/error cross-product.

## What the native fixtures show

The PKGBUILD path stages the held target in a read-only harness and runs `makepkg --verifysource`. The positive case observes both the checksum and conventional detached-signature checks. A damaged signature and a fixture that disables signature checking both stop before the fake consumer.

The pacman path uses libalpm to check one synthetic package and one synthetic repository database. It requires the configured package and database signature levels, verifies one signature result for each, checks the database package identity, refuses interactive questions, and calls the fake consumer only after the caller accepts the complete evidence.

Only the package archive is a TUF target. The database, package signature, database signature, and synthetic public certificate are independent native companions. The fixtures therefore preserve native controls without claiming that TUF authenticates every companion.

The native signatures are conventional synthetic OpenPGP signatures. They test interoperability and ordering, not native post-quantum qualification. Libalpm reports the primary certificate fingerprint in this fixture; the separately declared signing-subkey fingerprint is not independently identified by that returned field.

## What the negative cases establish

The selected tests stop the tested path when:

- any protected-selection field is substituted on the file-backed makepkg path;
- HTTP returns target bytes that disagree with authenticated metadata;
- the retained policy rejects the held bytes;
- held bytes are changed after acquisition;
- a makepkg detached signature is damaged or its check is disabled;
- a pacman package or database no longer matches its detached signature; or
- mandatory pacman package or database signature checking is disabled.

The selection-substitution result is representative evidence for the shared acquisition boundary. It is not an independently exercised pacman-selection result or an exhaustive consumer/transport cross-product.

The held-byte mutation case is specifically a post-acquisition check. The mutated bytes reach and pass the fake policy gate, are written into the read-only harness, and fail the staged artifact digest check before `makepkg` is invoked.

These cases establish the observed stop points and fake-consumer call counts. They do not establish recovery behavior, exhaustive error classification, or absence of every possible side effect. HTTP evidence retention occurs after the attempted pipeline while the fixture server is joined; in a successful attempt, a retention failure can therefore occur after the fake-consumer call.

## What remains before an adoption decision

A TUF adoption and contract-mapping decision still needs to assign production ownership for:

- the application-selection and retained-byte contract;
- trusted-root bootstrap, rotation, expiry, rollback, clocks, consistent snapshots, and metadata limits;
- release and status authority, withdrawal, recovery, and lifecycle state;
- protected-consumer admission, installation, execution, archive, and updater policy;
- transport authentication, remote failure handling, mirrors, retry rules, and retained evidence;
- policy-profile and private deployment-binding inputs;
- durable state, interruption and power-loss behavior, and reconciliation;
- production key custody, provider boundaries, publication authority, and operational review; and
- supported platforms, including macOS, plus qualification and compatibility renewal.

The experiment supplies evidence for that decision. It does not make the decision or authorize an experimental or production release.

For exact inputs and observations, consult [Consumer and transport evidence](../evidence/consumer-transport.md). For the prepared-input procedure, use [Replay the consumer and transport checks](replay-consumer-transport.md).
