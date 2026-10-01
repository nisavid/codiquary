# Fixtures

`conformance/exact-target-v1/` contains public, inert inputs for the disposable
Tough exact-target proof. Its current corpus covers one exact-match case with
four synthetic public role certificates, signed metadata, one target, a
six-field selection, and expected observations.

[`manifest.json`](conformance/exact-target-v1/manifest.json) binds the fixture
generation revision, source and patch identities, input checksums, fixed
verification instant, and expected results. `inputs.sha512` projects those input
checksums for independent byte verification. The author lives in
[`proofs/exact-target-tough/`](../proofs/exact-target-tough/).

The public output passed static inspection. Signature authentication and
observed request, helper, and allocation behavior still require the executable
proof. These experimental fixtures do not establish normative conformance,
platform qualification, or release authority.

Fixtures must remain disposable and free of serialized secret keys, production
identifiers, credentials, provider payloads, and captured private evidence.
