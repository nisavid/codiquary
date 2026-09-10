use std::env;
use std::ffi::OsStr;
use std::fs::{self, OpenOptions};
use std::io::{self, Write};
use std::path::{Path, PathBuf};
use std::process::Command;

use aws_lc_rs::digest;
use sequoia_openpgp as openpgp;
use serde::Serialize;

use openpgp::cert::prelude::*;
use openpgp::policy::StandardPolicy;
use openpgp::serialize::stream::{Message, Signer};
use openpgp::serialize::SerializeInto;
use openpgp::types::HashAlgorithm;

const PACKAGE_MEMBER_PATH: &str = ".PKGINFO";
const PACKAGE_MEMBER: &[u8] = b"pkgname = codiquary17-native\npkgver = 1.0-1\n";
const DATABASE_MEMBER_PATH: &str = "codiquary17-native-1.0-1/desc";
const DATABASE_MEMBER: &[u8] = b"%NAME%\ncodiquary17-native\n\n%VERSION%\n1.0-1\n\n%FILENAME%\ncodiquary17-native-1.0-1-any.pkg.tar.zst\n\n";

const PACKAGE_NAME: &str = "codiquary17-native-1.0-1-any.pkg.tar.zst";
const PACKAGE_CHANGED_NAME: &str = "codiquary17-native-1.0-1-any.pkg.mtime-changed.tar.zst";
const PACKAGE_SIGNATURE_NAME: &str = "codiquary17-native-1.0-1-any.pkg.tar.zst.sig";
const DATABASE_NAME: &str = "codiquary17.db";
const DATABASE_CHANGED_NAME: &str = "codiquary17.mtime-changed.db";
const DATABASE_SIGNATURE_NAME: &str = "codiquary17.db.sig";
const PUBLIC_CERTIFICATE_NAME: &str = "signer.pub";
const EXPECTED_NAME: &str = "expected.json";

const ORIGINAL_MTIME: i64 = 946_684_800;
const CHANGED_MTIME: i64 = 946_684_801;
const MAX_NATIVE_SIGNATURE_LENGTH: usize = 16_384;

#[derive(Serialize)]
struct Expected {
    schema: &'static str,
    classification: &'static str,
    purpose: &'static str,
    determinism: Determinism,
    tools: Vec<ToolFact>,
    signer: SignerIdentity,
    native_expectations: NativeExpectations,
    public_certificate: FileIdentity,
    archives: Vec<ArchiveIdentity>,
    evidence_files: Vec<String>,
}

#[derive(Serialize)]
struct Determinism {
    archive_bytes: &'static str,
    signer_and_signatures: &'static str,
}

#[derive(Serialize)]
struct ToolFact {
    executable: &'static str,
    version_output: String,
}

#[derive(Serialize)]
struct SignerIdentity {
    primary_certificate_fingerprint: String,
    signing_subkey_fingerprint: String,
    expected_native_signature_fingerprint: String,
    algorithm: &'static str,
    secret_key_serialized: bool,
}

#[derive(Serialize)]
struct NativeExpectations {
    signature_count: usize,
    status: &'static str,
    validity: &'static str,
    trust_setup: &'static str,
}

#[derive(Clone, Serialize)]
struct FileIdentity {
    filename: String,
    byte_length: usize,
    sha256: String,
    sha512: String,
}

#[derive(Serialize)]
struct ArchiveIdentity {
    role: &'static str,
    variant: &'static str,
    filename: &'static str,
    tar_member_mtime_epoch: i64,
    member: MemberIdentity,
    archive: FileIdentity,
    signature: Option<FileIdentity>,
    stale_signature_filename: Option<&'static str>,
    expected_signature_relationship: &'static str,
}

#[derive(Serialize)]
struct MemberIdentity {
    path: &'static str,
    content_utf8: &'static str,
    byte_length: usize,
    sha256: String,
    sha512: String,
}

fn main() -> openpgp::Result<()> {
    let output_dir = parse_output_directory()?;
    fs::create_dir(&output_dir)?;

    let scratch = output_dir.join("scratch");
    let package_source = scratch.join("package-source");
    let database_source = scratch.join("database-source");
    let database_record_dir = database_source.join("codiquary17-native-1.0-1");
    fs::create_dir(&scratch)?;
    fs::create_dir(&package_source)?;
    fs::create_dir(&database_source)?;
    fs::create_dir(&database_record_dir)?;

    write_new_path(&package_source.join(PACKAGE_MEMBER_PATH), PACKAGE_MEMBER)?;
    write_new_path(&database_source.join(DATABASE_MEMBER_PATH), DATABASE_MEMBER)?;

    let package_tar = scratch.join("package-original.tar");
    let package_changed_tar = scratch.join("package-mtime-changed.tar");
    let database_tar = scratch.join("database-original.tar");
    let database_changed_tar = scratch.join("database-mtime-changed.tar");

    create_tar(
        &package_tar,
        &package_source,
        PACKAGE_MEMBER_PATH,
        ORIGINAL_MTIME,
    )?;
    create_tar(
        &package_changed_tar,
        &package_source,
        PACKAGE_MEMBER_PATH,
        CHANGED_MTIME,
    )?;
    create_tar(
        &database_tar,
        &database_source,
        DATABASE_MEMBER_PATH,
        ORIGINAL_MTIME,
    )?;
    create_tar(
        &database_changed_tar,
        &database_source,
        DATABASE_MEMBER_PATH,
        CHANGED_MTIME,
    )?;

    let package_path = output_dir.join(PACKAGE_NAME);
    let package_changed_path = output_dir.join(PACKAGE_CHANGED_NAME);
    let database_path = output_dir.join(DATABASE_NAME);
    let database_changed_path = output_dir.join(DATABASE_CHANGED_NAME);

    compress_zstd(&package_tar, &package_path)?;
    compress_zstd(&package_changed_tar, &package_changed_path)?;
    compress_zstd(&database_tar, &database_path)?;
    compress_zstd(&database_changed_tar, &database_changed_path)?;

    let package_bytes = fs::read(&package_path)?;
    let package_changed_bytes = fs::read(&package_changed_path)?;
    let database_bytes = fs::read(&database_path)?;
    let database_changed_bytes = fs::read(&database_changed_path)?;

    if package_bytes == package_changed_bytes || database_bytes == database_changed_bytes {
        return Err(invalid_data(
            "an mtime-only archive variant unexpectedly matched its original",
        )
        .into());
    }

    let (secret_cert, revocation) = CertBuilder::new()
        .set_cipher_suite(CipherSuite::RSA2k)
        .add_userid("Codiquary synthetic nonproduction pacman fixture <fixture@synthetic.invalid>")
        .add_signing_subkey()
        .generate()?;
    drop(revocation);

    let policy = StandardPolicy::new();
    let primary_fingerprint = secret_cert.fingerprint().to_hex();
    let signing_subkey_fingerprint = signing_fingerprint(&secret_cert, &policy)?;

    let package_signature = detached_signature(
        &secret_cert,
        &policy,
        &signing_subkey_fingerprint,
        &package_bytes,
    )?;
    let database_signature = detached_signature(
        &secret_cert,
        &policy,
        &signing_subkey_fingerprint,
        &database_bytes,
    )?;

    if package_signature.len() >= MAX_NATIVE_SIGNATURE_LENGTH
        || database_signature.len() >= MAX_NATIVE_SIGNATURE_LENGTH
    {
        return Err(invalid_data("a detached signature is not below 16384 bytes").into());
    }

    let public_cert = secret_cert.strip_secret_key_material();
    if public_cert.is_tsk() {
        return Err(invalid_data("secret key material remained after stripping").into());
    }
    let public_certificate = public_cert.armored().to_vec()?;
    drop(public_cert);

    if !public_certificate.starts_with(b"-----BEGIN PGP PUBLIC KEY BLOCK-----") {
        return Err(invalid_data("unexpected public-certificate serialization").into());
    }

    write_new(&output_dir, PUBLIC_CERTIFICATE_NAME, &public_certificate)?;
    write_new(&output_dir, PACKAGE_SIGNATURE_NAME, &package_signature)?;
    write_new(&output_dir, DATABASE_SIGNATURE_NAME, &database_signature)?;

    let package_signature_identity = file_identity(PACKAGE_SIGNATURE_NAME, &package_signature);
    let database_signature_identity = file_identity(DATABASE_SIGNATURE_NAME, &database_signature);

    let expected = Expected {
        schema: "codiquary.synthetic-pacman-native-fixtures.v1",
        classification: "public synthetic nonproduction fixtures",
        purpose: "isolated native libalpm package and sync-database signature checks",
        determinism: Determinism {
            archive_bytes: "repeatable for identical member bytes, explicit metadata, and the recorded tar and zstd implementations",
            signer_and_signatures: "not deterministic; an ephemeral signer is generated for every fresh fixture directory",
        },
        tools: vec![
            ToolFact {
                executable: "/usr/bin/tar",
                version_output: tool_version("/usr/bin/tar")?,
            },
            ToolFact {
                executable: "/usr/bin/zstd",
                version_output: tool_version("/usr/bin/zstd")?,
            },
        ],
        signer: SignerIdentity {
            primary_certificate_fingerprint: primary_fingerprint.clone(),
            signing_subkey_fingerprint: signing_subkey_fingerprint.clone(),
            expected_native_signature_fingerprint: primary_fingerprint,
            algorithm: "OpenPGP RSA-2048 detached binary signature with SHA-256; interoperability fixture only",
            secret_key_serialized: false,
        },
        native_expectations: NativeExpectations {
            signature_count: 1,
            status: "ALPM_SIGSTATUS_VALID",
            validity: "ALPM_SIGVALIDITY_FULL",
            trust_setup: "import signer.pub into the isolated GPG home and assign the primary certificate fingerprint ultimate ownertrust",
        },
        public_certificate: file_identity(PUBLIC_CERTIFICATE_NAME, &public_certificate),
        archives: vec![
            archive_identity(
                "package",
                "original",
                PACKAGE_NAME,
                ORIGINAL_MTIME,
                PACKAGE_MEMBER_PATH,
                PACKAGE_MEMBER,
                &package_bytes,
                Some(package_signature_identity),
                None,
                "the attached detached signature covers these exact archive bytes",
            ),
            archive_identity(
                "package",
                "mtime-only-changed",
                PACKAGE_CHANGED_NAME,
                CHANGED_MTIME,
                PACKAGE_MEMBER_PATH,
                PACKAGE_MEMBER,
                &package_changed_bytes,
                None,
                Some(PACKAGE_SIGNATURE_NAME),
                "retain the original signature to exercise stale-signature rejection",
            ),
            archive_identity(
                "database",
                "original",
                DATABASE_NAME,
                ORIGINAL_MTIME,
                DATABASE_MEMBER_PATH,
                DATABASE_MEMBER,
                &database_bytes,
                Some(database_signature_identity),
                None,
                "the attached detached signature covers these exact archive bytes",
            ),
            archive_identity(
                "database",
                "mtime-only-changed",
                DATABASE_CHANGED_NAME,
                CHANGED_MTIME,
                DATABASE_MEMBER_PATH,
                DATABASE_MEMBER,
                &database_changed_bytes,
                None,
                Some(DATABASE_SIGNATURE_NAME),
                "retain the original signature to exercise stale-signature rejection",
            ),
        ],
        evidence_files: vec![
            "scratch/package-source/.PKGINFO".into(),
            "scratch/database-source/codiquary17-native-1.0-1/desc".into(),
            "scratch/package-original.tar".into(),
            "scratch/package-mtime-changed.tar".into(),
            "scratch/database-original.tar".into(),
            "scratch/database-mtime-changed.tar".into(),
        ],
    };

    let mut expected_bytes = serde_json::to_vec_pretty(&expected)?;
    expected_bytes.push(b'\n');
    write_new(&output_dir, EXPECTED_NAME, &expected_bytes)?;

    Ok(())
}

fn parse_output_directory() -> io::Result<PathBuf> {
    let mut arguments = env::args_os();
    let _program = arguments.next();
    let argument = arguments.next().ok_or_else(|| {
        io::Error::new(
            io::ErrorKind::InvalidInput,
            "expected exactly one fresh output-directory argument",
        )
    })?;
    if arguments.next().is_some() {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            "unexpected extra argument",
        ));
    }

    let output = PathBuf::from(argument);
    if !output.is_absolute() || output.parent() != Some(Path::new("/tmp")) {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            "output directory must be an absolute fresh direct child of /tmp",
        ));
    }
    if output.file_name().is_none() || output.exists() {
        return Err(io::Error::new(
            io::ErrorKind::AlreadyExists,
            "output directory must not already exist",
        ));
    }
    Ok(output)
}

fn create_tar(
    destination: &Path,
    source_directory: &Path,
    member: &str,
    mtime: i64,
) -> io::Result<()> {
    ensure_absent(destination)?;
    let mtime_argument = format!("@{mtime}");
    run_checked(
        "/usr/bin/tar",
        [
            OsStr::new("--create"),
            OsStr::new("--format=ustar"),
            OsStr::new("--file"),
            destination.as_os_str(),
            OsStr::new("--directory"),
            source_directory.as_os_str(),
            OsStr::new("--sort=name"),
            OsStr::new("--mtime"),
            OsStr::new(&mtime_argument),
            OsStr::new("--owner=0"),
            OsStr::new("--group=0"),
            OsStr::new("--numeric-owner"),
            OsStr::new("--mode=0644"),
            OsStr::new("--no-recursion"),
            OsStr::new("--"),
            OsStr::new(member),
        ],
    )
}

fn compress_zstd(source: &Path, destination: &Path) -> io::Result<()> {
    ensure_absent(destination)?;
    run_checked(
        "/usr/bin/zstd",
        [
            OsStr::new("-19"),
            OsStr::new("--single-thread"),
            OsStr::new("--format=zstd"),
            OsStr::new("-q"),
            OsStr::new("-o"),
            destination.as_os_str(),
            OsStr::new("--"),
            source.as_os_str(),
        ],
    )
}

fn run_checked<I, S>(program: &str, arguments: I) -> io::Result<()>
where
    I: IntoIterator<Item = S>,
    S: AsRef<OsStr>,
{
    let output = Command::new(program).env_clear().args(arguments).output()?;
    if output.status.success() {
        return Ok(());
    }

    Err(io::Error::other(format!(
        "{} failed with {}: {}",
        program,
        output.status,
        String::from_utf8_lossy(&output.stderr).trim()
    )))
}

fn tool_version(program: &'static str) -> io::Result<String> {
    let output = Command::new(program)
        .env_clear()
        .arg("--version")
        .output()?;
    if !output.status.success() {
        return Err(io::Error::other(format!(
            "{program} --version failed with {}",
            output.status
        )));
    }

    String::from_utf8(output.stdout)
        .map(|value| value.trim_end().to_owned())
        .map_err(|error| io::Error::new(io::ErrorKind::InvalidData, error))
}

fn signing_fingerprint(
    certificate: &openpgp::Cert,
    policy: &StandardPolicy,
) -> openpgp::Result<String> {
    let key = certificate
        .keys()
        .unencrypted_secret()
        .with_policy(policy, None)
        .supported()
        .alive()
        .revoked(false)
        .for_signing()
        .next()
        .ok_or_else(|| invalid_data("generated certificate has no usable signing key"))?;
    Ok(key.key().fingerprint().to_hex())
}

fn detached_signature(
    certificate: &openpgp::Cert,
    policy: &StandardPolicy,
    expected_fingerprint: &str,
    bytes: &[u8],
) -> openpgp::Result<Vec<u8>> {
    let signing_key = certificate
        .keys()
        .unencrypted_secret()
        .with_policy(policy, None)
        .supported()
        .alive()
        .revoked(false)
        .for_signing()
        .next()
        .ok_or_else(|| invalid_data("generated certificate has no usable signing key"))?;

    if signing_key.key().fingerprint().to_hex() != expected_fingerprint {
        return Err(invalid_data("selected signing subkey changed unexpectedly").into());
    }

    let keypair = signing_key.key().clone().into_keypair()?;
    let mut signature = Vec::new();
    {
        let message = Message::new(&mut signature);
        let mut signer = Signer::new(message, keypair)?
            .detached()
            .hash_algo(HashAlgorithm::SHA256)?
            .build()?;
        signer.write_all(bytes)?;
        signer.finalize()?;
    }
    Ok(signature)
}

#[allow(clippy::too_many_arguments)]
fn archive_identity(
    role: &'static str,
    variant: &'static str,
    filename: &'static str,
    mtime: i64,
    member_path: &'static str,
    member_bytes: &'static [u8],
    archive_bytes: &[u8],
    signature: Option<FileIdentity>,
    stale_signature_filename: Option<&'static str>,
    relationship: &'static str,
) -> ArchiveIdentity {
    ArchiveIdentity {
        role,
        variant,
        filename,
        tar_member_mtime_epoch: mtime,
        member: MemberIdentity {
            path: member_path,
            content_utf8: std::str::from_utf8(member_bytes)
                .expect("fixture member constants are valid UTF-8"),
            byte_length: member_bytes.len(),
            sha256: sha256_hex(member_bytes),
            sha512: sha512_hex(member_bytes),
        },
        archive: file_identity(filename, archive_bytes),
        signature,
        stale_signature_filename,
        expected_signature_relationship: relationship,
    }
}

fn file_identity(filename: &str, bytes: &[u8]) -> FileIdentity {
    FileIdentity {
        filename: filename.to_owned(),
        byte_length: bytes.len(),
        sha256: sha256_hex(bytes),
        sha512: sha512_hex(bytes),
    }
}

fn sha256_hex(bytes: &[u8]) -> String {
    hex(digest::digest(&digest::SHA256, bytes).as_ref())
}

fn sha512_hex(bytes: &[u8]) -> String {
    hex(digest::digest(&digest::SHA512, bytes).as_ref())
}

fn hex(bytes: &[u8]) -> String {
    const DIGITS: &[u8; 16] = b"0123456789abcdef";
    let mut encoded = String::with_capacity(bytes.len() * 2);
    for byte in bytes {
        encoded.push(DIGITS[(byte >> 4) as usize] as char);
        encoded.push(DIGITS[(byte & 0x0f) as usize] as char);
    }
    encoded
}

fn ensure_absent(path: &Path) -> io::Result<()> {
    if path.exists() {
        return Err(io::Error::new(
            io::ErrorKind::AlreadyExists,
            format!("refusing to overwrite {}", path.display()),
        ));
    }
    Ok(())
}

fn write_new(directory: &Path, name: &str, bytes: &[u8]) -> io::Result<()> {
    write_new_path(&directory.join(name), bytes)
}

fn write_new_path(path: &Path, bytes: &[u8]) -> io::Result<()> {
    let mut file = OpenOptions::new().write(true).create_new(true).open(path)?;
    file.write_all(bytes)?;
    file.sync_all()
}

fn invalid_data(message: &str) -> io::Error {
    io::Error::new(io::ErrorKind::InvalidData, message)
}
