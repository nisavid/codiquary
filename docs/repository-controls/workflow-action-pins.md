# Workflow action pins

`./scripts/check-repository.sh` validates every `.yml` and `.yaml` file directly
inside `.github/workflows`. The same entrypoint is used locally, by the
pre-push hook, and by repository-policy CI. `scripts/test_workflow_pins.py`
owns the regression and compatibility checks.

## Reference policy

Every `jobs.<job>.uses` and `jobs.<job>.steps[*].uses` value, including actions
inside `parallel` step groups, must name an
external repository action or reusable workflow pinned to exactly 40
lowercase hexadecimal characters. Owners begin with an ASCII letter or digit
and contain only ASCII letters, digits, and hyphens. Repository and optional
subpath components may contain ASCII letters, digits, underscores, dots, and hyphens.
No branch, tag, expression, suffix, or trailing whitespace is permitted.
Exact `.` and `..` path components are rejected so action subpaths cannot
escape the pinned repository's directory.

Local (`./`) actions and workflows, Docker references, and other reference
forms are unsupported. This preserves the existing external-SHA-only policy;
supporting another reference kind requires its own integrity policy.

## YAML handling

The validator composes a PyYAML representation graph using `BaseLoader`,
without constructing Python objects or applying YAML 1.1 scalar coercions.
Quoted and escaped keys, flow mappings, explicit mapping syntax, block
scalars, and nonrecursive aliases are inspected after decoding. Comments do
not supply reference content. Block scalar values must themselves satisfy the
entire reference rule, including any final newline.

Duplicate mapping keys, merge keys, non-scalar mapping keys, nonstandard
tags, explicit scalar tags other than `!!str`, recursive aliases, malformed
or multiple documents, and invalid types along the workflow/job/step path
fail closed. Empty or missing workflow directories, unreadable files,
invalid UTF-8, and workflow symlinks also fail. This is a deliberately limited
policy parser, not a complete GitHub Actions schema validator.

Mapping keys containing `${{` are unsupported, including string-literal
expressions. Actions can normalize these into structural keys after YAML
parsing; rejecting them prevents a later template pass from revealing a
reference the policy did not inspect. Ordinary expressions in values such as
`runs-on` remain permitted; action reference values must still be literal pins.

Keys named `uses` in `env`, `with`, or script text are data, not execution
references. A value anchored in such a location is still inspected if an
action-bearing location aliases it.

## Parser dependency and compatibility

The tooling dependency is [PyYAML 6.0.3](https://pypi.org/project/PyYAML/6.0.3/)
(MIT), maintained at [yaml/pyyaml](https://github.com/yaml/pyyaml). Its wheel
hashes from the version-specific PyPI release metadata are recorded in
`scripts/requirements-policy.txt`. The installation accepts only these
wheels, with no transitive dependencies or source build hooks. It is isolated
in a virtual environment and is not a Codiquary runtime dependency.

CI installs this environment before checks. Local setup is documented in
`CONTRIBUTING.md`; a missing interpreter or parser makes the check fail,
including in the pre-push hook. Python 3.9 or newer and a platform supported by
the pinned wheels are required. Parser updates must update the pin and wheel
hashes together and rerun the regression suite; hashes establish artifact
identity, not freedom from vulnerabilities.

All existing pinned workflows remain accepted. Formerly overlooked unpinned
references now fail. To replace the parser, preserve the representation and
failure cases in the suite; reverting to line matching is not a safe rollback.

## Threat and evidence boundary

The attacker-controlled surface is workflow YAML admitted to a policy check.
The check prevents representation differences or misleading comments from
hiding mutable action references. It does not establish who controls a pinned
commit, the safety of that commit, transitive action dependencies, or the
integrity of shell downloads. Composite-action metadata is outside this
workflow-only increment.

A contributor who can change both the workflow and its validator can change
the policy itself. Review, trusted workflow admission, and hosted repository
rules remain separate controls. A green result is repository-policy evidence,
not release, deployment, or production authority.
