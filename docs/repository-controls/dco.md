# DCO repository control

Codiquary requires human-authored commits to certify contributions under the
[Developer Certificate of Origin 1.1](https://developercertificate.org/).
Repository policy selects the hosted [DCO app](https://github.com/apps/dco) to
publish the required `DCO` check once the qualification and rules migration
below are complete. This check governs contribution admission only; it does not
authenticate a pusher, assess a dependency, approve a release, or grant
deployment or production authority.

## Contribution guidance and enforced policy

Human contributors sign off every commit they create. The selected app's
ordinary pull-request gate has distinct source-derived semantics:

- commits with more than one parent are exempt as ordinary merge commits;
- a commit whose GitHub-associated author has type `Bot` is exempt;
- every other commit needs a `Signed-off-by` trailer whose name and email
  case-insensitively and independently match the commit author or committer
  fields;
- members require sign-off by default; and
- individual and third-party remediation commits are disabled by default.

The Bot exemption is not Dependabot-specific and does not prove who pushed the
commit. A pull request containing exempt Bot commits and an unsigned human
non-merge commit still fails. The ordinary merge exemption does not change the
project's guidance to sign off human-authored commits. Merge-group synthesized
commits use a separate evaluation path; Codiquary's observed repository rules
do not enable merge queues.

The source exposes a write-access manual approval that produces a successful
check recording the approval. This capability is intentionally unexercised.
Because Codiquary is a personal repository and retains the default
`require.members: true`, organization-member exemption behavior is not
applicable.

## Evidence boundary

The reviewed source is pinned to
[`dcoapp/app@822df17d83077098d659f4f673b18deba5d7405e`](https://github.com/dcoapp/app/tree/822df17d83077098d659f4f673b18deba5d7405e).
Its isolated upstream suite passed at that revision. That source-only evidence
covers the ordinary merge and Bot exemptions, human signoff outcomes, trailer
matching and mismatch cases, membership and remediation defaults, recheck
handlers, and manual-approval output.

A separately authored four-case evaluator replay using the same pinned
`lib/dco.js` covers Bot plus unsigned User and Bot plus signed User histories in
both commit orders. The
[public fixture and run receipt](https://github.com/nisavid/sacrysty/issues/22#issuecomment-5694455454)
bind its exact bytes and results. This replay is source-function evidence, not
part of the upstream suite. Neither source layer identifies or tests the hosted
deployment.

The public registration identifies app 1861 and its permissions and event
subscriptions. It does not prove installation on Codiquary, configuration at
event time, webhook delivery, availability, deployed source revision, or real
check behavior. Immutable observations and completion state belong on the
[owning maintenance issue](https://github.com/nisavid/codiquary/issues/40), so
adding a run receipt never requires editing this policy.

## Execution prerequisite and acceptance evidence

Installation, qualification, rules mutation, and rollback must follow the
reviewed immutable Sacrysty-owned `docs/agents/dco-provisioning.md` procedure
linked from the owning issue. The issue must bind the exact reviewed procedure
revision and its review receipt before dependent execution. This page supplies
only Codiquary's policy, accepted evidence, allowed rules delta, ordering, and
rollback invariants. These execution prerequisites do not block source-only
cleanup or draft preparation.

Before switching required contexts, issue receipts must bind `DCO` from
integration 1861 to both the signed-human migration pull request and PR14's real
Bot-only state. They must also bind one hosted unsigned-human missing-trailer
rejection because source evidence does not bind the hosted deployment. Running
that intentionally unsigned remote fixture requires an explicit signoff-policy
exception limited to its one unsigned commit and must follow the reviewed shared
procedure. This requirement leaves normal contribution guidance unchanged and
creates no broader standing exception.

Together, the upstream suite and focused replay are sufficient for the source
semantics and defaults individually attributed to them above; a live replay of
every upstream branch is not required. Manual override remains intentionally
unexercised, organization-member behavior is not applicable, and
`@dcoapp recheck` is optional operational recovery evidence rather than ordinary
contribution-admission evidence.

The DCO migration lands before PR14 is refreshed. After that Bot refresh, the
signed human Cocogitto compatibility commit is added. A successful `DCO` result
on that final mixed Bot and signed-human PR14 revision is a separate recovery
gate.

## Allowed rules delta and rollback

Record normalized full repository-rules readbacks immediately before and after
the switch. Their sole semantic difference must replace `DCO compliance` from
GitHub Actions integration 15368 with `DCO` from integration 1861. Any other
rule or parameter change stops the migration. The full normalized comparison,
not a copied subset of parameters, proves that every unrelated protection was
preserved.

The new check must be qualified and required, and the old requirement removed,
before the workflow-removal change merges; there is no interval with neither
DCO gate. If the hosted check is absent, misidentified, or untrustworthy, keep
or restore the local workflow. Rollback restores an available workflow,
observes its `DCO compliance` check, and requires that context before removing
the hosted requirement. Its normalized rules readback must show only the
reverse DCO-context replacement.
