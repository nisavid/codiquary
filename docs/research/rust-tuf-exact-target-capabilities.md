# Rust TUF capability evidence for the exact-target contract

As of the public evidence retrieved on 2026-09-16, neither maintained Rust TUF
release directly implements Codiquary's accepted cryptographic profile. Tough
0.24.0 is the leading evidenced adaptation path because the retained prototype
demonstrated its narrow verification seam with Sequoia OpenPGP 2.4.1. That is a
feasibility result, not a dependency selection or runtime qualification. The
alternative `tuf` 0.3.0-beta14 line has useful native seams, including an
explicit verification-time input, but adding the accepted algorithm has not
been prototyped and crosses more of its crypto model.

A dependency decision therefore remains necessary. Selecting Tough means
owning or upstreaming a security-sensitive patch and accepting Sequoia's
backend and license consequences. Selecting `tuf` means first proving a larger,
currently unmeasured cryptographic change against an unstable prerelease API.
The stable `tuf` 0.2.0 release is not a viable candidate for this contract.

## Scope and evidence boundary

This report evaluates the exact contract accepted in
[Choose the first executable release-conformance increment](https://github.com/nisavid/codiquary/issues/35#issuecomment-5691918154)
without reopening
[Choose TUF adoption and contract mapping before Codiquary's first public release](https://github.com/nisavid/codiquary/issues/18#issuecomment-5624996832)
or
[Define qualification authority and lifecycle](https://github.com/nisavid/codiquary/issues/27#issuecomment-5626157832).
The contract requires:

- the four canonical top-level TUF roles and no delegated-target path;
- RFC 9980 algorithm 30, ML-DSA-65 + Ed25519, with the accepted SHA-512
  profile;
- ordinary TUF key IDs for role authorization, distinct issuer OpenPGP v6
  fingerprints, and normal TUF thresholds;
- one versioned, closed binding with exactly `product`, `version`, `channel`,
  `purpose`, `platform`, and `policy_profile`;
- exactly one matching top-level target before any target acquisition;
- authenticated length and digest verification before one held byte object is
  created, with no reacquisition before the helper observes that same object
  once;
- injectable in-memory transport and an explicit test clock, neither exposed
  as a production weakening;
- fail-closed handling of missing, malformed, mismatched, or unknown required
  security data, including duplicate JSON members in the signed binding.

The evidence classes are deliberately separate:

- **Published-package evidence** is a checksum-verified crates.io archive plus
  registry metadata. Where `.cargo_vcs_info.json` exists, the report maps it to
  the exact source revision.
- **Immutable source evidence** describes code at the linked commit. It does
  not show that Codiquary built or ran that code.
- **Development-head evidence** is a maintenance signal and a view of
  unreleased code. It is not transferred to a published release.
- **Prototype evidence** reports only what the retained, immutable prototype
  records showed. Those snapshots were not executed in this research pass and
  do not qualify the current contract, platforms, or dependency graph.
- **Inference** identifies the smallest integration or source change suggested
  by the source shape. It is not executable proof.

The supplied Context7 excerpts were current documentation-discovery inputs.
Every material capability below is corroborated in immutable source. No
archive, example, prototype, candidate, build, or dependency resolver was run.

## Candidate identity and maintenance

| Candidate | Published identity | Immutable release source | Maintenance evidence at retrieval | Status for this contract |
| --- | --- | --- | --- | --- |
| [Tough 0.24.0](https://crates.io/crates/tough/0.24.0) | Published 2026-07-10; checksum-verified package; `MIT OR Apache-2.0` | `.cargo_vcs_info.json` maps the package to [`98d8eb8b2ce63515d9b4981c938ef6453c5b5771`](https://github.com/awslabs/tough/tree/98d8eb8b2ce63515d9b4981c938ef6453c5b5771) | The retrieved, non-archived development repository was at that same commit | Maintained and the best-evidenced adaptation path; not conforming unmodified |
| [`tuf` 0.3.0-beta14](https://crates.io/crates/tuf/0.3.0-beta14) | Published 2025-10-20; checksum-verified package; Rust 1.80; `MIT/Apache-2.0` | `.cargo_vcs_info.json` maps the package to [`8e60df7e6adc95cffe4b4bc9a913b924dc82d860`](https://github.com/theupdateframework/rust-tuf/tree/8e60df7e6adc95cffe4b4bc9a913b924dc82d860) | The retrieved, non-archived repository was at [`219ca7d05818d5dd44ec4e32ab47c5ce64a7bf44`](https://github.com/theupdateframework/rust-tuf/tree/219ca7d05818d5dd44ec4e32ab47c5ce64a7bf44), an unreleased 0.3.0-beta15 line requiring Rust 1.88 | Current prerelease line, explicitly API-unstable, and not conforming unmodified |
| [`tuf` 0.2.0](https://crates.io/crates/tuf/0.2.0) | Published 2017-07-06 and last updated 2017-11-30; checksum-verified package | No `.cargo_vcs_info.json`; a release-to-commit mapping is not established | Its age alone says nothing about beta14, but this stable package lacks target custom data and the required algorithm-30 profile | Disqualified |
| [Sequoia OpenPGP 2.4.1](https://crates.io/crates/sequoia-openpgp/2.4.1) | Published 2026-07-09; checksum-verified package; Rust 1.85; `LGPL-2.0-or-later` | `.cargo_vcs_info.json` maps the package to [`0b0c8c7f038b829de2da0d28a822941d8600f3ee`](https://gitlab.com/sequoia-pgp/sequoia/-/tree/0b0c8c7f038b829de2da0d28a822941d8600f3ee) | A newer development revision was retrieved, but its archive state was unknown and its features are not attributed to 2.4.1 | Maintained crypto companion, not a TUF implementation |

Registry and repository metadata were retrieved at
`2026-09-16T04:19:40.729393+00:00`; beta14's separate registry record was
retrieved at `2026-09-16T04:32:54.212833+00:00`; Sequoia's corrected repository
record was retrieved at `2026-09-16T04:21:35.884139+00:00`. Null metadata is
treated as unknown.

## Capability matrix

`Direct` means the published package exposes the needed primitive. `Wrapper`
means a narrow Codiquary-owned check can enforce the accepted contract without
changing the dependency. `Source change` means the dependency itself must be
patched, forked, or changed upstream. `Absent` means the published candidate
cannot meet the requirement on the available evidence. `Not established`
means the inspected evidence cannot show that the candidate enforces the
requirement. Matrix entries do not constitute runtime qualification.

| Accepted requirement | Tough 0.24.0 | `tuf` 0.3.0-beta14 | `tuf` 0.2.0 |
| --- | --- | --- | --- |
| Authenticate root, timestamp, snapshot, and targets; exclude delegations | Wrapper | Wrapper | Direct for its older four-role model |
| RFC 9980 algorithm 30 and accepted SHA-512 profile | Source change, with retained prototype evidence | Source change, no prototype evidence | Absent |
| TUF key IDs, distinct v6 issuer fingerprint, and thresholds | Source change for the new key form; native threshold machinery remains | Source change for the new key form; native threshold machinery remains | Source change; native threshold machinery alone is insufficient |
| Closed, versioned six-field target binding; reject missing, malformed, mismatched, and unknown required fields | Wrapper over opaque target `custom` after map deserialization | Wrapper over opaque target `custom` after map deserialization | Absent |
| Reject duplicate JSON members in the signed binding | Not established; needs a pre-map source-boundary proof or change | Not established; needs a pre-map source-boundary proof or change | Absent |
| Exactly one matching top-level target before target acquisition | Wrapper | Wrapper | Absent |
| Verify exact length and digest, then hold one byte object without reacquisition | Wrapper | Wrapper | Not established |
| Injectable custom or in-memory transport | Direct | Direct | Direct repository abstraction and ephemeral repository |
| Explicit test clock with no production override | Source change | Wrapper around a public time-taking API | Absent |

### Tough 0.24.0

Tough loads and verifies all four top-level roles, including consistent-snapshot
behavior, and exposes their signed top-level metadata
([loader](https://github.com/awslabs/tough/blob/98d8eb8b2ce63515d9b4981c938ef6453c5b5771/tough/src/lib.rs#L174-L246),
[role loading](https://github.com/awslabs/tough/blob/98d8eb8b2ce63515d9b4981c938ef6453c5b5771/tough/src/lib.rs#L335-L445)).
Targets carry authenticated length, hashes, and opaque custom fields
([schema](https://github.com/awslabs/tough/blob/98d8eb8b2ce63515d9b4981c938ef6453c5b5771/tough/src/schema/mod.rs#L387-L456)).
Codiquary can therefore inspect only `targets.signed.targets`, reject any
nonempty delegation structure, deserialize a closed versioned custom object,
and require exactly one six-field match before calling `read_target`. This is a
wrapper responsibility; Tough itself permits delegations and additional
metadata fields.

That wrapper sees `custom` only after Tough has deserialized it into a
`HashMap<String, Value>`. The inspected source does not establish rejection of
duplicate JSON member names before that map is constructed, and a
post-deserialization wrapper cannot observe members the map representation no
longer distinguishes. It can still reject missing, malformed, mismatched, and
unknown binding fields and count distinct eligible target entries. Duplicate
binding members therefore remain a separate, not-established capability.

Tough's native verifier canonicalizes the signed role, authorizes signatures
by TUF key ID, deduplicates signatures, and enforces role thresholds
([verification](https://github.com/awslabs/tough/blob/98d8eb8b2ce63515d9b4981c938ef6453c5b5771/tough/src/schema/verify.rs#L8-L58)).
That duplicate-signature behavior is independent of duplicate members inside
the signed target binding and does not close the map-deserialization gap.
Its closed key enum and verifier support RSA, Ed25519, and ECDSA only. A TUF key
ID is the SHA-256 digest of the canonical key object
([key model](https://github.com/awslabs/tough/blob/98d8eb8b2ce63515d9b4981c938ef6453c5b5771/tough/src/schema/key.rs#L35-L81),
[key ID and algorithms](https://github.com/awslabs/tough/blob/98d8eb8b2ce63515d9b4981c938ef6453c5b5771/tough/src/schema/key.rs#L141-L189)).
The accepted OpenPGP algorithm and its distinct issuer fingerprint cannot be
added by a consumer wrapper; this is a security-sensitive source change.

The retained prototype at
[`fddd7f564737ed5d213061a0c8621141bd85a2e9`](https://github.com/nisavid/codiquary/blob/fddd7f564737ed5d213061a0c8621141bd85a2e9/prototypes/tuf-rust-pq/evidence/slice-3-red-green.md#L68-L110)
records a two-file Tough patch using Tough 0.24.0 and Sequoia OpenPGP 2.4.1. It
reports successful positive and negative verification for algorithm 30,
OpenPGP v6, SHA-512, a distinct issuer fingerprint, composite-component
failures, and threshold behavior. A later compatibility record reports 15
prototype tests and 78 Tough tests on Linux with OpenSSL 3.6.3
([compatibility record](https://github.com/nisavid/codiquary/blob/fddd7f564737ed5d213061a0c8621141bd85a2e9/prototypes/tuf-rust-pq/evidence/current-compatibility.md#L96-L133)).
Those are
retained observations from that revision, not tests of this report's candidate,
the accepted exact-target contract, macOS, or today's resolved dependency
graph.

Tough's public transport trait supports a purpose-built in-memory transport
([transport](https://github.com/awslabs/tough/blob/98d8eb8b2ce63515d9b4981c938ef6453c5b5771/tough/src/transport.rs#L17-L49)).
Its target reader applies a maximum length and SHA-256 digest adapter, and the
digest failure is returned only when the stream reaches EOF
([target fetch](https://github.com/awslabs/tough/blob/98d8eb8b2ce63515d9b4981c938ef6453c5b5771/tough/src/cache.rs#L268-L327),
[EOF verification](https://github.com/awslabs/tough/blob/98d8eb8b2ce63515d9b4981c938ef6453c5b5771/tough/src/io.rs#L30-L97)).
The library itself warns that bytes may be exposed before the checksum is known
([reader contract](https://github.com/awslabs/tough/blob/98d8eb8b2ce63515d9b4981c938ef6453c5b5771/tough/src/lib.rs#L448-L499)).
The safe integration is to consume the stream completely into a private
buffer, accept it only after EOF verification succeeds, independently require
`buffer.len()` to equal the signed length, and then create the one held byte
object. No helper or target path may observe the streaming buffer.

The retained consumer prototype at
[`27bedbfffbc0d4a8f3e51c89289078c2be2a3480`](https://github.com/nisavid/codiquary/blob/27bedbfffbc0d4a8f3e51c89289078c2be2a3480/prototypes/tuf-rust-pq/evidence/consumer-transport.md#L85-L148)
records custom-transport request counting, held verified bytes, and a hostile
one-byte target response. It is relevant feasibility and ordering evidence,
but it predates the accepted exact-target contract and was not executed here.

The remaining noncrypto source gap is time. Tough's datastore obtains
`Timestamp::now()` inside a private `system_time` method
([datastore](https://github.com/awslabs/tough/blob/98d8eb8b2ce63515d9b4981c938ef6453c5b5771/tough/src/datastore.rs#L86-L119)).
An explicit test clock that cannot weaken the production entry point therefore
requires a small dependency change or another upstream-supported seam; it is
not currently injectable from a wrapper.

### `tuf` 0.3.0-beta14

The beta14 release has the same useful high-level primitives: authenticated
top-level role access, opaque target custom data, inspectable delegations, a
public repository-provider trait, and verified target readers
([top-level access](https://github.com/theupdateframework/rust-tuf/blob/8e60df7e6adc95cffe4b4bc9a913b924dc82d860/tuf/src/database.rs#L190-L213),
[target schema](https://github.com/theupdateframework/rust-tuf/blob/8e60df7e6adc95cffe4b4bc9a913b924dc82d860/tuf/src/metadata.rs#L1640-L1744),
[targets and delegations](https://github.com/theupdateframework/rust-tuf/blob/8e60df7e6adc95cffe4b4bc9a913b924dc82d860/tuf/src/metadata.rs#L1867-L1914),
[repository interface](https://github.com/theupdateframework/rust-tuf/blob/8e60df7e6adc95cffe4b4bc9a913b924dc82d860/tuf/src/repository.rs#L41-L74)).
The same map-visible binding checks, top-level-only selection, exactly-one
rule, and held-buffer rules can be enforced by a wrapper.

Beta14 likewise deserializes target `custom` into a `BTreeMap` without using a
duplicate-rejecting map adapter
([POUF1 shim](https://github.com/theupdateframework/rust-tuf/blob/8e60df7e6adc95cffe4b4bc9a913b924dc82d860/tuf/src/pouf/pouf1/shims.rs#L510-L516)).
The post-deserialization wrapper can enforce the binding shape it receives,
but the inspected API cannot establish that duplicate JSON members were
rejected before map construction. Beta14 separately deduplicates signature key
IDs while enforcing authorized-key thresholds
([signature verification](https://github.com/theupdateframework/rust-tuf/blob/8e60df7e6adc95cffe4b4bc9a913b924dc82d860/tuf/src/verify.rs#L85-L166));
that is not duplicate-binding-member evidence.

Its crypto model is nevertheless closed around Ed25519. Signature scheme and
key type have unknown sentinels, but verification accepts only Ed25519
([scheme](https://github.com/theupdateframework/rust-tuf/blob/8e60df7e6adc95cffe4b4bc9a913b924dc82d860/tuf/src/crypto.rs#L277-L325),
[key type](https://github.com/theupdateframework/rust-tuf/blob/8e60df7e6adc95cffe4b4bc9a913b924dc82d860/tuf/src/crypto.rs#L354-L380),
[verification](https://github.com/theupdateframework/rust-tuf/blob/8e60df7e6adc95cffe4b4bc9a913b924dc82d860/tuf/src/crypto.rs#L625-L662)).
Adding algorithm 30 would cross scheme decoding, key decoding, public-key
verification, encoding, and their tests. No retained prototype demonstrates
that path. Inspection of the unreleased beta15 development revision found the
[same Ed25519-only boundary](https://github.com/theupdateframework/rust-tuf/blob/219ca7d05818d5dd44ec4e32ab47c5ce64a7bf44/tuf/src/crypto.rs#L280-L380),
so there is no evidence that this gap has closed; that development observation
is not a beta14 release claim.

Beta14 has an explicit `update_with_start_time` entry point and equivalent
target methods
([client time seam](https://github.com/theupdateframework/rust-tuf/blob/8e60df7e6adc95cffe4b4bc9a913b924dc82d860/tuf/src/client.rs#L417-L437),
[target fetch](https://github.com/theupdateframework/rust-tuf/blob/8e60df7e6adc95cffe4b4bc9a913b924dc82d860/tuf/src/client.rs#L840-L874)).
The source warns that an old caller-supplied time can enable a freeze attack.
Codiquary would have to keep that API behind a test-only boundary and expose
only the real-clock entry point in production. Its target adapter, like
Tough's, treats signed length as a maximum and validates hashes at EOF
([reader implementation](https://github.com/theupdateframework/rust-tuf/blob/8e60df7e6adc95cffe4b4bc9a913b924dc82d860/tuf/src/util.rs#L96-L177));
the wrapper must consume privately, require exact length, and hold only the
post-verification bytes.

The project's own release documentation describes the beta line as active
development with an unstable API that may be unsuitable for production. The
newer retrieved head declares beta15, Rust 1.88, and edition 2024. That is a
real maintenance signal, but it also raises migration cost relative to the
published beta14 package and must not be silently treated as beta14 behavior.

### `tuf` 0.2.0

The checksum-verified stable archive predates the current prerelease design.
Its target description contains only length and hashes, with no custom field
([package source](https://docs.rs/crate/tuf/0.2.0/source/src/metadata.rs#L825-L951));
its crypto model supports Ed25519, RSA-PSS/SHA-256, and RSA-PSS/SHA-512, but
not algorithm 30
([schemes](https://docs.rs/crate/tuf/0.2.0/source/src/crypto.rs#L134-L179),
[verification](https://docs.rs/crate/tuf/0.2.0/source/src/crypto.rs#L525-L531));
and metadata expiry calls the wall clock directly
([package source](https://docs.rs/crate/tuf/0.2.0/source/src/tuf.rs#L135-L143)).
It does expose a repository trait and an ephemeral repository
([package source](https://docs.rs/crate/tuf/0.2.0/source/src/repository.rs#L24-L70)),
but that isolated capability cannot recover the missing signed binding and
cryptographic profile. Because its archive does not identify a source
revision, these versioned package-source observations cannot be linked to an
exact Git commit. It is disqualified on capability, not merely on release age,
and its shortcomings are not attributed to the beta14 line.

### Sequoia OpenPGP 2.4.1

Sequoia is the only inspected published component that directly models
algorithm 30. It maps public-key algorithm ID 30 to ML-DSA-65 + Ed25519
([algorithm mapping](https://gitlab.com/sequoia-pgp/sequoia/-/blob/0b0c8c7f038b829de2da0d28a822941d8600f3ee/openpgp/src/crypto/types/public_key_algorithm.rs#L208-L266))
and requires both signature components to verify
([composite verification](https://gitlab.com/sequoia-pgp/sequoia/-/blob/0b0c8c7f038b829de2da0d28a822941d8600f3ee/openpgp/src/packet/key.rs#L1895-L1979)).
It represents v6 fingerprints as 32 bytes
([fingerprint](https://gitlab.com/sequoia-pgp/sequoia/-/blob/0b0c8c7f038b829de2da0d28a822941d8600f3ee/openpgp/src/fingerprint.rs#L138-L200)).
That is the issuer identity input; it must remain distinct from the TUF key ID
that authorizes a role signature.

Backend selection is a release and security constraint, not a build detail.
Sequoia requires the leaf application to select exactly one backend. The
default Nettle backend reports ML-DSA unsupported
([Nettle](https://gitlab.com/sequoia-pgp/sequoia/-/blob/0b0c8c7f038b829de2da0d28a822941d8600f3ee/openpgp/src/crypto/backend/nettle/asymmetric.rs#L32-L49)).
The OpenSSL backend supports it only with the OpenSSL 3.5 API or newer
([OpenSSL](https://gitlab.com/sequoia-pgp/sequoia/-/blob/0b0c8c7f038b829de2da0d28a822941d8600f3ee/openpgp/src/crypto/backend/openssl/asymmetric.rs#L142-L159)).
The RustCrypto backend has ML-DSA support, but Sequoia classifies that backend
as experimental, not production-ready, and variable-time
([implementation](https://gitlab.com/sequoia-pgp/sequoia/-/blob/0b0c8c7f038b829de2da0d28a822941d8600f3ee/openpgp/src/crypto/backend/rust/asymmetric.rs#L67-L82),
[backend policy](https://gitlab.com/sequoia-pgp/sequoia/-/blob/0b0c8c7f038b829de2da0d28a822941d8600f3ee/openpgp/build.rs#L84-L224));
it requires explicit opt-ins. The smallest currently evidenced
production-oriented path is therefore OpenSSL 3.5 or newer, subject to
platform qualification rather than assumption.

Sequoia's `LGPL-2.0-or-later` license is materially different from the TUF
candidates' permissive licenses. Before distribution, the maintainer needs a
license-compliance decision covering the chosen link and distribution model,
relinkability obligations, notices, and source or patch availability. This
report does not make that legal determination.

## Standards boundary

[RFC 9980 section 2.1](https://www.rfc-editor.org/rfc/rfc9980.html#section-2.1)
assigns algorithm ID 30 to ML-DSA-65 + Ed25519 and requires it for a conforming
implementation. Its
[verification procedure](https://www.rfc-editor.org/rfc/rfc9980.html#section-5.2)
requires both components, while
[section 5.3.1](https://www.rfc-editor.org/rfc/rfc9980.html#section-5.3.1)
constrains the signature to v6 and a digest of at least 256 bits. SHA-512 is the
accepted Codiquary profile; it is not presented here as a universal RFC 9980
mandate.

[RFC 9580 section 3.3](https://www.rfc-editor.org/rfc/rfc9580.html#section-3.3)
defines OpenPGP key IDs as eight octets and warns that they are not unique.
[Section 5.5.4.3](https://www.rfc-editor.org/rfc/rfc9580.html#section-5.5.4.3)
defines the 256-bit v6 fingerprint and derives the OpenPGP key ID from its high
64 bits. Neither value substitutes for TUF's canonical-key-object key ID.

## Consequences and unresolved evidence

Tough is ahead on evidence, not selected. Its source-change seam is small and a
retained prototype reached the fixed crypto profile, but carrying that seam
would make Codiquary or an upstream maintainer responsible for a sensitive
verification boundary. `tuf` beta14 offers the better clock seam and similar
transport and target primitives, but its unproven algorithm change is broader,
its public time-taking API needs confinement, and its prerelease API can move.
Both paths still need the same contract wrapper for exact target selection,
map-visible closed custom data, exact length, private buffering, and one held
object. Neither post-deserialization wrapper is sufficient evidence for
duplicate JSON binding-member rejection; that enforcement boundary remains
unresolved.

The ownership boundary is consequential. Codiquary's security maintainer owns
the post-deserialization binding validator, selection order, exact-length
check, buffering, and production/test-clock API boundary. The selected TUF
project's maintainer owns an accepted upstream crypto or clock change;
Codiquary owns the same code and its security response if it carries a fork
instead. Ownership of duplicate-member enforcement cannot be assigned until
the pre-map source boundary is chosen. The leaf application and release
maintainer own Sequoia backend selection, native-library availability, and the
locked build. The distribution maintainer owns license compliance, and the
compatibility maintainer owns Linux and GitHub-hosted macOS qualification.
No inspected path establishes a need for unsafe code, and this report selects
neither a parallel TUF parser nor a dependency source change for duplicate
members. Either would be a new security and maintenance boundary requiring
review.

Toolchain and dependency consequences are not yet resolver-qualified. Tough
0.24.0 declares Rust 2018 but no minimum Rust version in its published
manifest and uses `aws-lc-rs`; composing it with Sequoia makes Rust 1.85 and the
chosen Sequoia backend the known floors. Beta14 declares Rust 1.80 and uses
`ring`, but the same composition again raises the known Rust floor to 1.85 and
adds a second cryptographic implementation boundary. Choosing the unreleased
beta15 revision instead raises the declared floor to Rust 1.88. OpenSSL-backed
algorithm 30 adds an OpenSSL 3.5-or-newer native dependency on both paths. No
current lockfile, transitive dependency tree, minimum-version build, or native
link result was produced in this research pass.

The following evidence is still missing:

- an operator decision between maintaining or upstreaming the evidenced Tough
  change and funding a new `tuf` crypto adaptation proof;
- an explicit Sequoia backend and license-compliance decision;
- a resolver-produced, locked dependency graph and minimum-supported Rust
  version for the chosen composition;
- a current build and conformance run at exact dependency commits and backend
  versions;
- Linux and GitHub-hosted macOS qualification, including availability and
  behavior of the selected OpenSSL implementation;
- proof that every zero-target, multi-target, six-field mismatch, delegation,
  malformed or unknown required-field case fails before the transport receives
  a target request;
- proof that signed bindings with duplicate JSON member names, using both equal
  and conflicting values, are rejected before map construction can make the
  duplicate unobservable and before the transport receives a target request;
- proof that digest and exact length complete before byte publication, the
  helper sees the same held allocation and bytes once, and no second target
  acquisition occurs;
- proof that a fixed clock is available only to tests and cannot be selected by
  the production entry point; and
- upstream maintenance intent for either source change. Repository activity
  alone does not establish willingness to accept or sustain this profile.

The smallest duplicate-member source-boundary question is: “Does the candidate
reject duplicate member names while parsing signed target `custom` data,
before constructing its `HashMap` or `BTreeMap`; if not, at what authenticated
parse boundary can rejection be enforced, and does that require a dependency
source change or a separately reviewed parser boundary?” The current map-based
APIs do not answer that question.

If upstream intent is needed before the decision, the minimal public questions
are: “Would Tough accept an extensible verifier/key representation for RFC 9980
algorithm 30 plus an injectable verification-time provider restricted from its
default production path?” and “Would rust-tuf accept algorithm 30 and separate
issuer-fingerprint plumbing in its current beta line?” The answers resolve
maintenance ownership; they do not replace the conformance proof.

## Smallest future proof

After the operator chooses which source-change risk to investigate, build one
disposable, lockfile-pinned exact-target-v1 proof around that candidate and
Sequoia 2.4.1 with an explicitly selected backend. Do not broaden it into an
updater or release system. The proof is complete only when the same revision:

1. verifies all four canonical roles with algorithm 30, SHA-512, ordinary TUF
   key IDs, distinct v6 issuer fingerprints, both composite components, and
   thresholds, including duplicate and unauthorized signature failures;
2. rejects delegations and enforces missing, malformed, mismatched, and unknown
   fields in the closed, versioned six-field binding;
3. rejects signed bindings containing duplicate JSON member names, with equal
   and conflicting values, at a proven pre-map boundary before any target
   fetch;
4. rejects zero and multiple matching targets and every single-field mismatch
   before any target fetch;
5. uses a counting in-memory transport to prove one acquisition, including a
   hostile one-byte-equivocation case;
6. consumes target bytes privately through EOF, checks signed digest and exact
   length, creates one held byte object, and proves the helper observes that
   object once without reacquisition;
7. uses a fixed time in tests while the production entry point has no
   caller-controlled clock; and
8. passes the candidate's relevant upstream tests and the new conformance cases
   on Linux and GitHub-hosted macOS with the exact toolchain, lockfile, crypto
   backend, and native-library versions recorded.

The retained prototypes can supply test ideas, but they do not satisfy this
proof because they predate the accepted exact-target increment and were not
rerun against its final contract. Final dependency selection should follow,
not precede, this decision and proof.
