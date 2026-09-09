# Interrupted Tough metadata refresh observation

This experiment observes one process interruption after Tough 0.24.0 verifies
and persists candidate `timestamp.json` but before it reads candidate
`2.snapshot.json`. It uses Tough's existing `RepositoryLoader::datastore`
interface with `FilesystemTransport`; it adds no transport adapter or refresh
interface. A second load with the same datastore and newer local source is the
refresh.

## Fixed public input

The fixture under `tests/data/interrupted-refresh` was generated from the
existing all-role composite fixture. It keeps one signed root, its keys,
`consistent_snapshot = true`, TUF 1.0.36, and the `2999-01-01T00:00:00Z`
expiry unchanged. V2 advances timestamp, snapshot, targets, and delegated
metadata from version 1 to version 2. The top-level targets role has no target;
the delegated role describes one synthetic target, but the experiment never
fetches target content. Synthetic private signing material existed only in the
generator process; these files contain public metadata only.

```text
5023f44894b2a7c434138a2b8a5ae3c2216f217f74eff87ee999a11e21552e90  tests/data/interrupted-refresh/trusted-root.json
f28da9a44f7364d572ede28460a7a0fdab56f2c6a3f081e8cd0ff08f3cd7afa0  tests/data/interrupted-refresh/v1/1.delegated.json
e2756c673c0a345917b31a6e2e2cf0ac65c47cbdbae76bfdb8317c2f48dc4697  tests/data/interrupted-refresh/v1/1.snapshot.json
8e78b164313efe2699caa61154bba0ad9951186f95c6ae2462c449870ebf2ab1  tests/data/interrupted-refresh/v1/1.targets.json
54b539221e480c8c538e3383e8a49a4871030c4678481557140da29febd72a9a  tests/data/interrupted-refresh/v1/timestamp.json
cd1bd57f3c7451b1e32f50c9181e7da682b5deeddf4cd2f38e75a5daa083c43e  tests/data/interrupted-refresh/v2/2.delegated.json
16cb6c561569d06c75c4693981b2a81c7f5fe86be508dde35fab61316409bf95  tests/data/interrupted-refresh/v2/2.snapshot.json
6ed4c83f0c2600bdeff8cf977a30a2b120f2c7a5ff6e9cbf5fc7c97bde8fbe96  tests/data/interrupted-refresh/v2/2.targets.json
59e734efdb3afd0b51f49ec1fdfc9ecd85545f78e804b7d0a84b1d4d67fee312  tests/data/interrupted-refresh/v2/timestamp.json
```

## Prediction and observation

Source inspection predicted mixed datastore residue after interruption and
candidate acceptance on a fresh load with the complete candidate source.

The uninterrupted control loaded timestamp, snapshot, targets, and delegated
metadata at v2. In the interrupted case, the controller first matched the
persisted timestamp against Tough's expected reserialization exactly: 7,270
bytes with SHA-256
`1bfa57f6040d77f957b78a811c90d462614899553e26b7ac2c20f852df314a19`.
It then completed a FIFO writer-open handshake while holding the writer open
without bytes. The loader remained blocked, was killed with `SIGKILL`, and was
reaped. The writer also exited and was reaped. The datastore held timestamp v2
with snapshot, targets, and delegated metadata at v1, so its accepted-state
classification was `neither`, not a hard-coded success.

After the controller restored the exact regular-file bytes of
`2.snapshot.json`, a fresh child process loaded the complete candidate source
with the same trusted root and datastore. The verified repository classified
as `candidate`: timestamp, snapshot, targets, and delegated metadata were all
v2. This execution matched both predictions.

Each run prints the actual `rawObservation` receipt path. By default, the
receipt is written inside the run's fresh disposable workspace and removed
with that workspace. Set `CODIQUARY_22_OUTPUT_DIR` to a fresh directory to
retain it. The receipt records every fixture and datastore filename, exact
UTF-8 bytes, length, SHA-256, FIFO phase evidence, and reaped process
disposition. It also records the actual `latest_known_time.json` bytes and
hash; that wall-clock value is observational and is not a replay invariant.

## Replay

From the repository root, run:

```sh
rustup run 1.98.1 cargo test \
  --manifest-path prototypes/tuf-rust-pq/Cargo.toml \
  --locked --offline --test interrupted_refresh -- --nocapture
```

To retain the receipt, prefix the command with
`CODIQUARY_22_OUTPUT_DIR="$(mktemp -d)"`; the printed `rawObservation` value is
the resulting receipt location.

The 20-second phase guards can only fail the test. They cannot produce a pass
or substitute for the timestamp-byte and FIFO-open evidence.

This is one cooperative process-kill observation. It is not power-loss,
`fsync`, torn-write, full lifecycle, standalone offline-cache, consumer,
installation, execution, release, or production qualification. Tough's
datastore is rollback memory; the successful fresh load also requires the
complete candidate source.
