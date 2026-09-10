use serde::Deserialize;
use std::fs;
use std::os::unix::fs::PermissionsExt;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};
use std::time::{SystemTime, UNIX_EPOCH};

pub(crate) const REQUIRED_PACKAGE_SIGLEVEL: u32 = 1 << 0;
pub(crate) const REQUIRED_DATABASE_SIGLEVEL: u32 = 1 << 10;
pub(crate) const PRIMARY_CERTIFICATE_FINGERPRINT: &str = "3A013736081935591B079572B5BF6DA65146304C";
pub(crate) const SIGNING_SUBKEY_FINGERPRINT: &str = "5A6FC73F924A7C473C2B76AD5B9B92C920B04114";
pub(crate) const EXPECTED_NATIVE_SIGNATURE_FINGERPRINT: &str = PRIMARY_CERTIFICATE_FINGERPRINT;
const PROBE_SCHEMA: &str = "codiquary.pacman-native-probe.v1";

const SIGNER_SHA256: &str = "94935870e160c38323db1324cc0f02f9305059c6dd62a92380843862c0e7f4fc";
const PACKAGE_SHA256: &str = "dc68e48a55268cd10831487acff7e36936401d044c14cc2dec1ed94b3467b313";
const MTIME_CHANGED_PACKAGE_SHA256: &str =
    "4084b6af1224854f4565f17847cdfec35c0e616d848e1fbfee61b14de35fac36";
const PACKAGE_SIGNATURE_SHA256: &str =
    "d0ff7b8135dc2adff16fef9ccd8cd7a472f328331314b24d89dc937d36e6a092";
const DATABASE_SHA256: &str = "65a8552a8faf3d612369e955b3b08a99aa9e283512976bde9049f75babbc7de6";
const MTIME_CHANGED_DATABASE_SHA256: &str =
    "0621e9625401897ae381984651780a3d531a3e1c038ec9f4539ca332a7adaf50";
const DATABASE_SIGNATURE_SHA256: &str =
    "089b4995b67497460da1a146a0bf95f6239b6e6d0ce83086bf15dd4136d76e0d";

#[derive(Clone, Copy, Debug)]
pub(crate) enum PackageFixture {
    Original,
    MtimeChangedWithOriginalSignature,
}

impl PackageFixture {
    pub(crate) fn filename(self) -> &'static str {
        match self {
            Self::Original => "codiquary17-native-1.0-1-any.pkg.tar.zst",
            Self::MtimeChangedWithOriginalSignature => {
                "codiquary17-native-1.0-1-any.pkg.mtime-changed.tar.zst"
            }
        }
    }

    pub(crate) fn sha256(self) -> &'static str {
        match self {
            Self::Original => PACKAGE_SHA256,
            Self::MtimeChangedWithOriginalSignature => MTIME_CHANGED_PACKAGE_SHA256,
        }
    }
}

#[derive(Clone, Copy, Debug)]
pub(crate) enum DatabaseFixture {
    Original,
    MtimeChangedWithOriginalSignature,
}

impl DatabaseFixture {
    fn filename(self) -> &'static str {
        match self {
            Self::Original => "codiquary17.db",
            Self::MtimeChangedWithOriginalSignature => "codiquary17.mtime-changed.db",
        }
    }

    fn sha256(self) -> &'static str {
        match self {
            Self::Original => DATABASE_SHA256,
            Self::MtimeChangedWithOriginalSignature => MTIME_CHANGED_DATABASE_SHA256,
        }
    }
}

#[derive(Clone, Copy, Debug)]
pub(crate) enum PackageSignatureCheck {
    Required,
    Disabled,
}

impl PackageSignatureCheck {
    fn native_argument(self) -> &'static str {
        match self {
            Self::Required => "required",
            Self::Disabled => "disabled",
        }
    }
}

#[derive(Clone, Copy, Debug)]
pub(crate) enum DatabaseSignatureCheck {
    Required,
    Disabled,
}

impl DatabaseSignatureCheck {
    fn native_argument(self) -> &'static str {
        match self {
            Self::Required => "required",
            Self::Disabled => "disabled",
        }
    }
}

#[derive(Debug)]
pub(crate) struct HeldPackageBytes<'a> {
    pub(crate) expected_sha256: &'static str,
    pub(crate) bytes: &'a [u8],
}

#[derive(Debug)]
pub(crate) struct NativeProbeRequest<'a> {
    pub(crate) fixture_dir: PathBuf,
    pub(crate) scratch_dir: PathBuf,
    pub(crate) held_package: HeldPackageBytes<'a>,
    pub(crate) database_fixture: DatabaseFixture,
    pub(crate) package_signature_check: PackageSignatureCheck,
    pub(crate) database_signature_check: DatabaseSignatureCheck,
    pub(crate) primary_certificate_fingerprint: &'static str,
    pub(crate) signing_subkey_fingerprint: &'static str,
    pub(crate) expected_native_signature_fingerprint: &'static str,
}

#[derive(Debug, Deserialize, PartialEq, Eq)]
enum NativeSignatureStatus {
    Valid,
}

#[derive(Debug, Deserialize, PartialEq, Eq)]
enum NativeSignatureValidity {
    Full,
}

#[derive(Debug, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub(crate) struct SignatureEvidence {
    check_return: i32,
    count: usize,
    fingerprint: String,
    status: NativeSignatureStatus,
    validity: NativeSignatureValidity,
}

impl SignatureEvidence {
    pub(crate) fn is_expected(&self) -> bool {
        self.check_return == 0
            && self.count == 1
            && self.fingerprint == EXPECTED_NATIVE_SIGNATURE_FINGERPRINT
            && self.status == NativeSignatureStatus::Valid
            && self.validity == NativeSignatureValidity::Full
    }
}

#[derive(Debug, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub(crate) struct PackageEvidence {
    pub(crate) configured_siglevel: u32,
    pub(crate) full_archive_requested: bool,
    pub(crate) load_return: i32,
    pub(crate) errno_ok: bool,
    pub(crate) package_non_null: bool,
    pub(crate) signature: SignatureEvidence,
}

#[derive(Debug, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub(crate) struct DatabasePackageEvidence {
    pub(crate) name: String,
    pub(crate) version: String,
    pub(crate) filename: String,
}

#[derive(Debug, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub(crate) struct DatabaseEvidence {
    pub(crate) configured_siglevel: u32,
    pub(crate) database_non_null: bool,
    pub(crate) valid_return: i32,
    pub(crate) signature: SignatureEvidence,
    pub(crate) package: DatabasePackageEvidence,
}

#[derive(Debug, PartialEq, Eq)]
pub(crate) struct NativePositiveEvidence {
    pub(crate) question_callback_calls: usize,
    pub(crate) package: PackageEvidence,
    pub(crate) database: DatabaseEvidence,
}

impl NativePositiveEvidence {
    fn native_ready(&self) -> bool {
        self.question_callback_calls == 0
            && self.package.configured_siglevel == REQUIRED_PACKAGE_SIGLEVEL
            && self.package.full_archive_requested
            && self.package.load_return == 0
            && self.package.errno_ok
            && self.package.package_non_null
            && self.package.signature.is_expected()
            && self.database.configured_siglevel == REQUIRED_DATABASE_SIGLEVEL
            && self.database.database_non_null
            && self.database.valid_return == 0
            && self.database.signature.is_expected()
            && self.database.package.name == "codiquary17-native"
            && self.database.package.version == "1.0-1"
            && self.database.package.filename == "codiquary17-native-1.0-1-any.pkg.tar.zst"
    }
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct ProbeEnvelope {
    schema: String,
    question_callback_calls: usize,
    package: PackageEvidence,
    database: DatabaseEvidence,
}

#[derive(Debug)]
pub(crate) struct NativeRejectionReceipt {
    pub(crate) status_code: Option<i32>,
    pub(crate) stdout: Vec<u8>,
    pub(crate) stderr: Vec<u8>,
}

#[derive(Debug)]
enum NativeRunOutcome {
    Accepted(NativePositiveEvidence),
    Rejected(NativeRejectionReceipt),
}

#[derive(Debug)]
pub(crate) enum NativeProbeError {
    Failed {
        attempt_dir: PathBuf,
        message: String,
    },
    Rejected {
        attempt_dir: PathBuf,
        receipt: NativeRejectionReceipt,
    },
    UnsafeNativeEvidence {
        evidence: Box<NativePositiveEvidence>,
    },
}

struct AttemptPaths {
    attempt: PathBuf,
    root: PathBuf,
    db: PathBuf,
    gpg: PathBuf,
    artifacts: PathBuf,
    home: PathBuf,
    temporary: PathBuf,
    output: PathBuf,
    binary: PathBuf,
}

fn create_attempt(base: &Path) -> Result<PathBuf, String> {
    fs::create_dir_all(base)
        .map_err(|error| format!("could not create scratch base {}: {error}", base.display()))?;

    let timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_or(0, |duration| duration.as_nanos());

    for sequence in 0..1000_u32 {
        let candidate = base.join(format!(
            "attempt-{timestamp}-{}-{sequence}",
            std::process::id()
        ));
        match fs::create_dir(&candidate) {
            Ok(()) => {
                return fs::canonicalize(&candidate).map_err(|error| {
                    format!(
                        "could not canonicalize attempt directory {}: {error}",
                        candidate.display()
                    )
                });
            }
            Err(error) if error.kind() == std::io::ErrorKind::AlreadyExists => {}
            Err(error) => {
                return Err(format!(
                    "could not create attempt directory {}: {error}",
                    candidate.display()
                ));
            }
        }
    }

    Err(format!(
        "could not allocate a unique retained attempt under {}",
        base.display()
    ))
}

fn prepare_paths(attempt: PathBuf) -> Result<AttemptPaths, String> {
    let paths = AttemptPaths {
        root: attempt.join("root"),
        db: attempt.join("db"),
        gpg: attempt.join("gpg"),
        artifacts: attempt.join("artifacts"),
        home: attempt.join("home"),
        temporary: attempt.join("tmp"),
        output: attempt.join("output"),
        binary: attempt.join("bin"),
        attempt,
    };

    for directory in [
        &paths.root,
        &paths.db.join("local"),
        &paths.db.join("sync"),
        &paths.gpg,
        &paths.artifacts,
        &paths.home,
        &paths.temporary,
        &paths.output,
        &paths.binary,
    ] {
        fs::create_dir_all(directory).map_err(|error| {
            format!(
                "could not create directory {}: {error}",
                directory.display()
            )
        })?;
    }

    fs::set_permissions(&paths.gpg, fs::Permissions::from_mode(0o700)).map_err(|error| {
        format!(
            "could not set mode 0700 on GPG home {}: {error}",
            paths.gpg.display()
        )
    })?;

    Ok(paths)
}

fn configure_command(command: &mut Command, paths: &AttemptPaths) {
    command
        .env_clear()
        .env("PATH", "/usr/bin:/bin")
        .env("LANG", "C.UTF-8")
        .env("LC_ALL", "C.UTF-8")
        .env("HOME", &paths.home)
        .env("TMPDIR", &paths.temporary)
        .env("GNUPGHOME", &paths.gpg)
        .current_dir(&paths.attempt);
}

fn run_logged(mut command: Command, paths: &AttemptPaths, label: &str) -> Result<Output, String> {
    configure_command(&mut command, paths);

    fs::write(
        paths.output.join(format!("{label}.command.txt")),
        format!("{command:?}\n"),
    )
    .map_err(|error| format!("could not record {label} command: {error}"))?;

    let output = match command.output() {
        Ok(output) => output,
        Err(error) => {
            let diagnostic = format!("could not execute {label}: {error}\n");
            let _ = fs::write(
                paths.output.join(format!("{label}.stderr")),
                diagnostic.as_bytes(),
            );
            return Err(diagnostic.trim_end().to_owned());
        }
    };

    fs::write(paths.output.join(format!("{label}.stdout")), &output.stdout)
        .map_err(|error| format!("could not retain {label} stdout: {error}"))?;
    fs::write(paths.output.join(format!("{label}.stderr")), &output.stderr)
        .map_err(|error| format!("could not retain {label} stderr: {error}"))?;

    let status_receipt = match output.status.code() {
        Some(code) => format!("exit_code={code}\n"),
        None => "exit_code=none\n".to_owned(),
    };
    fs::write(paths.output.join(format!("{label}.status")), status_receipt)
        .map_err(|error| format!("could not retain {label} exit status: {error}"))?;

    Ok(output)
}

fn require_success(command: Command, paths: &AttemptPaths, label: &str) -> Result<Output, String> {
    let output = run_logged(command, paths, label)?;
    if output.status.code() != Some(0) {
        return Err(format!(
            "{label} exited with {:?}: {}",
            output.status.code(),
            String::from_utf8_lossy(&output.stderr).trim_end()
        ));
    }
    Ok(output)
}

fn verify_sha256(
    path: &Path,
    expected: &str,
    paths: &AttemptPaths,
    label: &str,
) -> Result<(), String> {
    if !path.is_file() {
        return Err(format!(
            "required public fixture is not a file: {}",
            path.display()
        ));
    }

    let mut command = Command::new("sha256sum");
    command.arg(path);
    let output = require_success(command, paths, label)?;
    let text = std::str::from_utf8(&output.stdout)
        .map_err(|error| format!("{label} emitted non-UTF-8 output: {error}"))?;
    let line = text
        .strip_suffix('\n')
        .ok_or_else(|| format!("{label} output was not newline terminated"))?;
    if line.contains(['\n', '\r']) {
        return Err(format!("{label} emitted more than one line"));
    }

    let mut fields = line.split_whitespace();
    let actual = fields
        .next()
        .ok_or_else(|| format!("{label} omitted the SHA-256 digest"))?;
    let reported_path = fields
        .next()
        .ok_or_else(|| format!("{label} omitted the hashed path"))?;
    if fields.next().is_some() {
        return Err(format!("{label} emitted unexpected fields"));
    }

    let expected_path = path.to_string_lossy();
    if reported_path != expected_path.as_ref() as &str {
        return Err(format!(
            "{label} reported path {reported_path}, expected {}",
            path.display()
        ));
    }
    if actual != expected {
        return Err(format!(
            "SHA-256 mismatch for {}: got {actual}, expected {expected}",
            path.display()
        ));
    }

    Ok(())
}

fn verify_bytes_sha256(bytes: &[u8], expected: &str, label: &str) -> Result<(), String> {
    const HEX: &[u8; 16] = b"0123456789abcdef";

    let digest = aws_lc_rs::digest::digest(&aws_lc_rs::digest::SHA256, bytes);
    let mut actual = String::with_capacity(digest.as_ref().len() * 2);
    for byte in digest.as_ref() {
        let byte = *byte;
        actual.push(HEX[(byte >> 4) as usize] as char);
        actual.push(HEX[(byte & 0x0f) as usize] as char);
    }

    if actual != expected {
        return Err(format!(
            "SHA-256 mismatch for {label}: got {actual}, expected {expected}"
        ));
    }

    Ok(())
}

fn read_companion_once(source: &Path) -> Result<Vec<u8>, String> {
    if !source.is_file() {
        return Err(format!(
            "required public fixture is not a file: {}",
            source.display()
        ));
    }

    fs::read(source).map_err(|error| format!("could not read {}: {error}", source.display()))
}

fn stage_checked_bytes(
    bytes: &[u8],
    destination: &Path,
    expected_sha256: &str,
    paths: &AttemptPaths,
    label: &str,
) -> Result<(), String> {
    verify_bytes_sha256(bytes, expected_sha256, &format!("{label} held bytes"))?;
    fs::write(destination, bytes)
        .map_err(|error| format!("could not stage {}: {error}", destination.display()))?;

    let staged_bytes = fs::read(destination)
        .map_err(|error| format!("could not retain {}: {error}", destination.display()))?;
    if staged_bytes.as_slice() != bytes {
        return Err(format!(
            "staged bytes differed from the held buffer for {}",
            destination.display()
        ));
    }

    verify_sha256(
        destination,
        expected_sha256,
        paths,
        &format!("{label}-staged-sha256"),
    )
}

fn run_native_positive(
    request: &NativeProbeRequest<'_>,
    paths: &AttemptPaths,
) -> Result<NativeRunOutcome, String> {
    if request.primary_certificate_fingerprint != PRIMARY_CERTIFICATE_FINGERPRINT {
        return Err("caller supplied the wrong primary certificate fingerprint".to_owned());
    }
    if request.signing_subkey_fingerprint != SIGNING_SUBKEY_FINGERPRINT {
        return Err("caller supplied the wrong signing-subkey fingerprint".to_owned());
    }
    if request.expected_native_signature_fingerprint != EXPECTED_NATIVE_SIGNATURE_FINGERPRINT {
        return Err("caller supplied the wrong expected native signature fingerprint".to_owned());
    }

    let signer_source = request.fixture_dir.join("signer.pub");
    let package_signature_source = request
        .fixture_dir
        .join("codiquary17-native-1.0-1-any.pkg.tar.zst.sig");
    let database_source = request
        .fixture_dir
        .join(request.database_fixture.filename());
    let database_signature_source = request.fixture_dir.join("codiquary17.db.sig");

    let signer = paths.artifacts.join("signer.pub");
    let package = paths
        .artifacts
        .join("codiquary17-native-1.0-1-any.pkg.tar.zst");
    let package_signature = paths
        .artifacts
        .join("codiquary17-native-1.0-1-any.pkg.tar.zst.sig");
    let database = paths.db.join("sync/codiquary17.db");
    let database_signature = paths.db.join("sync/codiquary17.db.sig");

    stage_checked_bytes(
        request.held_package.bytes,
        &package,
        request.held_package.expected_sha256,
        paths,
        "package",
    )?;

    let signer_bytes = read_companion_once(&signer_source)?;
    stage_checked_bytes(&signer_bytes, &signer, SIGNER_SHA256, paths, "signer")?;

    let package_signature_bytes = read_companion_once(&package_signature_source)?;
    stage_checked_bytes(
        &package_signature_bytes,
        &package_signature,
        PACKAGE_SIGNATURE_SHA256,
        paths,
        "package-signature",
    )?;

    let database_bytes = read_companion_once(&database_source)?;
    stage_checked_bytes(
        &database_bytes,
        &database,
        request.database_fixture.sha256(),
        paths,
        "database",
    )?;

    let database_signature_bytes = read_companion_once(&database_signature_source)?;
    stage_checked_bytes(
        &database_signature_bytes,
        &database_signature,
        DATABASE_SIGNATURE_SHA256,
        paths,
        "database-signature",
    )?;

    let mut import = Command::new("gpg");
    import
        .arg("--batch")
        .arg("--no-options")
        .arg("--homedir")
        .arg(&paths.gpg)
        .arg("--import")
        .arg(&signer);
    require_success(import, paths, "gpg-import-public-certificate")?;

    let ownertrust = paths.attempt.join("ownertrust.txt");
    fs::write(
        &ownertrust,
        format!("{}:6:\n", request.primary_certificate_fingerprint),
    )
    .map_err(|error| format!("could not write {}: {error}", ownertrust.display()))?;

    let mut import_ownertrust = Command::new("gpg");
    import_ownertrust
        .arg("--batch")
        .arg("--no-options")
        .arg("--homedir")
        .arg(&paths.gpg)
        .arg("--import-ownertrust")
        .arg(&ownertrust);
    require_success(import_ownertrust, paths, "gpg-import-primary-ownertrust")?;

    let helper_source =
        Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/support/pacman_native_probe.c");
    if !helper_source.is_file() {
        return Err(format!(
            "native helper is not a normal source file: {}",
            helper_source.display()
        ));
    }

    let mut pkg_config = Command::new("pkg-config");
    pkg_config.args(["--cflags", "--libs", "libalpm"]);
    let pkg_config_output = require_success(pkg_config, paths, "pkg-config-libalpm-cflags-libs")?;
    let pkg_config_flags = std::str::from_utf8(&pkg_config_output.stdout)
        .map_err(|error| format!("pkg-config emitted non-UTF-8 flags: {error}"))?;
    let pkg_config_flags: Vec<&str> = pkg_config_flags.split_ascii_whitespace().collect();
    if pkg_config_flags.is_empty() {
        return Err("pkg-config emitted no libalpm compilation or link flags".to_owned());
    }

    let probe = paths.binary.join("pacman_native_probe");
    let mut compiler = Command::new("cc");
    compiler
        .args(["-std=c11", "-Wall", "-Wextra", "-Werror", "-o"])
        .arg(&probe)
        .arg(&helper_source)
        .args(&pkg_config_flags);
    require_success(compiler, paths, "compile-pacman-native-probe")?;

    let root = fs::canonicalize(&paths.root)
        .map_err(|error| format!("could not canonicalize root: {error}"))?;
    let db = fs::canonicalize(&paths.db)
        .map_err(|error| format!("could not canonicalize database path: {error}"))?;
    let gpg = fs::canonicalize(&paths.gpg)
        .map_err(|error| format!("could not canonicalize GPG path: {error}"))?;
    let package = fs::canonicalize(&package)
        .map_err(|error| format!("could not canonicalize package path: {error}"))?;

    let mut native_probe = Command::new(&probe);
    native_probe
        .arg(&root)
        .arg(&db)
        .arg(&gpg)
        .arg(&package)
        .arg(request.package_signature_check.native_argument())
        .arg(request.database_signature_check.native_argument())
        .arg(request.expected_native_signature_fingerprint);
    let output = run_logged(native_probe, paths, "run-pacman-native-probe")?;
    if output.status.code() != Some(0) {
        return Ok(NativeRunOutcome::Rejected(NativeRejectionReceipt {
            status_code: output.status.code(),
            stdout: output.stdout,
            stderr: output.stderr,
        }));
    }

    let json = output
        .stdout
        .strip_suffix(b"\n")
        .ok_or_else(|| "native probe stdout was not newline terminated".to_owned())?;
    if json.is_empty()
        || json.first() != Some(&b'{')
        || json.last() != Some(&b'}')
        || json.contains(&b'\n')
        || json.contains(&b'\r')
    {
        return Err(
            "native probe stdout was not exactly one compact newline-terminated JSON object"
                .to_owned(),
        );
    }

    let envelope: ProbeEnvelope = serde_json::from_slice(json)
        .map_err(|error| format!("native probe emitted invalid or divergent evidence: {error}"))?;
    if envelope.schema != PROBE_SCHEMA {
        return Err(format!(
            "native probe schema was {}, expected {PROBE_SCHEMA}",
            envelope.schema
        ));
    }

    Ok(NativeRunOutcome::Accepted(NativePositiveEvidence {
        question_callback_calls: envelope.question_callback_calls,
        package: envelope.package,
        database: envelope.database,
    }))
}

pub(crate) fn verify_native_held_package(
    request: &NativeProbeRequest<'_>,
) -> Result<NativePositiveEvidence, NativeProbeError> {
    let attempt =
        create_attempt(&request.scratch_dir).map_err(|message| NativeProbeError::Failed {
            attempt_dir: request.scratch_dir.clone(),
            message,
        })?;
    let retained_attempt = attempt.clone();
    let paths = prepare_paths(attempt).map_err(|message| NativeProbeError::Failed {
        attempt_dir: retained_attempt.clone(),
        message,
    })?;

    match run_native_positive(request, &paths) {
        Ok(NativeRunOutcome::Accepted(evidence)) => Ok(evidence),
        Ok(NativeRunOutcome::Rejected(receipt)) => Err(NativeProbeError::Rejected {
            attempt_dir: retained_attempt,
            receipt,
        }),
        Err(message) => Err(NativeProbeError::Failed {
            attempt_dir: retained_attempt,
            message,
        }),
    }
}

fn expected_signature() -> SignatureEvidence {
    SignatureEvidence {
        check_return: 0,
        count: 1,
        fingerprint: EXPECTED_NATIVE_SIGNATURE_FINGERPRINT.to_owned(),
        status: NativeSignatureStatus::Valid,
        validity: NativeSignatureValidity::Full,
    }
}

pub(crate) fn expected_positive_evidence() -> NativePositiveEvidence {
    NativePositiveEvidence {
        question_callback_calls: 0,
        package: PackageEvidence {
            configured_siglevel: REQUIRED_PACKAGE_SIGLEVEL,
            full_archive_requested: true,
            load_return: 0,
            errno_ok: true,
            package_non_null: true,
            signature: expected_signature(),
        },
        database: DatabaseEvidence {
            configured_siglevel: REQUIRED_DATABASE_SIGLEVEL,
            database_non_null: true,
            valid_return: 0,
            signature: expected_signature(),
            package: DatabasePackageEvidence {
                name: "codiquary17-native".to_owned(),
                version: "1.0-1".to_owned(),
                filename: "codiquary17-native-1.0-1-any.pkg.tar.zst".to_owned(),
            },
        },
    }
}

pub(crate) fn fake_consumer(consumer_calls: &mut usize) {
    *consumer_calls += 1;
}

pub(crate) fn verify_native_before_consumer<F>(
    request: &NativeProbeRequest<'_>,
    consumer: F,
) -> Result<NativePositiveEvidence, NativeProbeError>
where
    F: FnOnce(),
{
    let evidence = verify_native_held_package(request)?;
    if evidence.native_ready() {
        consumer();
        Ok(evidence)
    } else {
        Err(NativeProbeError::UnsafeNativeEvidence {
            evidence: Box::new(evidence),
        })
    }
}
