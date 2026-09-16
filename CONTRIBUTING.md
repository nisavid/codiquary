# Contributing to Codiquary

Contributions should leave the public source reviewable, reproducible, and
explicit about what their evidence proves.

## Prepare a change

1. Link the issue that owns the change and claim it when the Wayfinder protocol
   applies.
2. Read `CONTEXT.md` and preserve the public/private authority boundary.
3. Update normative source before derived fixtures, reference output, or
   documentation.
4. Use a Conventional Commit message.
5. Certify each human-authored commit you create under the Developer Certificate
   of Origin 1.1 by signing it off:

   ```sh
   git commit --signoff
   ```

   The sign-off records that you have the right to submit the contribution
   under this project's license. It is not a cryptographic release signature.
   This is contribution guidance. The selected hosted DCO app's ordinary gate
   also exempts merge commits and commits whose GitHub-associated author is a
   Bot; the Bot exemption does not identify or authenticate the pusher. See
   [DCO repository control](docs/repository-controls/dco.md) for the enforced
   matching rules, evidence boundary, and rollout state.
6. Run `./scripts/check-repository.sh`, `git diff --check`, and every focused
   test or conformance suite for the changed surface.

If Cocogitto is installed, `cog install-hook --all` installs the checked-in
commit-message and pre-push hooks.

## Scope and evidence

- Keep production keys, credentials, provider resources, host bindings,
  private configuration, and acceptance evidence out of the repository.
- Use disposable, value-free fixtures. Never test an undisclosed report against
  a live deployment.
- Bind qualification claims to exact implementations, interfaces, platforms,
  dependencies, profiles, fixtures, suites, and evidence.
- Describe a green check at the layer it exercised. Repository policy is not a
  release or deployment acceptance result.
- Do not split a distribution or adapter merely to mirror a directory. Show the
  dependency, privilege, platform, or release seam first.

## Pull requests

Use the pull-request template. State the outcome, owning boundary, authorizing
issue, provenance, compatibility effect, checks run, and whether the change has
any release, publication, provider, host, or production authority. Resolve
review findings on the same final revision that carries the reported evidence.
