mod support {
    pub mod composite_metadata_fixture;
    pub mod curl_fixture_transport;
    pub mod loopback_http_fixture;
    pub mod pacman_native_fixture;
}

use support::composite_metadata_fixture::publish_single_delegated_target;
use support::pacman_native_fixture::*;

use aws_lc_rs::digest::{digest, SHA256};
use std::error::Error;
use std::fs::{self, OpenOptions};
use std::future::Future;
use std::io::Write;
use std::os::unix::fs::{OpenOptionsExt, PermissionsExt};
use std::path::{Path, PathBuf};
use std::process::{Command, Output};
use std::sync::atomic::{AtomicU64, Ordering};
use tough::{FilesystemTransport, IntoVec, RepositoryLoader, TargetName};
use url::Url;

const EXPECTED_ARTIFACT_BYTES: &[u8] = b"codiquary issue 20 fixture\n";
const EXPECTED_ARTIFACT_SHA256: [u8; 32] = [
    0xd1, 0x3e, 0xcc, 0x86, 0x5c, 0x37, 0xb2, 0x36, 0x50, 0x61, 0x50, 0x38, 0xc2, 0x32, 0xec, 0xcf,
    0xa5, 0x77, 0x0c, 0x8b, 0xc3, 0x45, 0xdb, 0xe9, 0x8d, 0xb5, 0x95, 0x27, 0x3f, 0xec, 0xda, 0x4a,
];
const EXPECTED_PUBLIC_CERTIFICATE_SHA256: [u8; 32] = [
    0x88, 0xdd, 0x35, 0xa5, 0x03, 0x39, 0x17, 0x43, 0x8b, 0x1e, 0x9a, 0x2f, 0x21, 0xa9, 0x31, 0x77,
    0x37, 0x66, 0xd6, 0xe8, 0x67, 0xfb, 0xba, 0xa4, 0xe6, 0x01, 0x85, 0xa0, 0xd2, 0xac, 0xb2, 0xe7,
];
const EXPECTED_DETACHED_SIGNATURE_SHA256: [u8; 32] = [
    0x10, 0xc6, 0x32, 0xd7, 0xfb, 0x16, 0x0b, 0x0d, 0x9a, 0xdf, 0xce, 0xeb, 0x9a, 0x8c, 0x9b, 0x20,
    0x97, 0xf5, 0xf4, 0x96, 0xed, 0xff, 0xda, 0xd8, 0x82, 0xc1, 0x88, 0x12, 0x82, 0x02, 0xf9, 0x7c,
];
const EXPECTED_BAD_DETACHED_SIGNATURE_SHA256: [u8; 32] = [
    0xbf, 0x43, 0xd8, 0xcc, 0xc9, 0xe1, 0xc1, 0x8c, 0x4b, 0x16, 0x1f, 0x1a, 0x8e, 0xce, 0x4a, 0x66,
    0xa2, 0x8b, 0x52, 0x05, 0x25, 0xfe, 0xed, 0xbc, 0x8b, 0xcd, 0x8a, 0x98, 0x50, 0xe3, 0xbf, 0xa2,
];
const SYNTHETIC_MANIFEST_FINGERPRINT: &str = "AB1BEA3A37B7B55E01010FCF9FD95792A60A3222";
static ATTEMPT_COUNTER: AtomicU64 = AtomicU64::new(0);

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct ProtectedSelection {
    product: &'static str,
    version: &'static str,
    channel: &'static str,
    purpose: &'static str,
    policy_profile: &'static str,
    target_path: &'static str,
}

#[derive(Clone, Copy)]
enum NativeFixture {
    Valid,
    BadSignature,
    SignatureCheckDisabled,
}

struct NativeFixtureSelection {
    pairing: PathBuf,
    signature_sha256: [u8; 32],
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum GateOutcome {
    Accepted,
    Rejected,
}

#[derive(Clone, Copy)]
struct FakeRetainedCodiquaryPolicy {
    outcome: GateOutcome,
}

#[derive(Debug, PartialEq, Eq)]
enum NativeSignatureOutcome {
    Passed,
}

#[derive(Debug, Default, PartialEq, Eq)]
struct AttemptObserver {
    held_bytes: Option<Vec<u8>>,
    held_sha256: Option<[u8; 32]>,
    gate_input_bytes: Option<Vec<u8>>,
    gate_calls: usize,
    gate_outcome: Option<GateOutcome>,
    native_attempts: usize,
    consumer_calls: usize,
    consumer_bytes: Option<Vec<u8>>,
    consumer_sha256: Option<[u8; 32]>,
    evidence: Option<PathBuf>,
    http_evidence: Option<PathBuf>,
}

#[derive(Debug, PartialEq, Eq)]
struct AttemptObservation {
    selection: ProtectedSelection,
    artifact_bytes: Vec<u8>,
    artifact_sha256: [u8; 32],
    gate: GateOutcome,
    native_signature: NativeSignatureOutcome,
    consumer_calls: usize,
    consumer_sha256: Option<[u8; 32]>,
}

fn accepted_selection() -> ProtectedSelection {
    ProtectedSelection {
        product: "codiquary",
        version: "fddd7f564737ed5d213061a0c8621141bd85a2e9",
        channel: "file-fixture",
        purpose: "makepkg-source-verification",
        policy_profile: "experimental-pouf",
        target_path: "artifact.bin",
    }
}

fn accepted_policy() -> FakeRetainedCodiquaryPolicy {
    FakeRetainedCodiquaryPolicy {
        outcome: GateOutcome::Accepted,
    }
}

#[derive(Clone, Copy)]
struct TargetExpectations<'a> {
    selection: ProtectedSelection,
    bytes: &'a [u8],
    sha256: [u8; 32],
}

struct HeldTarget {
    selection: ProtectedSelection,
    bytes: Vec<u8>,
    sha256: [u8; 32],
    gate: GateOutcome,
}

fn makepkg_target_expectations() -> TargetExpectations<'static> {
    TargetExpectations {
        selection: accepted_selection(),
        bytes: EXPECTED_ARTIFACT_BYTES,
        sha256: EXPECTED_ARTIFACT_SHA256,
    }
}

fn pacman_selection() -> ProtectedSelection {
    ProtectedSelection {
        product: "codiquary",
        version: "1.0-1",
        channel: "file-fixture",
        purpose: "pacman-package-verification",
        policy_profile: "experimental-pouf",
        target_path: "codiquary17-native-1.0-1-any.pkg.tar.zst",
    }
}

fn native_fixture_selection(base: &Path, fixture: NativeFixture) -> NativeFixtureSelection {
    match fixture {
        NativeFixture::Valid => NativeFixtureSelection {
            pairing: base.to_path_buf(),
            signature_sha256: EXPECTED_DETACHED_SIGNATURE_SHA256,
        },
        NativeFixture::BadSignature => NativeFixtureSelection {
            pairing: base.join("bad-signature"),
            signature_sha256: EXPECTED_BAD_DETACHED_SIGNATURE_SHA256,
        },
        NativeFixture::SignatureCheckDisabled => NativeFixtureSelection {
            pairing: base.join("signature-check-disabled"),
            signature_sha256: EXPECTED_DETACHED_SIGNATURE_SHA256,
        },
    }
}

fn sha256_bytes(bytes: &[u8]) -> [u8; 32] {
    digest(&SHA256, bytes)
        .as_ref()
        .try_into()
        .expect("SHA-256 has a fixed 32-byte output")
}

fn sha256_hex(bytes: &[u8]) -> String {
    sha256_bytes(bytes)
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect()
}

fn require_file_sha256(
    path: &Path,
    expected: [u8; 32],
) -> Result<(), Box<dyn Error + Send + Sync + 'static>> {
    let actual = sha256_bytes(&fs::read(path)?);
    if actual != expected {
        return Err(format!("fixture hash did not match for {}", path.display()).into());
    }
    Ok(())
}

fn create_dir(path: &Path, mode: u32) -> Result<(), Box<dyn Error + Send + Sync + 'static>> {
    fs::create_dir(path)?;
    fs::set_permissions(path, fs::Permissions::from_mode(mode))?;
    Ok(())
}

fn create_empty_file(path: &Path, mode: u32) -> Result<(), Box<dyn Error + Send + Sync + 'static>> {
    let mut options = OpenOptions::new();
    options.write(true).create_new(true).mode(mode);
    options.open(path)?.sync_all()?;
    fs::set_permissions(path, fs::Permissions::from_mode(mode))?;
    Ok(())
}

fn retain_output(
    evidence: &Path,
    label: &str,
    output: &Output,
) -> Result<(), Box<dyn Error + Send + Sync + 'static>> {
    fs::write(evidence.join(format!("{label}.stdout")), &output.stdout)?;
    fs::write(evidence.join(format!("{label}.stderr")), &output.stderr)?;
    fs::write(
        evidence.join(format!("{label}.status")),
        format!(
            "success={}\ncode={:?}\n",
            output.status.success(),
            output.status.code()
        ),
    )?;
    Ok(())
}

fn clean_gpg_command(native: &Path) -> Command {
    let mut command = Command::new("/usr/bin/env");
    command.env_clear();
    command
        .arg("-i")
        .arg("PATH=/usr/bin:/bin")
        .arg("LANG=C")
        .arg("LC_ALL=C")
        .arg(format!("HOME={}", native.join("home").display()))
        .arg(format!("TMPDIR={}", native.join("tmp").display()))
        .arg(format!("GNUPGHOME={}", native.join("gnupg").display()))
        .arg("/usr/bin/gpg")
        .arg("--batch")
        .arg("--no-options")
        .arg("--homedir")
        .arg(native.join("gnupg"));
    command
}

fn primary_fingerprints(
    colon_output: &[u8],
) -> Result<Vec<String>, Box<dyn Error + Send + Sync + 'static>> {
    let text = std::str::from_utf8(colon_output)?;
    let mut expect_primary_fingerprint = false;
    let mut fingerprints = Vec::new();

    for line in text.lines() {
        let fields: Vec<&str> = line.split(':').collect();
        match fields.first().copied() {
            Some("pub") => expect_primary_fingerprint = true,
            Some("sub") => expect_primary_fingerprint = false,
            Some("fpr") if expect_primary_fingerprint => {
                let fingerprint = fields
                    .get(9)
                    .copied()
                    .ok_or("GPG fingerprint record was incomplete")?;
                fingerprints.push(fingerprint.to_owned());
                expect_primary_fingerprint = false;
            }
            _ => {}
        }
    }

    Ok(fingerprints)
}

fn copy_readonly(
    source: &Path,
    destination: &Path,
) -> Result<(), Box<dyn Error + Send + Sync + 'static>> {
    fs::copy(source, destination)?;
    fs::set_permissions(destination, fs::Permissions::from_mode(0o444))?;
    Ok(())
}

fn stage_readonly_harness(
    common_fixture: &Path,
    pairing: &Path,
    harness: &Path,
    held_bytes: &[u8],
) -> Result<(), Box<dyn Error + Send + Sync + 'static>> {
    create_dir(harness, 0o700)?;
    copy_readonly(&pairing.join("PKGBUILD"), &harness.join("PKGBUILD"))?;
    copy_readonly(
        &common_fixture.join("makepkg.conf"),
        &harness.join("makepkg.conf"),
    )?;
    copy_readonly(
        &pairing.join("artifact.bin.sig"),
        &harness.join("artifact.bin.sig"),
    )?;

    let artifact_path = harness.join("artifact.bin");
    let mut artifact = OpenOptions::new()
        .write(true)
        .create_new(true)
        .mode(0o400)
        .open(&artifact_path)?;
    artifact.write_all(held_bytes)?;
    artifact.sync_all()?;
    fs::set_permissions(&artifact_path, fs::Permissions::from_mode(0o444))?;
    fs::set_permissions(harness, fs::Permissions::from_mode(0o555))?;
    Ok(())
}

fn makepkg_command(native: &Path, harness: &Path) -> Command {
    let mut command = Command::new("/usr/bin/bwrap");
    command.env_clear();
    command
        .arg("--unshare-all")
        .arg("--die-with-parent")
        .arg("--new-session")
        .arg("--ro-bind")
        .arg("/usr")
        .arg("/usr")
        .arg("--symlink")
        .arg("usr/bin")
        .arg("/bin")
        .arg("--symlink")
        .arg("usr/bin")
        .arg("/sbin")
        .arg("--symlink")
        .arg("usr/lib")
        .arg("/lib")
        .arg("--symlink")
        .arg("usr/lib")
        .arg("/lib64")
        // The parent envelope supplies only its named, sanitized /etc files.
        .arg("--ro-bind")
        .arg("/etc")
        .arg("/etc")
        .arg("--proc")
        .arg("/proc")
        .arg("--dev")
        .arg("/dev")
        .arg("--tmpfs")
        .arg("/tmp")
        .arg("--tmpfs")
        .arg("/run")
        // The acquired artifact, signature, config, and PKGBUILD are all below
        // this actual readonly mount. Writable makepkg destinations are separate.
        .arg("--ro-bind")
        .arg(harness)
        .arg("/harness")
        .arg("--bind")
        .arg(native)
        .arg("/scratch")
        .arg("--chdir")
        .arg("/harness")
        .arg("/usr/bin/env")
        .arg("-i")
        .arg("PATH=/usr/bin:/bin")
        .arg("LANG=C")
        .arg("LC_ALL=C")
        .arg("HOME=/scratch/home")
        .arg("TMPDIR=/scratch/tmp")
        .arg("GNUPGHOME=/scratch/gnupg")
        .arg("/usr/bin/makepkg")
        .arg("--verifysource")
        .arg("--config")
        .arg("/harness/makepkg.conf")
        .arg("--dir")
        .arg("/harness");
    command
}

fn fake_retained_codiquary_policy_gate(
    policy: FakeRetainedCodiquaryPolicy,
    held_artifact: &[u8],
    observer: &mut AttemptObserver,
) -> GateOutcome {
    observer.gate_input_bytes = Some(held_artifact.to_vec());
    observer.gate_calls += 1;
    observer.gate_outcome = Some(policy.outcome);
    policy.outcome
}

fn fake_consume_once(artifact_bytes: Vec<u8>, observer: &mut AttemptObserver) -> Vec<u8> {
    observer.consumer_calls += 1;
    observer.consumer_bytes = Some(artifact_bytes.clone());
    observer.consumer_sha256 = Some(sha256_bytes(&artifact_bytes));
    artifact_bytes
}

async fn run_makepkg_fixture_attempt(
    selection: ProtectedSelection,
    fixture: &Path,
    native_fixture: NativeFixture,
    policy: FakeRetainedCodiquaryPolicy,
    observer: &mut AttemptObserver,
) -> Result<AttemptObservation, Box<dyn Error + Send + Sync + 'static>> {
    run_makepkg_fixture_attempt_with_held_bytes_mutation(
        selection,
        fixture,
        native_fixture,
        policy,
        None,
        observer,
    )
    .await
}

async fn run_makepkg_fixture_attempt_with_held_bytes_mutation(
    selection: ProtectedSelection,
    fixture: &Path,
    native_fixture: NativeFixture,
    policy: FakeRetainedCodiquaryPolicy,
    held_bytes_mutation: Option<fn(&mut Vec<u8>)>,
    observer: &mut AttemptObserver,
) -> Result<AttemptObservation, Box<dyn Error + Send + Sync + 'static>> {
    if selection != accepted_selection() {
        return Err("protected fixture selection did not match the accepted profile".into());
    }

    let root = fs::read(fixture.join("trusted-root.json"))?;
    let metadata = fixture.join("metadata").canonicalize()?;
    let targets = fixture.join("targets").canonicalize()?;
    let metadata_url = format!("file://{}/", metadata.display()).parse()?;
    let targets_url = format!("file://{}/", targets.display()).parse()?;

    run_makepkg_fixture_attempt_with_transport(
        selection,
        RepositoryLoadInputs {
            root: &root,
            metadata_url,
            targets_url,
        },
        makepkg_target_expectations(),
        FilesystemTransport,
        native_fixture,
        HeldTargetGateInputs {
            policy,
            held_bytes_mutation,
        },
        observer,
    )
    .await
}

struct RepositoryLoadInputs<'a> {
    root: &'a [u8],
    metadata_url: Url,
    targets_url: Url,
}

struct HeldTargetGateInputs {
    policy: FakeRetainedCodiquaryPolicy,
    held_bytes_mutation: Option<fn(&mut Vec<u8>)>,
}

async fn acquire_selected_held_bytes_with_transport(
    selection: ProtectedSelection,
    repository_inputs: RepositoryLoadInputs<'_>,
    expectations: TargetExpectations<'_>,
    transport: impl tough::Transport + 'static,
    policy: FakeRetainedCodiquaryPolicy,
    held_bytes_mutation: Option<fn(&mut Vec<u8>)>,
    observer: &mut AttemptObserver,
) -> Result<HeldTarget, Box<dyn Error + Send + Sync + 'static>> {
    if selection != expectations.selection {
        return Err("protected fixture selection did not match the accepted profile".into());
    }

    let RepositoryLoadInputs {
        root,
        metadata_url,
        targets_url,
    } = repository_inputs;
    let repository = RepositoryLoader::new(&root, metadata_url, targets_url)
        .transport(transport)
        .load()
        .await?;

    let target_name = TargetName::new(selection.target_path)?;
    let mut described_targets = repository.all_targets();
    let (described_name, described_target) = described_targets
        .next()
        .ok_or("verified metadata did not describe the selected target")?;
    if described_targets.next().is_some() {
        return Err("verified metadata described more than the selected fixture target".into());
    }
    if described_name != &target_name
        || described_target.length != expectations.bytes.len() as u64
        || described_target.hashes.sha256.as_ref() != &expectations.sha256[..]
    {
        return Err("verified target description did not match the protected selection".into());
    }

    let mut held_bytes = repository
        .read_target(&target_name)
        .await?
        .ok_or("selected target was absent")?
        .into_vec()
        .await?;
    observer.held_bytes = Some(held_bytes.clone());
    let held_sha256 = sha256_bytes(&held_bytes);
    observer.held_sha256 = Some(held_sha256);

    if held_bytes.as_slice() != expectations.bytes || held_sha256 != expectations.sha256 {
        return Err("fully collected target did not match the accepted artifact".into());
    }

    if let Some(mutate_held_bytes) = held_bytes_mutation {
        mutate_held_bytes(&mut held_bytes);
    }

    let gate = fake_retained_codiquary_policy_gate(policy, &held_bytes, observer);
    if gate == GateOutcome::Rejected {
        return Err("retained Codiquary policy rejected the held artifact".into());
    }

    Ok(HeldTarget {
        selection,
        bytes: held_bytes,
        sha256: held_sha256,
        gate,
    })
}

async fn run_makepkg_fixture_attempt_with_transport(
    selection: ProtectedSelection,
    repository_inputs: RepositoryLoadInputs<'_>,
    target_expectations: TargetExpectations<'_>,
    transport: impl tough::Transport + 'static,
    native_fixture: NativeFixture,
    gate_inputs: HeldTargetGateInputs,
    observer: &mut AttemptObserver,
) -> Result<AttemptObservation, Box<dyn Error + Send + Sync + 'static>> {
    let held = acquire_selected_held_bytes_with_transport(
        selection,
        repository_inputs,
        target_expectations,
        transport,
        gate_inputs.policy,
        gate_inputs.held_bytes_mutation,
        observer,
    )
    .await?;
    let selection = held.selection;
    let held_bytes = held.bytes;
    let held_sha256 = held.sha256;
    let gate = held.gate;

    let native_base = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/data/file-makepkg-native");
    let native_selection = native_fixture_selection(&native_base, native_fixture);
    require_file_sha256(
        &native_base.join("synthetic-signer-public.asc"),
        EXPECTED_PUBLIC_CERTIFICATE_SHA256,
    )?;
    require_file_sha256(
        &native_selection.pairing.join("artifact.bin.sig"),
        native_selection.signature_sha256,
    )?;

    let tmpdir = PathBuf::from(std::env::var_os("TMPDIR").ok_or("TMPDIR was not set")?);
    let attempt_id = ATTEMPT_COUNTER.fetch_add(1, Ordering::Relaxed);
    let scratch = tmpdir.join(format!(
        "codiquary-issue17-file-makepkg-{}-{attempt_id}",
        std::process::id()
    ));
    create_dir(&scratch, 0o700)?;
    let evidence = scratch.join("evidence");
    let native = scratch.join("native");
    let harness = scratch.join("readonly-harness");
    create_dir(&evidence, 0o700)?;
    observer.evidence = Some(evidence.clone());
    create_dir(&native, 0o700)?;
    for name in [
        "build",
        "package",
        "source",
        "source-package",
        "log",
        "tmp",
        "home",
        "gnupg",
    ] {
        create_dir(
            &native.join(name),
            if matches!(name, "home" | "gnupg") {
                0o700
            } else {
                0o755
            },
        )?;
    }
    create_empty_file(&native.join("gnupg/gpg.conf"), 0o600)?;
    create_empty_file(&native.join("gnupg/common.conf"), 0o600)?;

    let mut import_command = clean_gpg_command(&native);
    import_command
        .arg("--import")
        .arg(native_base.join("synthetic-signer-public.asc"));
    let import_output = import_command.output()?;
    retain_output(&evidence, "gpg-import", &import_output)?;
    if !import_output.status.success() {
        return Err(format!(
            "public-certificate import failed; evidence retained at {}",
            evidence.display()
        )
        .into());
    }

    let mut fingerprint_command = clean_gpg_command(&native);
    fingerprint_command
        .arg("--with-colons")
        .arg("--fingerprint")
        .arg("--list-keys");
    let fingerprint_output = fingerprint_command.output()?;
    retain_output(&evidence, "gpg-fingerprint", &fingerprint_output)?;
    if !fingerprint_output.status.success() {
        return Err(format!(
            "public-certificate fingerprint inspection failed; evidence retained at {}",
            evidence.display()
        )
        .into());
    }
    if primary_fingerprints(&fingerprint_output.stdout)?
        != [SYNTHETIC_MANIFEST_FINGERPRINT.to_owned()]
    {
        return Err("imported primary fingerprint did not match the frozen manifest".into());
    }

    stage_readonly_harness(
        &native_base,
        &native_selection.pairing,
        &harness,
        &held_bytes,
    )?;
    let staged_artifact = harness.join("artifact.bin");
    require_file_sha256(&staged_artifact, EXPECTED_ARTIFACT_SHA256)?;

    observer.native_attempts += 1;
    let makepkg_output = makepkg_command(&native, &harness).output()?;
    retain_output(&evidence, "makepkg-verifysource", &makepkg_output)?;
    require_file_sha256(&staged_artifact, EXPECTED_ARTIFACT_SHA256)?;

    if !makepkg_output.status.success() {
        return Err(format!(
            "native makepkg verification failed; evidence retained at {}",
            evidence.display()
        )
        .into());
    }
    let native_output = format!(
        "{}{}",
        String::from_utf8_lossy(&makepkg_output.stdout),
        String::from_utf8_lossy(&makepkg_output.stderr)
    );
    if !native_output.contains("Verifying source file signatures with gpg...")
        || !native_output.contains("artifact.bin ... Passed")
    {
        return Err(format!(
            "makepkg succeeded without the expected native signature observation; evidence retained at {}",
            evidence.display()
        )
        .into());
    }

    let artifact_bytes = fake_consume_once(held_bytes, observer);

    Ok(AttemptObservation {
        selection,
        artifact_bytes,
        artifact_sha256: held_sha256,
        gate,
        native_signature: NativeSignatureOutcome::Passed,
        consumer_calls: observer.consumer_calls,
        consumer_sha256: observer.consumer_sha256,
    })
}

#[tokio::test(flavor = "current_thread")]
async fn file_backed_tuf_target_reaches_makepkg_verified_fake_consumer(
) -> Result<(), Box<dyn Error + Send + Sync + 'static>> {
    let selection = accepted_selection();
    let fixture = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/data/publisher-client-binding");
    let mut observer = AttemptObserver::default();

    let actual = run_makepkg_fixture_attempt(
        selection,
        &fixture,
        NativeFixture::Valid,
        accepted_policy(),
        &mut observer,
    )
    .await?;

    assert_eq!(
        actual,
        AttemptObservation {
            selection: accepted_selection(),
            artifact_bytes: b"codiquary issue 20 fixture\n".to_vec(),
            artifact_sha256: EXPECTED_ARTIFACT_SHA256,
            gate: GateOutcome::Accepted,
            native_signature: NativeSignatureOutcome::Passed,
            consumer_calls: 1,
            consumer_sha256: Some(EXPECTED_ARTIFACT_SHA256),
        }
    );

    Ok(())
}

#[tokio::test(flavor = "current_thread")]
async fn file_backed_tuf_target_rejects_bad_native_signature_before_fake_consumer(
) -> Result<(), Box<dyn Error + Send + Sync + 'static>> {
    let selection = accepted_selection();
    let fixture = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/data/publisher-client-binding");
    let mut observer = AttemptObserver::default();

    let attempt = run_makepkg_fixture_attempt(
        selection,
        &fixture,
        NativeFixture::BadSignature,
        accepted_policy(),
        &mut observer,
    )
    .await;

    assert_eq!(observer.consumer_calls, 0);
    assert_eq!(observer.consumer_sha256, None);
    let error = attempt.expect_err("the damaged native signature was accepted");
    assert!(error
        .to_string()
        .contains("native makepkg verification failed; evidence retained at"));

    let evidence = observer
        .evidence
        .as_ref()
        .expect("the native attempt did not expose retained evidence");
    let status = fs::read_to_string(evidence.join("makepkg-verifysource.status"))?;
    let stdout = fs::read_to_string(evidence.join("makepkg-verifysource.stdout"))?;
    let stderr = fs::read_to_string(evidence.join("makepkg-verifysource.stderr"))?;

    assert!(status.contains("success=false\ncode=Some(1)\n"));
    assert!(stdout.contains("Verifying source file signatures with gpg..."));
    assert!(stderr.contains("artifact.bin.sig ... Passed"));
    assert!(
        stderr.contains("artifact.bin ... FAILED (bad signature from public key 94E148D20214A61A)")
    );

    Ok(())
}

#[tokio::test(flavor = "current_thread")]
async fn file_backed_tuf_rejects_protected_selection_substitutions_before_acquisition_or_native_attempt(
) -> Result<(), Box<dyn Error + Send + Sync + 'static>> {
    let fixture = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/data/publisher-client-binding");
    let substitutions = [
        (
            "product",
            ProtectedSelection {
                product: "other-product",
                ..accepted_selection()
            },
        ),
        (
            "version",
            ProtectedSelection {
                version: "0000000000000000000000000000000000000000",
                ..accepted_selection()
            },
        ),
        (
            "channel",
            ProtectedSelection {
                channel: "other-channel",
                ..accepted_selection()
            },
        ),
        (
            "purpose",
            ProtectedSelection {
                purpose: "other-purpose",
                ..accepted_selection()
            },
        ),
        (
            "policy_profile",
            ProtectedSelection {
                policy_profile: "other-policy-profile",
                ..accepted_selection()
            },
        ),
        (
            "target_path",
            ProtectedSelection {
                target_path: "other-artifact.bin",
                ..accepted_selection()
            },
        ),
    ];

    for (field, selection) in substitutions {
        let mut observer = AttemptObserver::default();
        let attempt = run_makepkg_fixture_attempt(
            selection,
            &fixture,
            NativeFixture::Valid,
            accepted_policy(),
            &mut observer,
        )
        .await;

        assert_eq!(observer.held_bytes, None, "{field} substitution");
        assert_eq!(observer.held_sha256, None, "{field} substitution");
        assert_eq!(observer.native_attempts, 0, "{field} substitution");
        assert_eq!(observer.consumer_calls, 0, "{field} substitution");
        assert_eq!(observer.consumer_sha256, None, "{field} substitution");
        assert_eq!(observer.evidence, None, "{field} substitution");

        let error = attempt.expect_err("protected selection substitution crossed the boundary");
        assert_eq!(
            error.to_string(),
            "protected fixture selection did not match the accepted profile",
            "{field} substitution"
        );
    }

    Ok(())
}

#[tokio::test(flavor = "current_thread")]
async fn file_backed_tuf_target_rejects_retained_codiquary_policy_before_native_or_fake_consumer(
) -> Result<(), Box<dyn Error + Send + Sync + 'static>> {
    let selection = accepted_selection();
    let fixture = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/data/publisher-client-binding");
    let policy = FakeRetainedCodiquaryPolicy {
        outcome: GateOutcome::Rejected,
    };
    let mut observer = AttemptObserver::default();

    let attempt = run_makepkg_fixture_attempt(
        selection,
        &fixture,
        NativeFixture::Valid,
        policy,
        &mut observer,
    )
    .await;

    assert_eq!(
        observer.held_bytes.as_deref(),
        Some(EXPECTED_ARTIFACT_BYTES)
    );
    assert_eq!(observer.held_sha256, Some(EXPECTED_ARTIFACT_SHA256));
    assert_eq!(
        observer.gate_input_bytes.as_deref(),
        Some(EXPECTED_ARTIFACT_BYTES)
    );
    assert_eq!(observer.gate_calls, 1);
    assert_eq!(observer.gate_outcome, Some(GateOutcome::Rejected));
    assert_eq!(observer.native_attempts, 0);
    assert_eq!(observer.consumer_calls, 0);
    assert_eq!(observer.consumer_sha256, None);
    assert_eq!(observer.evidence, None);

    let error = attempt.expect_err("the retained Codiquary policy rejection was ignored");
    assert_eq!(
        error.to_string(),
        "retained Codiquary policy rejected the held artifact"
    );

    Ok(())
}

#[tokio::test(flavor = "current_thread")]
async fn file_backed_tuf_completed_held_bytes_mutation_stops_before_native_verification_or_fake_consumer(
) -> Result<(), Box<dyn Error + Send + Sync + 'static>> {
    let selection = accepted_selection();
    let fixture = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/data/publisher-client-binding");
    let mut observer = AttemptObserver::default();

    let attempt = run_makepkg_fixture_attempt_with_held_bytes_mutation(
        selection,
        &fixture,
        NativeFixture::Valid,
        accepted_policy(),
        Some(|held_bytes| held_bytes[0] ^= 0x01),
        &mut observer,
    )
    .await;

    assert_eq!(
        observer.held_bytes.as_deref(),
        Some(EXPECTED_ARTIFACT_BYTES)
    );
    assert_eq!(observer.held_sha256, Some(EXPECTED_ARTIFACT_SHA256));
    assert_eq!(
        observer.gate_input_bytes.as_deref(),
        Some(b"bodiquary issue 20 fixture\n".as_slice())
    );
    assert_eq!(observer.gate_calls, 1);
    assert_eq!(observer.gate_outcome, Some(GateOutcome::Accepted));
    assert_eq!(observer.consumer_calls, 0);
    assert_eq!(observer.consumer_sha256, None);
    assert_eq!(observer.native_attempts, 0);
    assert!(observer.evidence.is_some());

    let error = attempt.expect_err("mutated held bytes reached the fake consumer");
    let error = error.to_string();
    assert!(error.starts_with("fixture hash did not match for "));
    assert!(error.ends_with("readonly-harness/artifact.bin"));

    Ok(())
}

#[derive(Debug)]
enum PacmanPipelineError {
    NativeRequestChangedHeldBytes,
    RetainedEvidenceChanged,
    Native(NativeProbeError),
}

fn verify_file_pacman_before_consumer(
    request: &NativeProbeRequest<'_>,
    original_held_bytes: &[u8],
    observer: &mut AttemptObserver,
) -> Result<NativePositiveEvidence, PacmanPipelineError> {
    if request.held_package.bytes != original_held_bytes {
        return Err(PacmanPipelineError::NativeRequestChangedHeldBytes);
    }
    if observer.held_bytes.as_deref() != Some(original_held_bytes)
        || observer.held_sha256 != Some(sha256_bytes(original_held_bytes))
        || observer.gate_input_bytes.as_deref() != Some(original_held_bytes)
        || observer.gate_calls != 1
        || observer.gate_outcome != Some(GateOutcome::Accepted)
    {
        return Err(PacmanPipelineError::RetainedEvidenceChanged);
    }

    observer.native_attempts += 1;
    verify_native_before_consumer(request, || {
        fake_consumer(&mut observer.consumer_calls);
        observer.consumer_bytes = Some(original_held_bytes.to_vec());
        observer.consumer_sha256 = Some(sha256_bytes(original_held_bytes));
    })
    .map_err(PacmanPipelineError::Native)
}

#[tokio::test(flavor = "current_thread")]
async fn file_backed_published_pacman_target_reaches_native_verified_fake_consumer(
) -> Result<(), Box<dyn Error + Send + Sync + 'static>> {
    let native_fixture_dir =
        PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/data/pacman-native");
    let package_fixture = PackageFixture::Original;
    let package_bytes = fs::read(native_fixture_dir.join(package_fixture.filename()))?;
    assert_eq!(package_bytes.len(), 124, "the frozen package size changed");
    assert_eq!(
        sha256_hex(&package_bytes),
        package_fixture.sha256(),
        "the frozen package digest changed"
    );

    let selection = pacman_selection();
    let published = publish_single_delegated_target(selection.target_path, &package_bytes).await?;
    let tmpdir = PathBuf::from(std::env::var_os("TMPDIR").ok_or("TMPDIR was not set")?);
    let attempt_id = ATTEMPT_COUNTER.fetch_add(1, Ordering::Relaxed);
    let repository_dir = tmpdir.join(format!(
        "codiquary-file-pacman-{}-{attempt_id}",
        std::process::id()
    ));
    let metadata = repository_dir.join("metadata");
    let targets = repository_dir.join("targets");
    create_dir(&repository_dir, 0o700)?;
    create_dir(&metadata, 0o700)?;
    create_dir(&targets, 0o700)?;
    for (filename, bytes) in &published.metadata_by_filename {
        fs::write(metadata.join(filename), bytes)?;
    }
    fs::write(
        targets.join(&published.consistent_snapshot_target_path),
        &published.target_bytes,
    )?;

    let metadata_url = format!("file://{}/", metadata.canonicalize()?.display()).parse()?;
    let targets_url = format!("file://{}/", targets.canonicalize()?.display()).parse()?;
    let package_sha256 = sha256_bytes(&package_bytes);
    let mut observer = AttemptObserver::default();
    let held = acquire_selected_held_bytes_with_transport(
        selection,
        RepositoryLoadInputs {
            root: &published.public_root,
            metadata_url,
            targets_url,
        },
        TargetExpectations {
            selection,
            bytes: &package_bytes,
            sha256: package_sha256,
        },
        FilesystemTransport,
        accepted_policy(),
        None,
        &mut observer,
    )
    .await?;

    assert_eq!(held.selection, selection);
    assert_eq!(held.bytes, package_bytes);
    assert_eq!(held.sha256, package_sha256);
    assert_eq!(held.gate, GateOutcome::Accepted);
    assert_eq!(
        observer.held_bytes.as_deref(),
        Some(package_bytes.as_slice())
    );
    assert_eq!(
        observer.gate_input_bytes.as_deref(),
        Some(package_bytes.as_slice())
    );
    assert_eq!(observer.gate_calls, 1);
    assert_eq!(observer.native_attempts, 0);
    assert_eq!(observer.consumer_calls, 0);

    let request = NativeProbeRequest {
        fixture_dir: native_fixture_dir,
        scratch_dir: std::env::temp_dir().join("codiquary17-native").join("run"),
        held_package: HeldPackageBytes {
            expected_sha256: package_fixture.sha256(),
            bytes: &held.bytes,
        },
        database_fixture: DatabaseFixture::Original,
        package_signature_check: PackageSignatureCheck::Required,
        database_signature_check: DatabaseSignatureCheck::Required,
        primary_certificate_fingerprint: PRIMARY_CERTIFICATE_FINGERPRINT,
        signing_subkey_fingerprint: SIGNING_SUBKEY_FINGERPRINT,
        expected_native_signature_fingerprint: EXPECTED_NATIVE_SIGNATURE_FINGERPRINT,
    };
    let native_evidence = verify_file_pacman_before_consumer(&request, &held.bytes, &mut observer)
        .map_err(|error| -> Box<dyn Error + Send + Sync + 'static> {
            format!("pacman pipeline rejected: {error:?}").into()
        })?;

    assert_eq!(native_evidence, expected_positive_evidence());
    assert_eq!(observer.consumer_calls, 1);
    assert_eq!(observer.consumer_sha256, Some(package_sha256));

    Ok(())
}

#[tokio::test(flavor = "current_thread")]
async fn file_backed_published_pacman_target_rejects_native_failures_and_disabled_mandatory_checks_before_fake_consumer(
) -> Result<(), Box<dyn Error + Send + Sync + 'static>> {
    let native_fixture_dir =
        PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/data/pacman-native");
    let cases = [
        (
            "mtime-changed package with original signature",
            PackageFixture::MtimeChangedWithOriginalSignature,
            DatabaseFixture::Original,
            PackageSignatureCheck::Required,
            DatabaseSignatureCheck::Required,
            "native package signature verification",
        ),
        (
            "mtime-changed database with original signature",
            PackageFixture::Original,
            DatabaseFixture::MtimeChangedWithOriginalSignature,
            PackageSignatureCheck::Required,
            DatabaseSignatureCheck::Required,
            "native database signature verification",
        ),
        (
            "disabled package signature check",
            PackageFixture::Original,
            DatabaseFixture::Original,
            PackageSignatureCheck::Disabled,
            DatabaseSignatureCheck::Required,
            "caller mandatory package-check enforcement",
        ),
        (
            "disabled database signature check",
            PackageFixture::Original,
            DatabaseFixture::Original,
            PackageSignatureCheck::Required,
            DatabaseSignatureCheck::Disabled,
            "caller mandatory database-check enforcement",
        ),
    ];

    for (
        case,
        package_fixture,
        database_fixture,
        package_signature_check,
        database_signature_check,
        expected_stage,
    ) in cases
    {
        let package_bytes = fs::read(native_fixture_dir.join(package_fixture.filename()))?;
        let selection = pacman_selection();
        let published =
            publish_single_delegated_target(selection.target_path, &package_bytes).await?;
        let attempt_id = ATTEMPT_COUNTER.fetch_add(1, Ordering::Relaxed);
        let repository_dir = std::env::temp_dir().join(format!(
            "codiquary-file-pacman-rejection-{}-{attempt_id}",
            std::process::id()
        ));
        let metadata = repository_dir.join("metadata");
        let targets = repository_dir.join("targets");
        create_dir(&repository_dir, 0o700)?;
        create_dir(&metadata, 0o700)?;
        create_dir(&targets, 0o700)?;
        for (filename, bytes) in &published.metadata_by_filename {
            fs::write(metadata.join(filename), bytes)?;
        }
        fs::write(
            targets.join(&published.consistent_snapshot_target_path),
            &published.target_bytes,
        )?;

        let package_sha256 = sha256_bytes(&package_bytes);
        let mut observer = AttemptObserver::default();
        let held = acquire_selected_held_bytes_with_transport(
            selection,
            RepositoryLoadInputs {
                root: &published.public_root,
                metadata_url: format!("file://{}/", metadata.canonicalize()?.display()).parse()?,
                targets_url: format!("file://{}/", targets.canonicalize()?.display()).parse()?,
            },
            TargetExpectations {
                selection,
                bytes: &package_bytes,
                sha256: package_sha256,
            },
            FilesystemTransport,
            accepted_policy(),
            None,
            &mut observer,
        )
        .await?;

        let request = NativeProbeRequest {
            fixture_dir: native_fixture_dir.clone(),
            scratch_dir: std::env::temp_dir().join("codiquary17-native").join("run"),
            held_package: HeldPackageBytes {
                expected_sha256: package_fixture.sha256(),
                bytes: &held.bytes,
            },
            database_fixture,
            package_signature_check,
            database_signature_check,
            primary_certificate_fingerprint: PRIMARY_CERTIFICATE_FINGERPRINT,
            signing_subkey_fingerprint: SIGNING_SUBKEY_FINGERPRINT,
            expected_native_signature_fingerprint: EXPECTED_NATIVE_SIGNATURE_FINGERPRINT,
        };

        let observed_stage = match verify_file_pacman_before_consumer(
            &request,
            &held.bytes,
            &mut observer,
        ) {
            Err(PacmanPipelineError::Native(NativeProbeError::Rejected {
                attempt_dir,
                receipt,
            })) => {
                assert_eq!(receipt.status_code, Some(1), "{case}");
                assert!(receipt.stdout.is_empty(), "{case}");
                let stderr = String::from_utf8(receipt.stderr)?;
                if stderr.starts_with("alpm_pkg_load returned -1") {
                    "native package signature verification"
                } else if stderr.starts_with("alpm_db_get_valid:") {
                    "native database signature verification"
                } else {
                    panic!(
                        "{case}: native rejection occurred at an unexpected stage: {stderr}; retained attempt {}",
                        attempt_dir.display()
                    );
                }
            }
            Err(PacmanPipelineError::Native(NativeProbeError::UnsafeNativeEvidence {
                evidence,
            })) if evidence.package.configured_siglevel == 0
                && evidence.database.configured_siglevel == REQUIRED_DATABASE_SIGLEVEL =>
            {
                "caller mandatory package-check enforcement"
            }
            Err(PacmanPipelineError::Native(NativeProbeError::UnsafeNativeEvidence {
                evidence,
            })) if evidence.package.configured_siglevel == REQUIRED_PACKAGE_SIGLEVEL
                && evidence.database.configured_siglevel == 0 =>
            {
                "caller mandatory database-check enforcement"
            }
            Err(PacmanPipelineError::Native(NativeProbeError::Failed {
                attempt_dir,
                message,
            })) => panic!(
                "{case}: pacman native setup failed; retained attempt {}: {message}",
                attempt_dir.display()
            ),
            Err(error) => {
                panic!("{case}: pacman pipeline rejected at an unexpected stage: {error:?}")
            }
            Ok(evidence) => panic!("{case}: pacman pipeline unexpectedly accepted: {evidence:?}"),
        };

        assert_eq!(observed_stage, expected_stage, "{case}");
        assert_eq!(held.selection, selection, "{case}");
        assert_eq!(held.bytes.as_slice(), package_bytes.as_slice(), "{case}");
        assert_eq!(
            observer.held_bytes.as_deref(),
            Some(package_bytes.as_slice()),
            "{case}"
        );
        assert_eq!(observer.held_sha256, Some(package_sha256), "{case}");
        assert_eq!(
            observer.gate_input_bytes.as_deref(),
            Some(package_bytes.as_slice()),
            "{case}"
        );
        assert_eq!(observer.gate_calls, 1, "{case}");
        assert_eq!(observer.gate_outcome, Some(GateOutcome::Accepted), "{case}");
        assert_eq!(observer.native_attempts, 1, "{case}");
        assert_eq!(observer.consumer_calls, 0, "{case}");
        assert_eq!(observer.consumer_bytes, None, "{case}");
        assert_eq!(observer.consumer_sha256, None, "{case}");
    }

    Ok(())
}

use support::curl_fixture_transport::{CurlFetchEvidence, CurlFixtureTransport};
use support::loopback_http_fixture::{HttpExchangeRecord, HttpFixtureRoute, LoopbackHttpFixture};

fn retain_curl_fetch_evidence(
    scratch_dir: &Path,
    origin: &Url,
    entries: &[CurlFetchEvidence],
) -> Result<(), Box<dyn Error + Send + Sync + 'static>> {
    for (expected_sequence, entry) in entries.iter().enumerate() {
        if entry.sequence != expected_sequence as u64 {
            return Err("curl fixture evidence was not recorded in fetch order".into());
        }
        if !entry.request_url.starts_with(origin.as_str()) {
            return Err("curl fixture evidence contained a request outside its origin".into());
        }
        if entry.body_path.parent() != Some(scratch_dir) {
            return Err("curl fixture response body escaped its evidence directory".into());
        }

        let label = format!("curl-fetch-{:020}", entry.sequence);
        fs::write(
            scratch_dir.join(format!("{label}.status")),
            format!(
                "request_url={}\nbody_path={}\ncurl_status={:?}\nhttp_status={:?}\ninvocation_error={:?}\nbody_read_error={:?}\n",
                entry.request_url,
                entry.body_path.display(),
                entry.curl_status,
                entry.http_status,
                entry.invocation_error,
                entry.body_read_error,
            ),
        )?;
        fs::write(scratch_dir.join(format!("{label}.stdout")), &entry.stdout)?;
        fs::write(scratch_dir.join(format!("{label}.stderr")), &entry.stderr)?;

        if entry.invocation_error.is_none()
            && !entry
                .curl_status
                .as_ref()
                .is_some_and(|status| status.success())
        {
            return Err("curl fixture process did not exit successfully".into());
        }
        if let Some(http_status) = entry.http_status {
            if entry.stdout != format!("{http_status:03}").as_bytes() {
                return Err("curl fixture HTTP status disagreed with write-out evidence".into());
            }
        }
        if entry.body_read_error.is_none() && fs::read(&entry.body_path)? != entry.body {
            return Err("curl fixture response file disagreed with retained body evidence".into());
        }
    }

    Ok(())
}

async fn run_loopback_http_fixture_attempt<T, F, Fut>(
    routes: impl IntoIterator<Item = HttpFixtureRoute>,
    curl_scratch: PathBuf,
    http_exchanges: &mut Vec<HttpExchangeRecord>,
    execute: F,
) -> Result<T, Box<dyn Error + Send + Sync + 'static>>
where
    F: FnOnce(CurlFixtureTransport, Url) -> Fut,
    Fut: Future<Output = Result<T, Box<dyn Error + Send + Sync + 'static>>>,
{
    let server = LoopbackHttpFixture::start(routes)?;
    let (attempt, curl_evidence, origin) =
        match CurlFixtureTransport::new(server.address(), curl_scratch.clone()) {
            Ok(transport) => {
                let origin = transport.origin();
                let attempt = execute(transport.clone(), origin.clone()).await;
                let evidence = transport.evidence();
                (attempt, evidence, Some(origin))
            }
            Err(error) => (
                Err(Box::new(error) as Box<dyn Error + Send + Sync + 'static>),
                Vec::new(),
                None,
            ),
        };

    let server_join = server.stop_and_join();
    let http_retention = match &server_join {
        Ok(evidence) => {
            http_exchanges.extend(evidence.exchanges.iter().cloned());
            evidence.exchanges.iter().enumerate().try_for_each(
                |(sequence, exchange)| -> std::io::Result<()> {
                    let label = format!("http-exchange-{sequence:020}");
                    fs::write(
                        curl_scratch.join(format!("{label}.request")),
                        &exchange.request_bytes,
                    )?;
                    fs::write(
                        curl_scratch.join(format!("{label}.response")),
                        &exchange.response_bytes,
                    )?;
                    Ok(())
                },
            )
        }
        Err(_) => Ok(()),
    };
    let curl_retention = match origin.as_ref() {
        Some(origin) => retain_curl_fetch_evidence(&curl_scratch, origin, &curl_evidence),
        None => Ok(()),
    };

    let server_evidence = server_join?;
    if !server_evidence.server_errors.is_empty() {
        return Err(format!(
            "loopback HTTP fixture recorded server errors: {}",
            server_evidence.server_errors.join("; ")
        )
        .into());
    }
    curl_retention?;
    http_retention?;
    attempt
}

async fn run_http_makepkg_fixture_attempt(
    selection: ProtectedSelection,
    fixture: &Path,
    native_fixture: NativeFixture,
    policy: FakeRetainedCodiquaryPolicy,
    observer: &mut AttemptObserver,
    http_exchanges: &mut Vec<HttpExchangeRecord>,
    served_target_bytes: Option<&[u8]>,
) -> Result<AttemptObservation, Box<dyn Error + Send + Sync + 'static>> {
    if selection != accepted_selection() {
        return Err("protected fixture selection did not match the accepted profile".into());
    }

    let root = fs::read(fixture.join("trusted-root.json"))?;
    let target_response_body = match served_target_bytes {
        Some(bytes) => bytes.to_vec(),
        None => fs::read(fixture.join(
            "targets/d13ecc865c37b23650615038c232eccfa5770c8bc345dbe98db595273fecda4a.artifact.bin",
        ))?,
    };
    let routes = [
        HttpFixtureRoute::new(
            "/metadata/1.snapshot.json",
            fs::read(fixture.join("metadata/1.snapshot.json"))?,
        ),
        HttpFixtureRoute::new(
            "/metadata/1.targets.json",
            fs::read(fixture.join("metadata/1.targets.json"))?,
        ),
        HttpFixtureRoute::new(
            "/metadata/1.delegated.json",
            fs::read(fixture.join("metadata/1.delegated.json"))?,
        ),
        HttpFixtureRoute::new(
            "/metadata/timestamp.json",
            fs::read(fixture.join("metadata/timestamp.json"))?,
        ),
        HttpFixtureRoute::new(
            "/targets/d13ecc865c37b23650615038c232eccfa5770c8bc345dbe98db595273fecda4a.artifact.bin",
            target_response_body,
        ),
    ];

    let tmpdir = PathBuf::from(std::env::var_os("TMPDIR").ok_or("TMPDIR was not set")?);
    if !tmpdir.is_absolute() {
        return Err("TMPDIR was not absolute".into());
    }
    let attempt_id = ATTEMPT_COUNTER.fetch_add(1, Ordering::Relaxed);
    let curl_scratch = tmpdir.join(format!(
        "codiquary-issue17-loopback-http-{}-{attempt_id}",
        std::process::id()
    ));
    create_dir(&curl_scratch, 0o700)?;
    observer.http_evidence = Some(curl_scratch.clone());

    run_loopback_http_fixture_attempt(
        routes,
        curl_scratch,
        http_exchanges,
        move |transport, origin| async move {
            let metadata_url = origin.join("metadata/")?;
            let targets_url = origin.join("targets/")?;
            run_makepkg_fixture_attempt_with_transport(
                selection,
                RepositoryLoadInputs {
                    root: &root,
                    metadata_url,
                    targets_url,
                },
                makepkg_target_expectations(),
                transport,
                native_fixture,
                HeldTargetGateInputs {
                    policy,
                    held_bytes_mutation: None,
                },
                observer,
            )
            .await
        },
    )
    .await
}

#[tokio::test(flavor = "current_thread")]
async fn loopback_http_published_pacman_target_reaches_native_verified_fake_consumer(
) -> Result<(), Box<dyn Error + Send + Sync + 'static>> {
    let native_fixture_dir =
        PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/data/pacman-native");
    let package_fixture = PackageFixture::Original;
    let package_bytes = fs::read(native_fixture_dir.join(package_fixture.filename()))?;
    assert_eq!(package_bytes.len(), 124, "the frozen package size changed");
    assert_eq!(
        sha256_hex(&package_bytes),
        package_fixture.sha256(),
        "the frozen package digest changed"
    );

    let selection = pacman_selection();
    let published = publish_single_delegated_target(selection.target_path, &package_bytes).await?;
    let mut routes = published
        .metadata_by_filename
        .iter()
        .map(|(filename, bytes)| {
            HttpFixtureRoute::new(format!("/metadata/{filename}"), bytes.clone())
        })
        .collect::<Vec<_>>();
    routes.push(HttpFixtureRoute::new(
        format!("/targets/{}", published.consistent_snapshot_target_path),
        published.target_bytes.clone(),
    ));

    let tmpdir = PathBuf::from(std::env::var_os("TMPDIR").ok_or("TMPDIR was not set")?);
    if !tmpdir.is_absolute() {
        return Err("TMPDIR was not absolute".into());
    }
    let attempt_id = ATTEMPT_COUNTER.fetch_add(1, Ordering::Relaxed);
    let curl_scratch = tmpdir.join(format!(
        "codiquary-pacman-loopback-http-{}-{attempt_id}",
        std::process::id()
    ));
    create_dir(&curl_scratch, 0o700)?;

    let package_sha256 = sha256_bytes(&package_bytes);
    let mut observer = AttemptObserver {
        http_evidence: Some(curl_scratch.clone()),
        ..AttemptObserver::default()
    };
    let mut http_exchanges = Vec::new();
    let held = {
        let root = published.public_root.as_slice();
        let expected_package_bytes = package_bytes.as_slice();
        let observer = &mut observer;
        run_loopback_http_fixture_attempt(
            routes,
            curl_scratch,
            &mut http_exchanges,
            move |transport, origin| async move {
                let metadata_url = origin.join("metadata/")?;
                let targets_url = origin.join("targets/")?;
                acquire_selected_held_bytes_with_transport(
                    selection,
                    RepositoryLoadInputs {
                        root,
                        metadata_url,
                        targets_url,
                    },
                    TargetExpectations {
                        selection,
                        bytes: expected_package_bytes,
                        sha256: package_sha256,
                    },
                    transport,
                    accepted_policy(),
                    None,
                    observer,
                )
                .await
            },
        )
        .await?
    };

    assert_eq!(held.selection, selection);
    assert_eq!(held.bytes.as_slice(), package_bytes.as_slice());
    assert_eq!(held.sha256, package_sha256);
    assert_eq!(held.gate, GateOutcome::Accepted);
    assert_eq!(
        observer.held_bytes.as_deref(),
        Some(package_bytes.as_slice())
    );
    assert_eq!(observer.held_sha256, Some(package_sha256));
    assert_eq!(
        observer.gate_input_bytes.as_deref(),
        Some(package_bytes.as_slice())
    );
    assert_eq!(observer.gate_calls, 1);
    assert_eq!(observer.gate_outcome, Some(GateOutcome::Accepted));
    assert_eq!(observer.native_attempts, 0);
    assert_eq!(observer.consumer_calls, 0);
    assert_eq!(observer.consumer_bytes, None);
    assert_eq!(observer.consumer_sha256, None);

    let request = NativeProbeRequest {
        fixture_dir: native_fixture_dir,
        scratch_dir: std::env::temp_dir().join("codiquary17-native").join("run"),
        held_package: HeldPackageBytes {
            expected_sha256: package_fixture.sha256(),
            bytes: &held.bytes,
        },
        database_fixture: DatabaseFixture::Original,
        package_signature_check: PackageSignatureCheck::Required,
        database_signature_check: DatabaseSignatureCheck::Required,
        primary_certificate_fingerprint: PRIMARY_CERTIFICATE_FINGERPRINT,
        signing_subkey_fingerprint: SIGNING_SUBKEY_FINGERPRINT,
        expected_native_signature_fingerprint: EXPECTED_NATIVE_SIGNATURE_FINGERPRINT,
    };
    let native_evidence = verify_file_pacman_before_consumer(&request, &held.bytes, &mut observer)
        .map_err(|error| -> Box<dyn Error + Send + Sync + 'static> {
            format!("pacman pipeline rejected: {error:?}").into()
        })?;

    assert_eq!(native_evidence, expected_positive_evidence());
    assert_eq!(held.bytes.as_slice(), package_bytes.as_slice());
    assert_eq!(observer.gate_calls, 1);
    assert_eq!(observer.native_attempts, 1);
    assert_eq!(observer.consumer_calls, 1);
    assert_eq!(
        observer.consumer_bytes.as_deref(),
        Some(package_bytes.as_slice())
    );
    assert_eq!(observer.consumer_sha256, Some(package_sha256));

    let expected_http_exchanges = vec![
        (
            "/metadata/2.root.json".to_owned(),
            "404 Not Found",
            Vec::new(),
        ),
        (
            "/metadata/timestamp.json".to_owned(),
            "200 OK",
            published
                .metadata_by_filename
                .get("timestamp.json")
                .ok_or("generated timestamp metadata was absent")?
                .clone(),
        ),
        (
            "/metadata/1.snapshot.json".to_owned(),
            "200 OK",
            published
                .metadata_by_filename
                .get("1.snapshot.json")
                .ok_or("generated snapshot metadata was absent")?
                .clone(),
        ),
        (
            "/metadata/1.targets.json".to_owned(),
            "200 OK",
            published
                .metadata_by_filename
                .get("1.targets.json")
                .ok_or("generated targets metadata was absent")?
                .clone(),
        ),
        (
            "/metadata/1.delegated.json".to_owned(),
            "200 OK",
            published
                .metadata_by_filename
                .get("1.delegated.json")
                .ok_or("generated delegated metadata was absent")?
                .clone(),
        ),
        (
            format!("/targets/{}", published.consistent_snapshot_target_path),
            "200 OK",
            package_bytes.clone(),
        ),
    ];
    assert_http_exchange_sequence_from_expected(&http_exchanges, expected_http_exchanges)?;
    let http_evidence = observer
        .http_evidence
        .as_deref()
        .expect("the pacman HTTP attempt did not expose retained exchange evidence");
    assert_retained_http_exchange_bytes(http_evidence, &http_exchanges)?;

    Ok(())
}

#[tokio::test(flavor = "current_thread")]
async fn loopback_http_tuf_target_reaches_makepkg_verified_fake_consumer(
) -> Result<(), Box<dyn Error + Send + Sync + 'static>> {
    let selection = accepted_selection();
    let fixture = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/data/publisher-client-binding");
    let mut observer = AttemptObserver::default();
    let mut http_exchanges = Vec::new();

    let actual = run_http_makepkg_fixture_attempt(
        selection,
        &fixture,
        NativeFixture::Valid,
        accepted_policy(),
        &mut observer,
        &mut http_exchanges,
        None,
    )
    .await?;

    assert_eq!(
        actual,
        AttemptObservation {
            selection: accepted_selection(),
            artifact_bytes: EXPECTED_ARTIFACT_BYTES.to_vec(),
            artifact_sha256: EXPECTED_ARTIFACT_SHA256,
            gate: GateOutcome::Accepted,
            native_signature: NativeSignatureOutcome::Passed,
            consumer_calls: 1,
            consumer_sha256: Some(EXPECTED_ARTIFACT_SHA256),
        }
    );
    assert_eq!(
        observer.held_bytes.as_deref(),
        Some(EXPECTED_ARTIFACT_BYTES)
    );
    assert_eq!(observer.held_sha256, Some(EXPECTED_ARTIFACT_SHA256));
    assert_eq!(
        observer.gate_input_bytes.as_deref(),
        Some(EXPECTED_ARTIFACT_BYTES)
    );
    assert_eq!(observer.gate_calls, 1);
    assert_eq!(observer.gate_outcome, Some(GateOutcome::Accepted));
    assert_eq!(observer.native_attempts, 1);
    assert_eq!(observer.consumer_calls, 1);
    assert_eq!(observer.consumer_sha256, Some(EXPECTED_ARTIFACT_SHA256));
    assert!(observer.evidence.is_some());

    assert_http_exchange_sequence(&http_exchanges, &fixture, EXPECTED_ARTIFACT_BYTES)?;

    Ok(())
}

fn assert_http_exchange_sequence(
    exchanges: &[HttpExchangeRecord],
    fixture: &Path,
    expected_target_body: &[u8],
) -> Result<(), Box<dyn Error + Send + Sync + 'static>> {
    assert_http_exchange_sequence_from_expected(
        exchanges,
        vec![
            ("/metadata/2.root.json".to_owned(), "404 Not Found", Vec::new()),
            (
                "/metadata/timestamp.json".to_owned(),
                "200 OK",
                fs::read(fixture.join("metadata/timestamp.json"))?,
            ),
            (
                "/metadata/1.snapshot.json".to_owned(),
                "200 OK",
                fs::read(fixture.join("metadata/1.snapshot.json"))?,
            ),
            (
                "/metadata/1.targets.json".to_owned(),
                "200 OK",
                fs::read(fixture.join("metadata/1.targets.json"))?,
            ),
            (
                "/metadata/1.delegated.json".to_owned(),
                "200 OK",
                fs::read(fixture.join("metadata/1.delegated.json"))?,
            ),
            (
                "/targets/d13ecc865c37b23650615038c232eccfa5770c8bc345dbe98db595273fecda4a.artifact.bin".to_owned(),
                "200 OK",
                expected_target_body.to_vec(),
            ),
        ],
    )
}

fn assert_http_exchange_sequence_from_expected(
    exchanges: &[HttpExchangeRecord],
    expected: Vec<(String, &'static str, Vec<u8>)>,
) -> Result<(), Box<dyn Error + Send + Sync + 'static>> {
    assert_eq!(exchanges.len(), expected.len());
    for (exchange, (path, status, body)) in exchanges.iter().zip(expected) {
        let request_line_end = exchange
            .request_bytes
            .windows(2)
            .position(|window| window == b"\r\n")
            .ok_or("loopback HTTP request lacked a complete request line")?;
        let expected_request_line = format!("GET {path} HTTP/1.1");
        assert_eq!(
            &exchange.request_bytes[..request_line_end],
            expected_request_line.as_bytes()
        );
        assert!(exchange.request_bytes.ends_with(b"\r\n\r\n"));

        let mut expected_response = format!(
            "HTTP/1.1 {status}\r\nContent-Length: {}\r\nConnection: close\r\n\r\n",
            body.len()
        )
        .into_bytes();
        expected_response.extend_from_slice(&body);
        assert_eq!(exchange.response_bytes, expected_response);
    }

    Ok(())
}

#[tokio::test(flavor = "current_thread")]
async fn loopback_http_tuf_rejects_target_transport_equivocation_before_gate_native_or_fake_consumer(
) -> Result<(), Box<dyn Error + Send + Sync + 'static>> {
    let selection = accepted_selection();
    let fixture = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/data/publisher-client-binding");
    let altered_target_bytes = b"codiquary issue 20 fixturE\n";
    let mut observer = AttemptObserver::default();
    let mut http_exchanges = Vec::new();

    let attempt = run_http_makepkg_fixture_attempt(
        selection,
        &fixture,
        NativeFixture::Valid,
        accepted_policy(),
        &mut observer,
        &mut http_exchanges,
        Some(altered_target_bytes),
    )
    .await;

    let _error = attempt.expect_err("altered HTTP target bytes were accepted");
    assert_eq!(observer.held_bytes, None);
    assert_eq!(observer.held_sha256, None);
    assert_eq!(observer.gate_input_bytes, None);
    assert_eq!(observer.gate_calls, 0);
    assert_eq!(observer.gate_outcome, None);
    assert_eq!(observer.native_attempts, 0);
    assert_eq!(observer.consumer_calls, 0);
    assert_eq!(observer.consumer_sha256, None);
    assert_eq!(observer.evidence, None);
    assert_http_exchange_sequence(&http_exchanges, &fixture, altered_target_bytes)?;

    Ok(())
}

fn assert_retained_http_exchange_bytes(
    evidence: &Path,
    exchanges: &[HttpExchangeRecord],
) -> Result<(), Box<dyn Error + Send + Sync + 'static>> {
    for (sequence, exchange) in exchanges.iter().enumerate() {
        let label = format!("http-exchange-{sequence:020}");
        let request_path = evidence.join(format!("{label}.request"));
        let response_path = evidence.join(format!("{label}.response"));

        let retained_request = fs::read(&request_path).map_err(|error| {
            format!(
                "retained HTTP request bytes were unavailable at {}: {error}",
                request_path.display()
            )
        })?;
        let retained_response = fs::read(&response_path).map_err(|error| {
            format!(
                "retained HTTP response bytes were unavailable at {}: {error}",
                response_path.display()
            )
        })?;

        assert_eq!(
            retained_request.as_slice(),
            exchange.request_bytes.as_slice(),
            "retained request {sequence} differed from the observed server bytes"
        );
        assert_eq!(
            retained_response.as_slice(),
            exchange.response_bytes.as_slice(),
            "retained response {sequence} differed from the observed server bytes"
        );
    }

    Ok(())
}

#[tokio::test(flavor = "current_thread")]
async fn loopback_http_retains_observed_server_exchange_bytes_for_success_and_equivocation_rejection(
) -> Result<(), Box<dyn Error + Send + Sync + 'static>> {
    let fixture = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/data/publisher-client-binding");

    let mut successful_observer = AttemptObserver::default();
    let mut successful_exchanges = Vec::new();
    run_http_makepkg_fixture_attempt(
        accepted_selection(),
        &fixture,
        NativeFixture::Valid,
        accepted_policy(),
        &mut successful_observer,
        &mut successful_exchanges,
        None,
    )
    .await?;

    assert_ne!(
        successful_observer.http_evidence.as_deref(),
        successful_observer.evidence.as_deref(),
        "curl and native evidence directories must remain distinct"
    );
    let successful_http_evidence = successful_observer
        .http_evidence
        .as_deref()
        .expect("the successful HTTP attempt did not expose its evidence directory");
    let successful_retention =
        assert_retained_http_exchange_bytes(successful_http_evidence, &successful_exchanges);

    let altered_target_bytes = b"codiquary issue 20 fixturE\n";
    let mut equivocation_observer = AttemptObserver::default();
    let mut equivocation_exchanges = Vec::new();
    let equivocation_attempt = run_http_makepkg_fixture_attempt(
        accepted_selection(),
        &fixture,
        NativeFixture::Valid,
        accepted_policy(),
        &mut equivocation_observer,
        &mut equivocation_exchanges,
        Some(altered_target_bytes),
    )
    .await;
    equivocation_attempt.expect_err("altered HTTP target bytes were accepted");

    let equivocation_http_evidence = equivocation_observer
        .http_evidence
        .as_deref()
        .expect("the rejected HTTP attempt did not expose its evidence directory");
    let equivocation_retention =
        assert_retained_http_exchange_bytes(equivocation_http_evidence, &equivocation_exchanges);

    successful_retention?;
    equivocation_retention?;
    Ok(())
}

#[tokio::test(flavor = "current_thread")]
async fn file_backed_tuf_target_rejects_disabled_native_signature_check_before_fake_consumer(
) -> Result<(), Box<dyn Error + Send + Sync + 'static>> {
    let selection = accepted_selection();
    let fixture = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/data/publisher-client-binding");
    let mut observer = AttemptObserver::default();

    let attempt = run_makepkg_fixture_attempt(
        selection,
        &fixture,
        NativeFixture::SignatureCheckDisabled,
        accepted_policy(),
        &mut observer,
    )
    .await;

    assert_eq!(observer.native_attempts, 1);
    assert_eq!(observer.consumer_calls, 0);
    assert_eq!(observer.consumer_sha256, None);

    let error = attempt.expect_err("disabled native signature checking reached the fake consumer");
    let evidence = observer
        .evidence
        .as_ref()
        .expect("the native attempt did not expose retained evidence");
    assert_eq!(
        error.to_string(),
        format!(
            "makepkg succeeded without the expected native signature observation; evidence retained at {}",
            evidence.display()
        )
    );

    let status = fs::read_to_string(evidence.join("makepkg-verifysource.status"))?;
    let stdout = fs::read_to_string(evidence.join("makepkg-verifysource.stdout"))?;
    let stderr = fs::read_to_string(evidence.join("makepkg-verifysource.stderr"))?;
    let native_output = format!("{stdout}{stderr}");

    assert_eq!(status, "success=true\ncode=Some(0)\n");
    assert!(native_output.contains("Validating source files with sha256sums..."));
    assert!(native_output.contains("artifact.bin ... Passed"));
    assert!(!native_output.contains("Verifying source file signatures with gpg..."));
    assert!(!native_output.contains("artifact.bin.sig ... Passed"));

    Ok(())
}
