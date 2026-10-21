# Tests

Repository-policy checks live under `scripts/`, independently of the release
implementation. Implementation tests will be colocated according to the selected
substrate while preserving clear ownership by contract, core, adapter,
conformance, and qualification surface.

A test describes only the behavior and externally oriented contract it proves.
`scripts/test_workflow_pins.py` tests the workflow action-reference policy and
its shell entrypoint. Install `scripts/requirements-policy.txt` as described in
`CONTRIBUTING.md`, then run:

```sh
python3 -m unittest discover -s scripts -p 'test_*.py' -v
```

Fixtures are disposable and never execute external actions. These tests prove
local repository-policy behavior, not hosted workflow admission or release
safety. No release-operations implementation test suite exists yet.
