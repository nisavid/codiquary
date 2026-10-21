"""Security and compatibility regressions for the repository-policy boundary."""

import contextlib
import io
import shutil
import subprocess
import tempfile
import unittest
from pathlib import Path
from unittest.mock import patch

import yaml

from check_workflow_pins import PolicyError, check_workflow, main


ROOT = Path(__file__).resolve().parent.parent
PIN = "example/action@" + "a" * 40
MUTABLE = "example/action@main"


def workflow(step):
    return "on: push\njobs:\n  check:\n    runs-on: ubuntu-latest\n    steps:\n      " + step + "\n"


class WorkflowPinsTest(unittest.TestCase):
    def assert_invalid(self, source):
        with self.assertRaises((PolicyError, yaml.YAMLError, RecursionError)):
            check_workflow(source)

    def test_mutable_references_in_equivalent_yaml_forms(self):
        for step in (
            "- uses: " + MUTABLE,
            "- name: Action\n        uses: " + MUTABLE,
            "- 'uses': " + MUTABLE,
            '- "uses": ' + MUTABLE,
            '- "u\\u0073es": ' + MUTABLE,
            "- {uses: " + MUTABLE + "}",
            "- uses: >-\n          " + MUTABLE,
            "- uses: |-\n          " + MUTABLE,
            '- uses: "example/action@ma\\u0069n"',
            "- ? uses\n        : " + MUTABLE,
            "- USES: " + MUTABLE,
        ):
            with self.subTest(step=step):
                self.assert_invalid(workflow(step))

    def test_pins_in_equivalent_yaml_forms(self):
        for step in (
            "- uses: " + PIN,
            "- name: Action\n        uses: " + PIN,
            "- 'uses': '" + PIN + "'",
            '- "u\\u0073es": "' + PIN + '"',
            "- {uses: " + PIN + "}",
            "- uses: >-\n          " + PIN,
            "- uses: |-\n          " + PIN,
            "- ? uses\n        : " + PIN,
            "- USES: " + PIN,
            "- uses: " + PIN + " # v1",
        ):
            with self.subTest(step=step):
                check_workflow(workflow(step))

    def test_comment_cannot_supply_a_pin(self):
        self.assert_invalid(workflow("- name: Action\n        uses: " + MUTABLE + " # uses: " + PIN))
        self.assert_invalid(workflow("- uses: '" + MUTABLE + " # uses: " + PIN + "'"))

    def test_reusable_workflow_jobs(self):
        for key in ("uses", "'uses'", '"uses"', "USES"):
            for ref, valid in ((PIN.replace("action@", "repo/.github/workflows/test.yml@"), True),
                               (MUTABLE, False)):
                for source in (
                    f"on: push\njobs:\n  call:\n    {key}: {ref}\n",
                    f"on: push\njobs: {{call: {{{key}: {ref}}}}}\n",
                ):
                    with self.subTest(source=source):
                        if valid:
                            check_workflow(source)
                        else:
                            self.assert_invalid(source)

    def test_case_variants_do_not_hide_containers(self):
        self.assert_invalid("JOBS: {check: {STEPS: [{USES: " + MUTABLE + "}]}}")
        check_workflow("JOBS: {check: {STEPS: [{USES: " + PIN + "}]}}")

    def test_scalar_mapping_sequence_and_job_aliases(self):
        for ref in (PIN, MUTABLE):
            sources = (
                f"env: {{REF: &ref {ref}}}\njobs: {{check: {{steps: [{{uses: *ref}}]}}}}",
                f"jobs:\n  check:\n    steps:\n      - &action {{uses: {ref}}}\n      - *action\n",
                f"jobs:\n  first:\n    steps: &steps\n      - uses: {ref}\n  second:\n    steps: *steps\n",
                f"jobs:\n  first: &job\n    uses: {ref}\n  second: *job\n",
            )
            for source in sources:
                with self.subTest(source=source):
                    if ref == PIN:
                        check_workflow(source)
                    else:
                        self.assert_invalid(source)

    def test_parallel_groups_validate_pinned_and_mutable_actions(self):
        for ref in (PIN, MUTABLE):
            for step in (
                "- parallel:\n          - uses: " + ref,
                "- parallel: [{uses: " + ref + "}]",
                "- 'parallel': [{\"uses\": " + ref + "}]",
                "- parallel: &group [{uses: " + ref + "}]\n      - parallel: *group",
                "- parallel:\n          - run: echo ok\n          - uses: " + ref,
            ):
                with self.subTest(step=step):
                    if ref == PIN:
                        check_workflow(workflow(step))
                    else:
                        self.assert_invalid(workflow(step))
        for step in ("- parallel: {}", "- parallel: value", "- parallel: [value]",
                     "- parallel: [{parallel: [{uses: " + PIN + "}]}]"):
            with self.subTest(step=step):
                self.assert_invalid(workflow(step))

    def test_background_actions_are_checked(self):
        for ref in (PIN, MUTABLE):
            source = workflow("- uses: " + ref + "\n        background: true")
            if ref == PIN:
                check_workflow(source)
            else:
                self.assert_invalid(source)

    def test_recursive_undefined_and_merge_aliases_fail(self):
        for source in (
            "jobs: &jobs {check: *jobs}",
            "jobs: {check: {steps: [*unknown]}}",
            "jobs: {check: {steps: [ &step {uses: " + PIN + "}, {<<: *step}]}}",
        ):
            with self.subTest(source=source):
                self.assert_invalid(source)

    def test_duplicate_keys_fail_regardless_of_order_or_case(self):
        for first, second in ((PIN, MUTABLE), (MUTABLE, PIN)):
            for key in ("uses", "'uses'", '"u\\u0073es"', "USES"):
                with self.subTest(key=key, first=first):
                    self.assert_invalid(workflow(f"- uses: {first}\n        {key}: {second}"))
        self.assert_invalid("jobs: {check: {steps: []}}\njobs: {check: {uses: " + MUTABLE + "}}")

    def test_references_require_exact_full_lowercase_sha(self):
        for ref in (
            "example/action", "example/action@v1", "example/action@" + "a" * 39,
            PIN + "a", "example/action@" + "A" * 40, "example/action@" + "g" * 40,
            PIN + " extra", PIN + "@main", PIN + "\\n", " " + PIN,
            "${{ inputs.action }}", "https://github.com/" + PIN,
        ):
            with self.subTest(ref=ref):
                self.assert_invalid(workflow('- uses: "' + ref + '"'))

    def test_local_and_docker_refs_remain_unsupported(self):
        for ref in ("./action", "./action@" + "a" * 40, "../action@" + "a" * 40,
                    "./.github/workflows/local.yml", "$/.github/actions/local",
                    "docker://alpine:latest", "docker://alpine@sha256:" + "a" * 64):
            with self.subTest(ref=ref):
                self.assert_invalid(workflow("- uses: " + ref))

    def test_shell_and_input_text_are_not_action_references(self):
        check_workflow(workflow("- run: |\n          uses: example/action@main\n          echo done"))
        check_workflow(workflow("- uses: " + PIN + "\n        with:\n          uses: example/action@main"))
        check_workflow(workflow("- run: echo done # uses: example/action@main"))

    def test_subdirectory_action_and_mixed_jobs(self):
        check_workflow("on: push\njobs:\n  local:\n    steps:\n      - run: echo ok\n      - uses: "
                       + PIN.replace("action@", "action/subdir@")
                       + "\n  reuse:\n    uses: " + PIN.replace("action@", "repo/.github/workflows/test.yaml@"))

    def test_malformed_documents_and_wrong_types(self):
        for source in (
            "", "# only a comment", "[]", "jobs: [", "jobs: {}", "jobs: []",
            "jobs: {check: []}", "jobs: {check: {}}", "jobs: {check: {steps: {uses: x}}}",
            "jobs: {check: {steps: [null]}}", "jobs: {check: {steps: [{uses: []}]}}",
            "jobs: {check: {uses: {ref: x}}}", "jobs: {check: {uses: null}}",
            "? [jobs]\n: {}", "jobs: {check: {steps: []}}\n---\njobs: {}",
            "jobs: !custom {check: {steps: []}}",
            "jobs: {check: {steps: [{uses: !!python/object/apply:os.system [echo]}]}}",
        ):
            with self.subTest(source=source):
                self.assert_invalid(source)

    def test_existing_workflows_pass(self):
        for path in sorted((ROOT / ".github/workflows").glob("*.yml")):
            with self.subTest(path=path.name):
                check_workflow(path.read_text())

    def test_directory_reads_both_extensions_and_all_files(self):
        with tempfile.TemporaryDirectory() as d:
            root = Path(d)
            (root / "good.yml").write_text(workflow("- uses: " + PIN))
            (root / "bad.yaml").write_text(workflow("- uses: " + MUTABLE))
            with contextlib.redirect_stderr(io.StringIO()) as error:
                self.assertEqual(main(root), 1)
            self.assertIn("bad.yaml", error.getvalue())
            (root / "bad.yaml").write_text(workflow("- uses: " + PIN))
            self.assertEqual(main(root), 0)

    def test_missing_empty_unreadable_or_invalid_utf8_fails(self):
        with tempfile.TemporaryDirectory() as d, contextlib.redirect_stderr(io.StringIO()):
            root = Path(d)
            self.assertEqual(main(root / "missing"), 1)
            self.assertEqual(main(root), 1)
            path = root / "test.yml"
            path.write_bytes(b"\xff")
            self.assertEqual(main(root), 1)
            path.write_text(workflow("- uses: " + PIN))
            with patch.object(Path, "read_text", side_effect=PermissionError("unreadable")):
                self.assertEqual(main(root), 1)
            path.unlink()
            path.mkdir()
            self.assertEqual(main(root), 1)

    def test_symlink_file_and_directory_fail(self):
        with tempfile.TemporaryDirectory() as d, contextlib.redirect_stderr(io.StringIO()):
            root = Path(d)
            source = root / "outside.txt"
            source.write_text(workflow("- uses: " + PIN))
            directory = root / "workflows"
            directory.mkdir()
            (directory / "link.yml").symlink_to(source)
            self.assertEqual(main(directory), 1)
            link = root / "directory-link"
            link.symlink_to(directory, target_is_directory=True)
            self.assertEqual(main(link), 1)

    def test_repository_policy_rejects_trigger_and_accepts_control(self):
        with tempfile.TemporaryDirectory() as d:
            root = Path(d) / "repo"
            shutil.copytree(ROOT, root, ignore=shutil.ignore_patterns(".git", "__pycache__", ".venv-policy", "target"))
            subprocess.run(["git", "init", "-q", str(root)], check=True)
            path = root / ".github/workflows/regression.yaml"
            for step, expected in (("- uses: " + MUTABLE, 1),
                                   ("- 'uses': " + MUTABLE, 1),
                                   ("- {uses: " + MUTABLE + "}", 1),
                                   ("- uses: " + MUTABLE + " # uses: " + PIN, 1),
                                   ("- parallel: [{uses: " + MUTABLE + "}]", 1),
                                   ("- parallel: [{uses: " + PIN + "}]", 0),
                                   ("- uses: " + PIN, 0)):
                with self.subTest(step=step):
                    path.write_text(workflow(step))
                    result = subprocess.run(["./scripts/check-repository.sh"], cwd=root,
                                            text=True, capture_output=True)
                    self.assertEqual(result.returncode, expected, result.stdout + result.stderr)
                    if expected:
                        self.assertNotIn("repository policy checks passed", result.stdout)
                    else:
                        self.assertIn("repository policy checks passed", result.stdout)


if __name__ == "__main__":
    unittest.main()
