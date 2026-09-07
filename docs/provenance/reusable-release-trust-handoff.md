# Reusable release-trust handoff

This repository begins from reviewed public design work in
[`nisavid/dotfiles`](https://github.com/nisavid/dotfiles). Those decisions are
inputs, not production state and not authority to access a private deployment.

## Controlling decisions

- [System threat and authority model](https://github.com/nisavid/dotfiles/issues/215#issuecomment-5464747293)
- [Release authority topology and continuity](https://github.com/nisavid/dotfiles/issues/206#issuecomment-5465835347)
- [Reusable core, adapter, and policy-profile boundary](https://github.com/nisavid/dotfiles/issues/207#issuecomment-5470521497)
- [Release-signer custody, recovery, and factor rotation](https://github.com/nisavid/dotfiles/issues/208#issuecomment-5472198422)
- [Signed release manifest and independent admission](https://github.com/nisavid/dotfiles/issues/209#issuecomment-5474608513)
- [Client trust bootstrap, refresh, and verification state](https://github.com/nisavid/dotfiles/issues/210#issuecomment-5474715429)
- [Cloudflare canonical publication and Vercel boundary](https://github.com/nisavid/dotfiles/issues/204#issuecomment-5469266482)
- [Minimal release operator surface](https://github.com/nisavid/dotfiles/issues/216#issuecomment-5500303338)
- [Public documentation architecture](https://github.com/nisavid/dotfiles/issues/211#issuecomment-5500606485)
- [Generic consumer updater and protected-launcher boundary](https://github.com/nisavid/dotfiles/issues/244#issuecomment-5531282859)
- [Operational delivery and first-adopter boundary](https://github.com/nisavid/dotfiles/issues/259#issuecomment-5532496281)

## Ownership transfer

Codiquary owns the generic public contracts, core implementation,
adapter interfaces and maintained reference adapters, public policy profiles,
fixtures, conformance assets, qualification formats and records,
documentation, governance, compatibility declarations, and releases.

Dotfiles remains the first system-user repository. It owns exact profile and
version locks, private and encrypted deployment bindings, real hosts and
accounts, provider resources, opaque secret references, protected state,
ceremonies, production mutations, and acceptance evidence. The operational
coordinator tickets remain in dotfiles and consume immutable public outputs
from this repository.

## Migration receipts

The genesis work must identify each imported contract or executable artifact by
its source decision and immutable source revision, state whether it was copied,
translated, or reimplemented, and bind its conformance evidence to the final
repository revision. A link or matching name is not a migration receipt.
