# ADR 0001: Use the hosted DCO app for contribution attestation

- Status: Accepted
- Decision scope: repository contribution admission

## Context

Human contributions require Developer Certificate of Origin 1.1 sign-off. The
repository's copied workflow requires a byte-exact author trailer and rejects
valid contribution shapes that the maintained DCO app supports. Maintaining a
custom policy also makes this repository responsible for GitHub commit-list,
retry, and merge-queue behavior.

The hosted app is a third-party repository gate. It can write Checks and reads
repository contents, metadata, merge queues, pull requests, and organization
membership. Its public source does not establish which revision is deployed or
that the service is available.

## Decision

Use hosted [DCO app registration 1861](https://github.com/apps/dco) with its
published defaults. The reviewed behavior is pinned to
[`dcoapp/app@822df17d83077098d659f4f673b18deba5d7405e`](https://github.com/dcoapp/app/tree/822df17d83077098d659f4f673b18deba5d7405e).

For ordinary pull requests, merge commits and commits whose GitHub-associated
author has type `Bot` are exempt. The Bot exemption is not limited to
Dependabot and does not authenticate the pusher. Other commits require a
`Signed-off-by` trailer whose name and email case-insensitively and
independently match the commit author or committer fields. Requiring sign-off
on every human-authored commit remains contributor guidance even where the app
exempts a merge commit.

Members require sign-off by default. Individual and third-party remediation
remain disabled. A user with write access may manually approve a failure, and
the resulting successful check records that approval; migration qualification
does not exercise this override. Merge-group synthesized commits use a separate
evaluation path. Codiquary's observed repository rules do not enable merge
queues.

The app's `DCO` check is contribution-attestation authority only. It is not
dependency verification, authenticated release approval, or production
authority.

## Consequences

Checks-write access lets the hosted service affect merge eligibility. Binding
the required context to integration 1861 limits same-name impersonation, while
source review, qualification of real check runs, and preservation of all other
repository protections remain separate controls. The pinned upstream suite
passed 4 suites, 287 tests, and 13 snapshots, but that establishes source
fixture behavior only. Third-party processing and service availability are
accepted operational risks; an unavailable or unqualified check stops the
migration rather than creating a bypass.

The migration changes no release contract, implementation, fixture, runtime
conformance surface, policy profile, deployment lock, or qualification record.
The focused compatibility and rollout contract lives in
[DCO repository control](../repository-controls/dco.md). Immutable rollout
receipts and completion state live on the
[owning maintenance issue](https://github.com/nisavid/codiquary/issues/40), not
in this decision record.

## Alternatives

Self-hosting the upstream app would add deployment ownership without an
established requirement. Keeping the bespoke script would retain narrower,
incompatible matching and its maintenance burden. An established need for
offline operation, a qualified deployed revision, or different contribution
semantics would reopen this choice.
