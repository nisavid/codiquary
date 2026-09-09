# Issue 23 publisher/client target binding

This fixture binds the synthetic publisher and one Tough-loaded client descriptor to one exact target:

- path: `tests/data/publisher-client-binding/targets/artifact.bin`
- bytes: `codiquary issue 20 fixture\n`
- length: 27 bytes
- SHA-256: `d13ecc865c37b23650615038c232eccfa5770c8bc345dbe98db595273fecda4a`

The publisher assertion in `tests/composite_metadata.rs` reads those bytes with `include_bytes!` and checks the serialized target length and SHA-256. The client bootstraps from `tests/data/publisher-client-binding/trusted-root.json`, loads and verifies the single-version metadata directory through `RepositoryLoader::load`, selects the sole `artifact.bin` descriptor through `Repository::all_targets()`, and compares its length and SHA-256 with the publisher bytes. The selected descriptor is `tests/data/publisher-client-binding/metadata/1.delegated.json`.

The metadata was signed once with an independently generated ephemeral synthetic composite key using the existing all-role fixture construction. Only public metadata was retained; no private signing material was written. The accepted interrupted-refresh fixtures and experiment are not inputs to the final test and remain unchanged.

## TDD and replay

The focused RED run loaded the prior v1-shaped public metadata and failed on its verified `artifact.bin` SHA-256, `b8d282e2625b313262da2203a7627e82275e8b96a08ce099324275c0724dc0b0`, versus the publisher SHA-256 above. `focused-red.log` retains that expected failure. After generating the dedicated public fixture, `focused-green.log` records one passing focused test.

Run the focused slice from the repository root:

```sh
rustup run 1.98.1 cargo test \
  --manifest-path prototypes/tuf-rust-pq/Cargo.toml \
  --locked --offline \
  --target-dir /tmp/codiquary-tuf-rust-pq-278-target \
  --test publisher_client_binding -- \
  --exact publisher_and_client_bind_exact_target_bytes --nocapture
```

The owning prototype suite, clippy with warnings denied, formatting check, repository policy check, and Git whitespace check are the completion checks. This fixture does not retrieve a target and does not establish transport, retained-policy, lifecycle, installation, production, or macOS authority.
