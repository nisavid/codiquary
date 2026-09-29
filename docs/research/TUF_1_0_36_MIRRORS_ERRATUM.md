# TUF 1.0.36 optional mirrors metadata

Assessment date: 2026-09-29.

The [tagged TUF 1.0.36 specification](https://github.com/theupdateframework/specification/blob/v1.0.36/tuf-spec.md)
defines four fundamental top-level roles: root, targets, snapshot, and
timestamp. It also defines an optional mirrors role and optional `/mirrors.EXT`
metadata. An application may hard-code mirror information instead. If it uses a
mirror list without specifying a mirrors role, that list need not be signed by
that role.

The [retained conformance report](CRYPTO_RELEASE_OPS_TUF_CONFORMANCE.md)
describes the repository surface as four logical top-level metadata files plus
delegated-targets metadata. That is incomplete as an exhaustive inventory
because it omits optional mirrors metadata. The four required files it names
remain required. The report preserves its 2026-09-07 source text; this
separately authored correction does not revise it.

The omission does not change the report's finding about the historical
`io.nisavid.release-trust/v1` design. That design lacked the four mandatory TUF
metadata roles and the TUF client update algorithm; optional mirrors metadata
cannot supply them.

The later [conditional TUF adoption decision](https://github.com/nisavid/codiquary/issues/18#issuecomment-5624996832)
retains four distinct required roles and makes TUF the sole current-release
authority plane only after qualification. It leaves optional mirrors-role use
unspecified. The accepted [first exact-target increment](https://github.com/nisavid/codiquary/issues/35#issuecomment-5691918154)
and [adaptation proof](https://github.com/nisavid/codiquary/issues/39#issuecomment-5692711096)
specify no signed-mirrors input or case. This clarification changes no accepted
Codiquary contract or selected conformance case; the [public conformance workstream](https://github.com/nisavid/codiquary/issues/26)
remains open on its existing scope.
