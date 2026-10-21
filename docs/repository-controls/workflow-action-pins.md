# Workflow action pins

`./scripts/check-repository.sh` invokes `scripts/check_workflow_pins.py` for
repository-policy CI, the local pre-push hook, and manual validation. Install
its pinned PyYAML dependency in a Python 3.10+ virtual environment using the
commands in `CONTRIBUTING.md`. CI provisions the same dependency explicitly.
Missing dependencies, unreadable workflow inputs, and parsing failures stop
the check.

## Reference policy

Every external action in a job's steps (including `parallel` groups and
background steps) and reusable workflow at `jobs.<job>.uses` must have an `owner/repository[/path]@<sha>` reference with
exactly 40 lowercase hexadecimal SHA characters. This preserves the existing
pin requirement. Local references (`./`), self-repository references (`$/`),
Docker references (`docker://...`), and expressions (`${{ ... }}`) remain
unsupported; supporting them requires a separate policy decision, including how
their transitive dependencies are checked. GitHub defines `$/` in its
[workflow action-reference syntax](https://docs.github.com/en/actions/reference/workflows-and-actions/workflow-syntax#jobsjob_idstepsuses);
rejecting it here is a repository policy restriction, not invalid GitHub syntax.

The checker parses all top-level `.yml` and `.yaml` files in
`.github/workflows`. It supports block and flow mappings, quoted and escaped
keys/values, explicit keys, multiline scalar values that decode to a valid
reference, and nonrecursive aliases. Property lookup and duplicate detection
ignore ASCII case, consistent with the Actions template reader. Text in
comments, `run` scripts, and action inputs is not an action reference.

Duplicate keys, merge keys, recursive aliases, non-string mapping keys,
unsupported YAML tags, multiple documents, empty documents, invalid structural
types, nested parallel groups, symlink inputs, and a missing or empty workflow directory fail closed.
Only standard map, sequence, and string node tags are supported; implicit
scalars remain text, so keys such as `on` are preserved. This is a conservative
policy subset, not a complete GitHub Actions schema validator. The checker
never constructs arbitrary YAML objects or executes workflow content.

## Evidence and limitations

The regression suite covers mutable and pinned references across equivalent
YAML forms, job-level calls, parallel/background steps, aliases, comment
spoofing, duplicate keys, invalid
input, both file extensions, and the shell check's failure/success output.
Current checked-in workflows are positive compatibility controls. Tests use
disposable local fixtures and do not execute third-party actions.

A passing result establishes reference shape, not commit existence or action
trustworthiness. It does not prove that the action pins its own dependencies,
that GitHub admitted a workflow, or that hosted branch protections enforce the
check. A contributor able to change both the workflow and its checker still
requires review of that change. No release, installation, deployment, secret,
or production authority is granted by this check.

See [ADR 0002](../adr/0002-parse-workflow-action-references.md) for the threat
model, compatibility decision, and alternatives.
