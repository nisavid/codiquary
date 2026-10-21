"""Regression and compatibility checks for workflow reference enforcement."""

import contextlib
import io
import shutil
import subprocess
import tempfile
import unittest
from pathlib import Path

import yaml

from check_workflow_pins import PolicyError, check_workflow, main


ROOT = Path(__file__).resolve().parents[1]
SHA = "0123456789abcdef0123456789abcdef01234567"
PIN = f"example/action@{SHA}"
ERRORS = (PolicyError, yaml.YAMLError, RecursionError)


def workflow(step):
    return "name: Test\non: push\njobs:\n  test:\n    runs-on: ubuntu-24.04\n    steps:\n" + step


class WorkflowPinsTest(unittest.TestCase):
    def test_repository_workflows(self):
        self.assertEqual(main(ROOT / ".github/workflows"), 0)

    def test_equivalent_step_forms(self):
        forms = [
            "      - uses: REF\n",
            "      - name: Example\n        uses: REF\n",
            "      - 'uses': 'REF'\n",
            '      - "uses": "REF"\n',
            '      - "u\\u0073es": "REF"\n',
            "      - {uses: REF}\n",
            "      - uses: >-\n          REF\n",
            "      - uses: |-\n          REF\n",
            "      - ? uses\n        : REF\n",
            "      - uses: &action REF\n      - uses: *action\n",
            "      - &step {uses: REF}\n      - *step\n",
        ]
        for form in forms:
            with self.subTest(form=form):
                check_workflow(workflow(form.replace("REF", PIN)))
                with self.assertRaises(ERRORS):
                    check_workflow(workflow(form.replace("REF", "example/action@main")))

    def test_flow_document(self):
        for ref in (PIN, "example/action@main"):
            source = f"{{on: push, jobs: {{test: {{steps: [{{uses: {ref}}}]}}}}}}"
            if ref == PIN:
                check_workflow(source)
            else:
                with self.assertRaises(ERRORS):
                    check_workflow(source)

    def test_reusable_workflows(self):
        forms = [
            "jobs:\n  call:\n    uses: REF\n",
            "jobs: {call: {'uses': 'REF'}}\n",
            "jobs:\n  call: &job {uses: REF}\n  again: *job\n",
        ]
        for form in forms:
            for ref in (f"example/repo/.github/workflows/build.yml@{SHA}",
                        "example/repo/.github/workflows/build.yml@main"):
                with self.subTest(form=form, ref=ref):
                    if ref.endswith(SHA):
                        check_workflow(form.replace("REF", ref))
                    else:
                        with self.assertRaises(ERRORS):
                            check_workflow(form.replace("REF", ref))

    def test_aliases_from_non_execution_locations(self):
        for ref in (PIN, "example/action@main"):
            source = f"env: {{REFERENCE: &ref {ref}}}\njobs:\n  test:\n    steps:\n      - uses: *ref\n"
            if ref == PIN:
                check_workflow(source)
            else:
                with self.assertRaises(ERRORS):
                    check_workflow(source)

    def test_parallel_and_background_actions(self):
        for ref in (PIN, "example/action@main"):
            for step in (
                f"      - parallel: [{{uses: {ref}}}]\n",
                f"      - parallel:\n          - uses: {ref}\n",
                f"      - parallel: &group [{{uses: {ref}}}]\n      - parallel: *group\n",
                f"      - parallel: [{{parallel: [{{uses: {ref}}}]}}]\n",
                f"      - uses: {ref}\n        background: true\n",
            ):
                with self.subTest(step=step):
                    if ref == PIN:
                        check_workflow(workflow(step))
                    else:
                        with self.assertRaises(ERRORS):
                            check_workflow(workflow(step))
        for group in ("{}", "null", "[null]"):
            with self.subTest(group=group), self.assertRaises(ERRORS):
                check_workflow(workflow(f"      - parallel: {group}\n"))

    def test_workflow_directory_symlink_is_rejected(self):
        with tempfile.TemporaryDirectory() as directory, contextlib.redirect_stderr(io.StringIO()):
            root = Path(directory)
            actual = root / "actual"
            actual.mkdir()
            (actual / "test.yml").write_text(workflow(f"      - uses: {PIN}\n"))
            link = root / "linked"
            link.symlink_to(actual, target_is_directory=True)
            self.assertEqual(main(link), 1)

    def test_rejects_unpinned_or_unsupported_references(self):
        references = [
            "example/action@main", "example/action@v1", "example/action",
            PIN[:-1], PIN + "0", PIN.upper(), PIN + " trailing-text",
            "./local-action", "./.github/workflows/local.yml",
            "./action@" + SHA, "../action@" + SHA,
            "./.github/workflows/local.yml@" + SHA,
            "actions/checkout/../../../../repo/repo/local-action@" + SHA,
            "example/repo/./action@" + SHA, "example/../action@" + SHA,
            "docker://example/image:latest", "docker://example/image@sha256:" + "a" * 64,
            "${{ inputs.action }}", "null", "true", "123", "", "[]", "{}",
            f"'{PIN} # comment in scalar'", f"'{PIN}\n          extra'",
        ]
        for reference in references:
            with self.subTest(reference=reference), self.assertRaises(ERRORS):
                check_workflow(workflow(f"      - uses: {reference}\n"))

    def test_comments_cannot_supply_a_pin(self):
        with self.assertRaises(ERRORS):
            check_workflow(workflow(f"      - name: Example\n        uses: example/action@main # uses: {PIN}\n"))
        check_workflow(workflow(f"      - uses: {PIN} # v1\n"))

    def test_multiline_value_with_trailing_newline_is_rejected(self):
        with self.assertRaises(ERRORS):
            check_workflow(workflow(f"      - uses: |\n          {PIN}\n"))

    def test_duplicates_at_all_enclosing_levels(self):
        sources = [
            f"jobs: {{test: {{steps: [{{uses: example/action@main, 'uses': {PIN}}}]}}}}",
            f"jobs: {{test: {{steps: [{{uses: {PIN}, uses: example/action@main}}]}}}}",
            f"jobs: {{test: {{steps: [{{uses: {PIN}}}], steps: []}}}}",
            "jobs: {test: {steps: []}, test: {steps: []}}",
            "jobs: {test: {steps: []}}\njobs: {other: {steps: []}}",
            "env: {X: a, X: b}\njobs: {test: {steps: []}}",
        ]
        for source in sources:
            with self.subTest(source=source), self.assertRaises(ERRORS):
                check_workflow(source)

    def test_expression_keys_cannot_hide_execution_locations(self):
        sources = [
            workflow("      - ${{ 'uses' }}: example/action@main\n"),
            workflow("      - uses: example/action@main\n").replace("steps:", "${{ 'steps' }}:"),
            "jobs: {call: {\"${{ 'uses' }}\": example/repo/.github/workflows/build.yml@main}}",
            f"jobs: {{test: {{steps: [{{uses: {PIN}, \"${{{{ 'uses' }}}}\": example/action@main}}]}}}}",
            "env: {KEY: &key \"${{ 'uses' }}\"}\njobs: {test: {steps: [{*key: example/action@main}]}}",
            workflow('      - "\\u0024{{ \'uses\' }}": example/action@main\n'),
        ]
        for source in sources:
            with self.subTest(source=source), self.assertRaisesRegex(PolicyError, "expressions in YAML mapping keys"):
                check_workflow(source)

    def test_malformed_or_ambiguous_yaml_fails(self):
        sources = [
            "", "# empty", "[]", "jobs: [", "jobs: {}", "jobs: []",
            "jobs: {test: null}", "jobs: {test: {steps: {uses: ignored}}}",
            "jobs: {test: {steps: [null]}}",
            "jobs: {test: {steps: []}}\n---\njobs: {}",
            "jobs: {test: {steps: *undefined}}",
            "jobs: &jobs {test: *jobs}",
            "jobs: {test: {steps: [{<<: {uses: ignored}}]}}",
            "jobs: {test: {steps: [{!custom uses: ignored}]}}",
            "jobs: {test: {steps: [{uses: !!python/object:object {}}]}}",
            "jobs: {test: {steps: [], ? [complex, key]: ignored}}",
            "env: {A: &same a, B: &same b}\njobs: {test: {steps: []}}",
        ]
        for source in sources:
            with self.subTest(source=source), self.assertRaises(ERRORS):
                check_workflow(source)

    def test_inert_uses_text_is_not_an_action(self):
        check_workflow("""on: push
env:
  uses: irrelevant
jobs:
  test:
    runs-on: ubuntu-24.04
    steps:
      - run: |
          uses: example/action@main
        env: {uses: irrelevant}
""")
        check_workflow(workflow(f"      - uses: {PIN}\n        with: {{uses: irrelevant}}\n"))

    def test_discovery_and_read_failures(self):
        with tempfile.TemporaryDirectory() as directory, contextlib.redirect_stderr(io.StringIO()):
            root = Path(directory)
            self.assertEqual(main(root / "missing"), 1)
            self.assertEqual(main(root), 1)
            path = root / "test.yaml"
            path.mkdir()
            self.assertEqual(main(root), 1)
            path.rmdir()
            path.symlink_to(root / "missing")
            self.assertEqual(main(root), 1)
            path.unlink()
            path.write_bytes(b"\xff")
            self.assertEqual(main(root), 1)
            path.write_text(workflow(f"      - uses: {PIN}\n"))
            self.assertEqual(main(root), 0)
            (root / "bad.yml").write_text(workflow("      - uses: example/action@main\n"))
            self.assertEqual(main(root), 1)

    def test_repository_entrypoint_propagates_failure(self):
        # Exercise the real shell caller, including its non-workflow checks.
        with tempfile.TemporaryDirectory() as directory:
            checkout = Path(directory) / "repo"
            shutil.copytree(ROOT, checkout, ignore=shutil.ignore_patterns(".git", "__pycache__", ".venv", "target"))
            subprocess.run(["git", "init", "-q", str(checkout)], check=True)
            subprocess.run(["git", "-C", str(checkout), "add", "."], check=True)
            probe = checkout / ".github/workflows/probe.yml"
            cases = [
                ("      - uses: example/action@main\n", 1),
                ("      - 'uses': example/action@main\n", 1),
                ("      - {uses: example/action@main}\n", 1),
                (f"      - uses: example/action@main # uses: {PIN}\n", 1),
                ("      - ${{ 'uses' }}: example/action@main\n", 1),
                ("      - parallel: [{uses: example/action@main}]\n", 1),
                (f"      - parallel: [{{uses: {PIN}}}]\n", 0),
                (f"      - uses: {PIN}\n", 0),
            ]
            for step, expected in cases:
                probe.write_text(workflow(step))
                result = subprocess.run(["./scripts/check-repository.sh"], cwd=checkout, text=True, capture_output=True)
                self.assertEqual(result.returncode, expected, result.stdout + result.stderr)
                if expected:
                    self.assertNotIn("repository policy checks passed", result.stdout)


if __name__ == "__main__":
    unittest.main()
