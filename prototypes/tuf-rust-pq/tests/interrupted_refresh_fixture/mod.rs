use aws_lc_rs::digest::{digest, SHA256};
use std::collections::BTreeMap;
use std::error::Error;
use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::{Child, Command, ExitStatus, Stdio};
use std::sync::atomic::{AtomicU64, Ordering};
use std::thread;
use std::time::{Duration, Instant};

use tough::schema::{Signed, Snapshot, Targets, Timestamp};
use tough::{FilesystemTransport, Repository, RepositoryLoader};

type BoxError = Box<dyn Error + Send + Sync + 'static>;

const CHILD_MODE: &str = "CODIQUARY_22_CHILD_MODE";
const CHILD_ROOT: &str = "CODIQUARY_22_CHILD_ROOT";
const CHILD_SOURCE: &str = "CODIQUARY_22_CHILD_SOURCE";
const CHILD_TARGETS: &str = "CODIQUARY_22_CHILD_TARGETS";
const CHILD_DATASTORE: &str = "CODIQUARY_22_CHILD_DATASTORE";
const CHILD_RECEIPT: &str = "CODIQUARY_22_CHILD_RECEIPT";
const OUTPUT_DIR: &str = "CODIQUARY_22_OUTPUT_DIR";
const TEST_NAME: &str = "observes_one_interrupted_tough_metadata_refresh";
const PHASE_TIMEOUT: Duration = Duration::from_secs(20);
const POLL_INTERVAL: Duration = Duration::from_millis(10);

static TEMP_SEQUENCE: AtomicU64 = AtomicU64::new(0);

struct SyntheticInputs {
    root: Vec<u8>,
    v1: BTreeMap<String, Vec<u8>>,
    v2: BTreeMap<String, Vec<u8>>,
}

struct DisposableDirectory {
    path: PathBuf,
}

impl DisposableDirectory {
    fn new() -> Result<Self, BoxError> {
        let sequence = TEMP_SEQUENCE.fetch_add(1, Ordering::Relaxed);
        let path = std::env::temp_dir().join(format!(
            "codiquary-tuf-rust-pq-interrupted-refresh-{}-{sequence}",
            std::process::id()
        ));
        fs::create_dir(&path)?;
        Ok(Self { path })
    }
}

impl Drop for DisposableDirectory {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.path);
    }
}

struct ReapedChild {
    child: Option<Child>,
}

impl ReapedChild {
    fn spawn(command: &mut Command) -> Result<Self, BoxError> {
        Ok(Self {
            child: Some(command.spawn()?),
        })
    }

    fn id(&self) -> u32 {
        self.child.as_ref().expect("child is present").id()
    }

    fn try_wait(&mut self) -> Result<Option<ExitStatus>, BoxError> {
        Ok(self.child.as_mut().expect("child is present").try_wait()?)
    }

    fn stdin_mut(&mut self) -> Option<&mut std::process::ChildStdin> {
        self.child
            .as_mut()
            .expect("child is present")
            .stdin
            .as_mut()
    }

    fn close_stdin(&mut self) {
        self.child.as_mut().expect("child is present").stdin.take();
    }

    fn kill_and_wait(&mut self) -> Result<ExitStatus, BoxError> {
        let child = self.child.as_mut().expect("child is present");
        child.kill()?;
        Ok(child.wait()?)
    }

    fn wait(&mut self) -> Result<ExitStatus, BoxError> {
        Ok(self.child.as_mut().expect("child is present").wait()?)
    }
}

impl Drop for ReapedChild {
    fn drop(&mut self) {
        if let Some(mut child) = self.child.take() {
            if child.try_wait().ok().flatten().is_none() {
                let _ = child.kill();
            }
            let _ = child.wait();
        }
    }
}

pub fn observe() -> Result<(), BoxError> {
    if std::env::var_os(CHILD_MODE).is_some() {
        return child_observe();
    }

    tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()?
        .block_on(parent_observe())
}

fn child_observe() -> Result<(), BoxError> {
    let root = required_path(CHILD_ROOT)?;
    let source = required_path(CHILD_SOURCE)?;
    let targets = required_path(CHILD_TARGETS)?;
    let datastore = required_path(CHILD_DATASTORE)?;
    let receipt = required_path(CHILD_RECEIPT)?;
    let root_bytes = fs::read(root)?;

    let observation = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()?
        .block_on(async {
            let repository = load_repository(&root_bytes, &source, &targets, &datastore).await?;
            repository_observation(&repository)
        })?;
    write_json(&receipt, &observation)?;
    Ok(())
}

async fn parent_observe() -> Result<(), BoxError> {
    let workspace = DisposableDirectory::new()?;
    let root_path = workspace.path.join("trusted-root.json");
    let v1_source = workspace.path.join("v1-source");
    let control_source = workspace.path.join("control-v2-source");
    let interrupted_source = workspace.path.join("interrupted-v2-source");
    let targets = workspace.path.join("targets");
    let v1_datastore = workspace.path.join("v1-datastore");
    let control_datastore = workspace.path.join("control-datastore");
    let interrupted_datastore = workspace.path.join("interrupted-datastore");
    for directory in [
        &v1_source,
        &control_source,
        &interrupted_source,
        &targets,
        &v1_datastore,
    ] {
        fs::create_dir(directory)?;
    }

    let inputs = synthetic_inputs()?;
    fs::write(&root_path, &inputs.root)?;
    write_source(&v1_source, &inputs.v1)?;
    write_source(&control_source, &inputs.v2)?;
    write_source(&interrupted_source, &inputs.v2)?;

    let v1_repository = load_repository(&inputs.root, &v1_source, &targets, &v1_datastore).await?;
    let v1_observation = repository_observation(&v1_repository)?;
    copy_directory(&v1_datastore, &control_datastore)?;
    copy_directory(&v1_datastore, &interrupted_datastore)?;

    let control_repository =
        load_repository(&inputs.root, &control_source, &targets, &control_datastore).await?;
    let control_observation = repository_observation(&control_repository)?;

    let snapshot_path = interrupted_source.join("2.snapshot.json");
    let original_snapshot = fs::read(&snapshot_path)?;
    fs::remove_file(&snapshot_path)?;
    let mkfifo = Command::new("mkfifo").arg(&snapshot_path).status()?;
    if !mkfifo.success() {
        return Err(format!("mkfifo failed with {mkfifo}").into());
    }

    let interrupted_receipt_path = workspace.path.join("interrupted-child.json");
    let mut interrupted_child = spawn_loader_child(
        &root_path,
        &interrupted_source,
        &targets,
        &interrupted_datastore,
        &interrupted_receipt_path,
    )?;
    let interrupted_pid = interrupted_child.id();

    let expected_timestamp = reserialized::<Timestamp>(&inputs.v2["timestamp.json"])?;
    wait_for_bytes(
        &interrupted_datastore.join("timestamp.json"),
        &expected_timestamp,
        &mut interrupted_child,
        "v2 timestamp persistence",
    )?;

    let writer_handshake = workspace.path.join("fifo-writer-open");
    let mut writer_command = Command::new("/bin/sh");
    writer_command
        .arg("-c")
        .arg("exec 3>\"$1\"; : >\"$2\"; IFS= read -r _ || :")
        .arg("codiquary-fifo-writer")
        .arg(&snapshot_path)
        .arg(&writer_handshake)
        .stdin(Stdio::piped())
        .stdout(Stdio::null())
        .stderr(Stdio::null());
    let mut writer_child = ReapedChild::spawn(&mut writer_command)?;
    let writer_pid = writer_child.id();
    wait_for_file(
        &writer_handshake,
        &mut writer_child,
        "FIFO writer-open handshake",
    )?;
    if interrupted_child.try_wait()?.is_some() {
        return Err("loader child exited after FIFO opened without receiving bytes".into());
    }
    if writer_child.stdin_mut().is_none() {
        return Err("FIFO writer stdin was not held open".into());
    }

    let interrupted_status = interrupted_child.kill_and_wait()?;
    writer_child.close_stdin();
    let writer_status = writer_child.wait()?;

    fs::remove_file(&snapshot_path)?;
    fs::write(&snapshot_path, &original_snapshot)?;
    if fs::read(&snapshot_path)? != original_snapshot {
        return Err("restored v2 snapshot differs from its original bytes".into());
    }

    let partial_datastore = directory_manifest(&interrupted_datastore)?;
    let datastore_residue = datastore_residue(&interrupted_datastore)?;

    let fresh_receipt_path = workspace.path.join("fresh-child.json");
    let mut fresh_child = spawn_loader_child(
        &root_path,
        &interrupted_source,
        &targets,
        &interrupted_datastore,
        &fresh_receipt_path,
    )?;
    let fresh_pid = fresh_child.id();
    let fresh_status = wait_for_successful_child(&mut fresh_child, "fresh loader")?;
    let fresh_observation: serde_json::Value =
        serde_json::from_slice(&fs::read(&fresh_receipt_path)?)?;

    let receipt = serde_json::json!({
        "experiment": "Observe one interrupted Tough metadata refresh",
        "fixtureInputs": {
            "root": file_record("trusted-root.json", &inputs.root),
            "v1": byte_map_manifest(&inputs.v1),
            "v2": byte_map_manifest(&inputs.v2),
        },
        "predictions": {
            "afterInterruption": "mixed",
            "freshLoad": "candidate",
        },
        "observations": {
            "establishedV1": v1_observation,
            "uninterruptedControl": {
                "repository": control_observation,
                "datastore": directory_manifest(&control_datastore)?,
            },
            "interrupted": {
                "expectedReserializedV2Timestamp": file_record("timestamp.json", &expected_timestamp),
                "fifoWriterOpenHandshake": true,
                "loaderProcess": process_record(interrupted_pid, interrupted_status),
                "writerProcess": process_record(writer_pid, writer_status),
                "datastoreResidue": datastore_residue,
                "datastore": partial_datastore,
                "candidateSnapshotRestored": file_record("2.snapshot.json", &original_snapshot),
            },
            "freshLoad": {
                "repository": fresh_observation,
                "process": process_record(fresh_pid, fresh_status),
                "datastore": directory_manifest(&interrupted_datastore)?,
            },
        },
        "limits": [
            "one cooperative process kill after a proven FIFO-read phase",
            "no power-loss, fsync, or torn-write observation",
            "datastore is rollback memory, not a self-contained repository cache",
            "no target content fetch, consumer, installation, execution, or production claim",
        ],
    });
    let observation_dir = match std::env::var_os(OUTPUT_DIR) {
        Some(path) if !path.is_empty() => PathBuf::from(path),
        Some(_) => return Err(format!("{OUTPUT_DIR} must not be empty").into()),
        None => workspace.path.clone(),
    };
    fs::create_dir_all(&observation_dir)?;
    let observation_path = observation_dir.join("issue-22-observation.json");
    write_json(&observation_path, &receipt)?;

    assert_state(&receipt["observations"]["establishedV1"], "previous")?;
    assert_state(
        &receipt["observations"]["uninterruptedControl"]["repository"],
        "candidate",
    )?;
    assert_mixed_datastore_residue(&receipt["observations"]["interrupted"]["datastoreResidue"])?;
    assert_killed(&receipt["observations"]["interrupted"]["loaderProcess"])?;
    if !writer_status.success() || !fresh_status.success() {
        return Err("a reaped fixture process had an unexpected disposition".into());
    }

    println!(
        "{}",
        serde_json::json!({
            "control": receipt["observations"]["uninterruptedControl"]["repository"]["acceptedState"],
            "interrupted": receipt["observations"]["interrupted"]["datastoreResidue"],
            "freshLoad": receipt["observations"]["freshLoad"]["repository"]["acceptedState"],
            "rawObservation": observation_path,
        })
    );
    Ok(())
}

fn synthetic_inputs() -> Result<SyntheticInputs, BoxError> {
    let fixture = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests")
        .join("data")
        .join("interrupted-refresh");
    Ok(SyntheticInputs {
        root: fs::read(fixture.join("trusted-root.json"))?,
        v1: read_source(&fixture.join("v1"))?,
        v2: read_source(&fixture.join("v2"))?,
    })
}

fn read_source(directory: &Path) -> Result<BTreeMap<String, Vec<u8>>, BoxError> {
    let mut source = BTreeMap::new();
    for entry in fs::read_dir(directory)? {
        let entry = entry?;
        source.insert(
            entry.file_name().to_string_lossy().into_owned(),
            fs::read(entry.path())?,
        );
    }
    Ok(source)
}

async fn load_repository(
    root: &[u8],
    source: &Path,
    targets: &Path,
    datastore: &Path,
) -> Result<Repository, BoxError> {
    let source_url = format!("file://{}/", source.canonicalize()?.display()).parse()?;
    let targets_url = format!("file://{}/", targets.canonicalize()?.display()).parse()?;
    Ok(RepositoryLoader::new(&root, source_url, targets_url)
        .transport(FilesystemTransport)
        .datastore(datastore)
        .load()
        .await?)
}

fn repository_observation(repository: &Repository) -> Result<serde_json::Value, BoxError> {
    let delegation = repository
        .targets()
        .signed
        .delegations
        .as_ref()
        .and_then(|delegations| delegations.roles.first())
        .and_then(|role| role.targets.as_ref())
        .ok_or_else(|| "verified repository did not load delegated metadata".to_owned())?;
    let versions = [
        repository.timestamp().signed.version.get(),
        repository.snapshot().signed.version.get(),
        repository.targets().signed.version.get(),
        delegation.signed.version.get(),
    ];
    let accepted_state = classify_versions(versions);
    Ok(serde_json::json!({
        "acceptedState": accepted_state,
        "rootVersion": repository.root().signed.version.get(),
        "timestampVersion": versions[0],
        "snapshotVersion": versions[1],
        "targetsVersion": versions[2],
        "delegatedVersion": versions[3],
    }))
}

fn classify_versions(versions: [u64; 4]) -> &'static str {
    if versions.iter().all(|version| *version == 1) {
        "previous"
    } else if versions.iter().all(|version| *version == 2) {
        "candidate"
    } else {
        "neither"
    }
}

fn datastore_residue(datastore: &Path) -> Result<serde_json::Value, BoxError> {
    let timestamp: Signed<Timestamp> = read_json(&datastore.join("timestamp.json"))?;
    let snapshot: Signed<Snapshot> = read_json(&datastore.join("snapshot.json"))?;
    let targets: Signed<Targets> = read_json(&datastore.join("targets.json"))?;
    let delegated_version = snapshot
        .signed
        .meta
        .get("delegated.json")
        .ok_or_else(|| "snapshot omits delegated.json".to_owned())?
        .version;
    let delegated: Signed<Targets> =
        read_json(&datastore.join(format!("{delegated_version}.delegated.json")))?;
    let versions = [
        timestamp.signed.version.get(),
        snapshot.signed.version.get(),
        targets.signed.version.get(),
        delegated.signed.version.get(),
    ];
    Ok(serde_json::json!({
        "residueState": classify_residue(versions),
        "timestampVersion": versions[0],
        "snapshotVersion": versions[1],
        "targetsVersion": versions[2],
        "delegatedVersion": versions[3],
    }))
}

fn classify_residue(versions: [u64; 4]) -> &'static str {
    if versions.iter().all(|version| *version == 1) {
        "all-v1"
    } else if versions.iter().all(|version| *version == 2) {
        "all-v2"
    } else {
        "mixed"
    }
}

fn read_json<T: serde::de::DeserializeOwned>(path: &Path) -> Result<T, BoxError> {
    Ok(serde_json::from_slice(&fs::read(path)?)?)
}

fn required_path(name: &str) -> Result<PathBuf, BoxError> {
    std::env::var_os(name)
        .map(PathBuf::from)
        .ok_or_else(|| format!("missing child path environment variable {name}").into())
}

fn spawn_loader_child(
    root: &Path,
    source: &Path,
    targets: &Path,
    datastore: &Path,
    receipt: &Path,
) -> Result<ReapedChild, BoxError> {
    let mut command = Command::new(std::env::current_exe()?);
    command
        .arg("--exact")
        .arg(TEST_NAME)
        .arg("--nocapture")
        .env(CHILD_MODE, "load")
        .env(CHILD_ROOT, root)
        .env(CHILD_SOURCE, source)
        .env(CHILD_TARGETS, targets)
        .env(CHILD_DATASTORE, datastore)
        .env(CHILD_RECEIPT, receipt)
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null());
    ReapedChild::spawn(&mut command)
}

fn wait_for_bytes(
    path: &Path,
    expected: &[u8],
    child: &mut ReapedChild,
    phase: &str,
) -> Result<(), BoxError> {
    let deadline = Instant::now() + PHASE_TIMEOUT;
    loop {
        if fs::read(path).is_ok_and(|bytes| bytes == expected) {
            return Ok(());
        }
        if let Some(status) = child.try_wait()? {
            return Err(format!("fixture child exited during {phase}: {status}").into());
        }
        if Instant::now() >= deadline {
            return Err(format!("timed out waiting for {phase}").into());
        }
        thread::sleep(POLL_INTERVAL);
    }
}

fn wait_for_file(path: &Path, child: &mut ReapedChild, phase: &str) -> Result<(), BoxError> {
    let deadline = Instant::now() + PHASE_TIMEOUT;
    loop {
        if path.is_file() {
            return Ok(());
        }
        if let Some(status) = child.try_wait()? {
            return Err(format!("fixture child exited during {phase}: {status}").into());
        }
        if Instant::now() >= deadline {
            return Err(format!("timed out waiting for {phase}").into());
        }
        thread::sleep(POLL_INTERVAL);
    }
}

fn wait_for_successful_child(child: &mut ReapedChild, phase: &str) -> Result<ExitStatus, BoxError> {
    let deadline = Instant::now() + PHASE_TIMEOUT;
    loop {
        if let Some(status) = child.try_wait()? {
            if status.success() {
                return Ok(status);
            }
            return Err(format!("{phase} exited unsuccessfully: {status}").into());
        }
        if Instant::now() >= deadline {
            return Err(format!("timed out waiting for {phase}").into());
        }
        thread::sleep(POLL_INTERVAL);
    }
}

fn write_source(directory: &Path, files: &BTreeMap<String, Vec<u8>>) -> Result<(), BoxError> {
    for (name, bytes) in files {
        fs::write(directory.join(name), bytes)?;
    }
    Ok(())
}

fn copy_directory(source: &Path, destination: &Path) -> Result<(), BoxError> {
    fs::create_dir(destination)?;
    for entry in fs::read_dir(source)? {
        let entry = entry?;
        fs::copy(entry.path(), destination.join(entry.file_name()))?;
    }
    Ok(())
}

fn reserialized<T>(bytes: &[u8]) -> Result<Vec<u8>, BoxError>
where
    T: serde::de::DeserializeOwned + serde::Serialize,
{
    Ok(serde_json::to_vec(&serde_json::from_slice::<Signed<T>>(
        bytes,
    )?)?)
}

fn byte_map_manifest(files: &BTreeMap<String, Vec<u8>>) -> serde_json::Value {
    serde_json::Value::Array(
        files
            .iter()
            .map(|(name, bytes)| file_record(name, bytes))
            .collect(),
    )
}

fn directory_manifest(directory: &Path) -> Result<serde_json::Value, BoxError> {
    let mut entries = fs::read_dir(directory)?.collect::<Result<Vec<_>, _>>()?;
    entries.sort_by_key(std::fs::DirEntry::file_name);
    Ok(serde_json::Value::Array(
        entries
            .into_iter()
            .map(|entry| {
                let bytes = fs::read(entry.path())?;
                Ok(file_record(&entry.file_name().to_string_lossy(), &bytes))
            })
            .collect::<Result<Vec<_>, std::io::Error>>()?,
    ))
}

fn file_record(name: &str, bytes: &[u8]) -> serde_json::Value {
    serde_json::json!({
        "name": name,
        "length": bytes.len(),
        "sha256": sha256_hex(bytes),
        "bytesUtf8": String::from_utf8_lossy(bytes),
    })
}

fn sha256_hex(bytes: &[u8]) -> String {
    digest(&SHA256, bytes)
        .as_ref()
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect()
}

#[cfg(unix)]
fn process_record(pid: u32, status: ExitStatus) -> serde_json::Value {
    use std::os::unix::process::ExitStatusExt;
    serde_json::json!({
        "pid": pid,
        "reaped": true,
        "success": status.success(),
        "code": status.code(),
        "signal": status.signal(),
    })
}

fn assert_state(value: &serde_json::Value, expected: &str) -> Result<(), BoxError> {
    if value["acceptedState"] != expected {
        return Err(format!(
            "expected accepted state {expected}, observed {}",
            value["acceptedState"]
        )
        .into());
    }
    Ok(())
}

fn assert_mixed_datastore_residue(value: &serde_json::Value) -> Result<(), BoxError> {
    if value["residueState"] != "mixed"
        || value["timestampVersion"] != 2
        || value["snapshotVersion"] != 1
        || value["targetsVersion"] != 1
        || value["delegatedVersion"] != 1
    {
        return Err(format!("expected timestamp-v2/other-v1 residue, observed {value}").into());
    }
    Ok(())
}

#[cfg(unix)]
fn assert_killed(value: &serde_json::Value) -> Result<(), BoxError> {
    if value["reaped"] != true || value["signal"] != 9 || value["success"] != false {
        return Err(format!("interrupted loader was not killed and reaped: {value}").into());
    }
    Ok(())
}

fn write_json(path: &Path, value: &serde_json::Value) -> Result<(), BoxError> {
    let mut file = fs::File::create(path)?;
    serde_json::to_writer_pretty(&mut file, value)?;
    file.write_all(b"\n")?;
    Ok(())
}
