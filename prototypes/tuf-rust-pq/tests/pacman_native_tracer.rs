use std::fs;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

#[path = "support/pacman_native_fixture.rs"]
mod pacman_native_fixture;

use pacman_native_fixture::*;

fn load_package_bytes(fixture_dir: &Path, fixture: PackageFixture) -> Vec<u8> {
    fs::read(fixture_dir.join(fixture.filename()))
        .expect("caller must hold the selected package bytes before verification")
}

fn create_fixture_companion_dir(source: &Path) -> PathBuf {
    let timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("system clock must follow the Unix epoch")
        .as_nanos();
    let destination = std::env::temp_dir()
        .join("codiquary17-native")
        .join(format!(
            "fixture-companions-{timestamp}-{}",
            std::process::id()
        ));
    fs::create_dir_all(&destination)
        .expect("disposable fixture-companion directory must be created");

    for filename in [
        "signer.pub",
        "codiquary17-native-1.0-1-any.pkg.tar.zst.sig",
        "codiquary17.db",
        "codiquary17.db.sig",
    ] {
        fs::copy(source.join(filename), destination.join(filename))
            .expect("frozen companion must be copied into the disposable directory");
    }

    destination
}

#[test]
fn native_positive_path_verifies_package_and_database_before_one_consumer_call() {
    let fixture_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/data/pacman-native");
    let package_fixture = PackageFixture::Original;
    let package_bytes = load_package_bytes(&fixture_dir, package_fixture);
    let request = NativeProbeRequest {
        fixture_dir,
        scratch_dir: std::env::temp_dir().join("codiquary17-native").join("run"),
        held_package: HeldPackageBytes {
            expected_sha256: package_fixture.sha256(),
            bytes: &package_bytes,
        },
        database_fixture: DatabaseFixture::Original,
        package_signature_check: PackageSignatureCheck::Required,
        database_signature_check: DatabaseSignatureCheck::Required,
        primary_certificate_fingerprint: PRIMARY_CERTIFICATE_FINGERPRINT,
        signing_subkey_fingerprint: SIGNING_SUBKEY_FINGERPRINT,
        expected_native_signature_fingerprint: EXPECTED_NATIVE_SIGNATURE_FINGERPRINT,
    };

    let expected = expected_positive_evidence();
    let mut consumer_calls = 0;
    let actual = match verify_native_before_consumer(&request, || {
        fake_consumer(&mut consumer_calls)
    }) {
        Ok(evidence) => evidence,
        Err(NativeProbeError::Failed {
            attempt_dir,
            message,
        }) => panic!(
            "native libalpm positive-path probe failed; retained attempt {}: {}",
            attempt_dir.display(),
            message
        ),
        Err(NativeProbeError::Rejected {
            attempt_dir,
            receipt,
        }) => panic!(
            "native libalpm positive-path probe rejected the fixture; retained attempt {}: status={:?}, stdout={:?}, stderr={:?}",
            attempt_dir.display(),
            receipt.status_code,
            String::from_utf8_lossy(&receipt.stdout),
            String::from_utf8_lossy(&receipt.stderr)
        ),
        Err(NativeProbeError::UnsafeNativeEvidence { evidence }) => panic!(
            "native libalpm positive-path evidence was rejected by the caller: {evidence:?}"
        ),
    };

    assert_eq!(actual, expected, "native evidence must match exactly");
    assert_eq!(consumer_calls, 1);
}

#[test]
fn native_verifies_held_package_when_original_fixture_path_is_unavailable() {
    let source_fixture_dir =
        PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/data/pacman-native");
    let package_fixture = PackageFixture::Original;
    let package_bytes = load_package_bytes(&source_fixture_dir, package_fixture);
    let fixture_dir = create_fixture_companion_dir(&source_fixture_dir);
    let unavailable_package_path = fixture_dir.join(package_fixture.filename());
    assert!(
        !unavailable_package_path.exists(),
        "disposable fixture-companion directory must not contain the package"
    );

    let request = NativeProbeRequest {
        fixture_dir,
        scratch_dir: std::env::temp_dir().join("codiquary17-native").join("run"),
        held_package: HeldPackageBytes {
            expected_sha256: package_fixture.sha256(),
            bytes: &package_bytes,
        },
        database_fixture: DatabaseFixture::Original,
        package_signature_check: PackageSignatureCheck::Required,
        database_signature_check: DatabaseSignatureCheck::Required,
        primary_certificate_fingerprint: PRIMARY_CERTIFICATE_FINGERPRINT,
        signing_subkey_fingerprint: SIGNING_SUBKEY_FINGERPRINT,
        expected_native_signature_fingerprint: EXPECTED_NATIVE_SIGNATURE_FINGERPRINT,
    };

    let actual = match verify_native_held_package(&request) {
        Ok(evidence) => evidence,
        Err(NativeProbeError::Failed {
            attempt_dir,
            message,
        }) => panic!(
            "held package bytes did not suffice; retained attempt {}: {}",
            attempt_dir.display(),
            message
        ),
        Err(NativeProbeError::Rejected {
            attempt_dir,
            receipt,
        }) => panic!(
            "native verification rejected held package bytes; retained attempt {}: status={:?}, stdout={:?}, stderr={:?}",
            attempt_dir.display(),
            receipt.status_code,
            String::from_utf8_lossy(&receipt.stdout),
            String::from_utf8_lossy(&receipt.stderr)
        ),
        Err(NativeProbeError::UnsafeNativeEvidence { evidence }) => panic!(
            "reusable verifier unexpectedly applied consumer policy: {evidence:?}"
        ),
    };

    assert_eq!(actual, expected_positive_evidence());
}

#[test]
fn native_rejects_mtime_changed_package_with_original_signature_before_consumer() {
    let fixture_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/data/pacman-native");
    let package_fixture = PackageFixture::MtimeChangedWithOriginalSignature;
    let package_bytes = load_package_bytes(&fixture_dir, package_fixture);
    let request = NativeProbeRequest {
        fixture_dir,
        scratch_dir: std::env::temp_dir().join("codiquary17-native").join("run"),
        held_package: HeldPackageBytes {
            expected_sha256: package_fixture.sha256(),
            bytes: &package_bytes,
        },
        database_fixture: DatabaseFixture::Original,
        package_signature_check: PackageSignatureCheck::Required,
        database_signature_check: DatabaseSignatureCheck::Required,
        primary_certificate_fingerprint: PRIMARY_CERTIFICATE_FINGERPRINT,
        signing_subkey_fingerprint: SIGNING_SUBKEY_FINGERPRINT,
        expected_native_signature_fingerprint: EXPECTED_NATIVE_SIGNATURE_FINGERPRINT,
    };

    let mut consumer_calls = 0;
    let result = verify_native_before_consumer(&request, || fake_consumer(&mut consumer_calls));
    let receipt = match result {
        Err(NativeProbeError::Rejected {
            attempt_dir: _,
            receipt,
        }) => receipt,
        Err(NativeProbeError::Failed {
            attempt_dir,
            message,
        }) => panic!(
            "setup failed before native rejection was observed; retained attempt {}: {}",
            attempt_dir.display(),
            message
        ),
        Ok(evidence) => panic!(
            "mtime-changed package with original signature was unexpectedly accepted; consumer_calls={consumer_calls}: {evidence:?}"
        ),
        Err(NativeProbeError::UnsafeNativeEvidence { evidence }) => panic!(
            "caller policy rejected evidence instead of observing native package rejection: {evidence:?}"
        ),
    };

    assert_eq!(receipt.status_code, Some(1), "native probe must reject");
    assert!(
        receipt.stdout.is_empty(),
        "rejection must not emit success evidence"
    );
    assert_eq!(
        std::str::from_utf8(&receipt.stderr).expect("native stderr must be UTF-8"),
        "alpm_pkg_load returned -1, errno_ok=0, package_non_null=0: invalid or corrupted package (PGP signature)\nnative probe failed; question callback calls=0\n",
        "native stderr must identify the libalpm signature rejection"
    );
    assert_eq!(
        consumer_calls, 0,
        "rejected input must not reach the consumer"
    );
}

#[test]
fn native_rejects_mtime_changed_database_with_original_signature_after_valid_package_before_consumer(
) {
    let fixture_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/data/pacman-native");
    let package_fixture = PackageFixture::Original;
    let package_bytes = load_package_bytes(&fixture_dir, package_fixture);
    let request = NativeProbeRequest {
        fixture_dir,
        scratch_dir: std::env::temp_dir().join("codiquary17-native").join("run"),
        held_package: HeldPackageBytes {
            expected_sha256: package_fixture.sha256(),
            bytes: &package_bytes,
        },
        database_fixture: DatabaseFixture::MtimeChangedWithOriginalSignature,
        package_signature_check: PackageSignatureCheck::Required,
        database_signature_check: DatabaseSignatureCheck::Required,
        primary_certificate_fingerprint: PRIMARY_CERTIFICATE_FINGERPRINT,
        signing_subkey_fingerprint: SIGNING_SUBKEY_FINGERPRINT,
        expected_native_signature_fingerprint: EXPECTED_NATIVE_SIGNATURE_FINGERPRINT,
    };

    let mut consumer_calls = 0;
    let result = verify_native_before_consumer(&request, || fake_consumer(&mut consumer_calls));
    let (attempt_dir, receipt) = match result {
        Err(NativeProbeError::Rejected {
            attempt_dir,
            receipt,
        }) => (attempt_dir, receipt),
        Err(NativeProbeError::Failed {
            attempt_dir,
            message,
        }) => panic!(
            "setup failed before native database rejection was observed; retained attempt {}: {}",
            attempt_dir.display(),
            message
        ),
        Ok(evidence) => panic!(
            "mtime-changed database with original signature was unexpectedly accepted; consumer_calls={consumer_calls}: {evidence:?}"
        ),
        Err(NativeProbeError::UnsafeNativeEvidence { evidence }) => panic!(
            "caller policy rejected evidence instead of observing native database rejection: {evidence:?}"
        ),
    };

    assert_eq!(
        receipt.status_code,
        Some(1),
        "native probe must reject the database"
    );
    assert!(
        receipt.stdout.is_empty(),
        "rejection must not emit success evidence"
    );

    let retained_status =
        fs::read_to_string(attempt_dir.join("output/run-pacman-native-probe.status"))
            .expect("native probe exit status must be retained with the attempt");
    assert_eq!(
        retained_status, "exit_code=1\n",
        "retained status must replay the observed native rejection"
    );

    let stderr = std::str::from_utf8(&receipt.stderr).expect("native stderr must be UTF-8");
    let mut stderr_lines = stderr.lines();
    let database_rejection = stderr_lines
        .next()
        .expect("native stderr must contain the database rejection");
    let database_error_code = database_rejection
        .strip_prefix("alpm_db_get_valid: invalid or corrupted database (PGP signature) (")
        .and_then(|suffix| suffix.strip_suffix(')'))
        .expect("native rejection must come from database signature verification")
        .parse::<i32>()
        .expect("native database rejection must include its libalpm error code");
    assert_ne!(
        database_error_code, 0,
        "database signature rejection must report a nonzero libalpm error"
    );
    assert_eq!(
        stderr_lines.next(),
        Some("native probe failed; question callback calls=0"),
        "native rejection must not invoke the question callback"
    );
    assert_eq!(
        stderr_lines.next(),
        None,
        "native rejection must not contain unrelated package, configuration, or setup errors"
    );
    assert_eq!(
        consumer_calls, 0,
        "rejected input must not reach the consumer"
    );
}

#[test]
fn native_rejects_disabled_database_signature_check_after_valid_package_before_consumer() {
    let fixture_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/data/pacman-native");
    let package_fixture = PackageFixture::Original;
    let package_bytes = load_package_bytes(&fixture_dir, package_fixture);
    let request = NativeProbeRequest {
        fixture_dir,
        scratch_dir: std::env::temp_dir().join("codiquary17-native").join("run"),
        held_package: HeldPackageBytes {
            expected_sha256: package_fixture.sha256(),
            bytes: &package_bytes,
        },
        database_fixture: DatabaseFixture::Original,
        package_signature_check: PackageSignatureCheck::Required,
        database_signature_check: DatabaseSignatureCheck::Disabled,
        primary_certificate_fingerprint: PRIMARY_CERTIFICATE_FINGERPRINT,
        signing_subkey_fingerprint: SIGNING_SUBKEY_FINGERPRINT,
        expected_native_signature_fingerprint: EXPECTED_NATIVE_SIGNATURE_FINGERPRINT,
    };

    let mut consumer_calls = 0;
    let result = verify_native_before_consumer(&request, || fake_consumer(&mut consumer_calls));
    assert_eq!(
        consumer_calls, 0,
        "disabled database signature checking must not reach the consumer"
    );

    let evidence = match result {
        Err(NativeProbeError::UnsafeNativeEvidence { evidence }) => *evidence,
        Err(NativeProbeError::Failed {
            attempt_dir,
            message,
        }) => panic!(
            "setup failed before disabled native database checking was observed; retained attempt {}: {}",
            attempt_dir.display(),
            message
        ),
        Err(NativeProbeError::Rejected {
            attempt_dir,
            receipt,
        }) => panic!(
            "native helper rejected instead of reporting its disabled database check; retained attempt {}: status={:?}, stdout={:?}, stderr={:?}",
            attempt_dir.display(),
            receipt.status_code,
            String::from_utf8_lossy(&receipt.stdout),
            String::from_utf8_lossy(&receipt.stderr)
        ),
        Ok(evidence) => panic!(
            "caller accepted actual native evidence with disabled database signature checking; consumer_calls={consumer_calls}: {evidence:?}"
        ),
    };

    assert_eq!(
        evidence.package.configured_siglevel,
        REQUIRED_PACKAGE_SIGLEVEL
    );
    assert_eq!(evidence.database.configured_siglevel, 0);

    let mut expected = expected_positive_evidence();
    expected.database.configured_siglevel = 0;
    assert_eq!(
        evidence, expected,
        "disabled database evidence must otherwise preserve the observed primary-certificate interpretation"
    );
}

#[test]
fn native_rejects_disabled_package_signature_check_before_consumer() {
    let fixture_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/data/pacman-native");
    let package_fixture = PackageFixture::Original;
    let package_bytes = load_package_bytes(&fixture_dir, package_fixture);
    let request = NativeProbeRequest {
        fixture_dir,
        scratch_dir: std::env::temp_dir().join("codiquary17-native").join("run"),
        held_package: HeldPackageBytes {
            expected_sha256: package_fixture.sha256(),
            bytes: &package_bytes,
        },
        database_fixture: DatabaseFixture::Original,
        package_signature_check: PackageSignatureCheck::Disabled,
        database_signature_check: DatabaseSignatureCheck::Required,
        primary_certificate_fingerprint: PRIMARY_CERTIFICATE_FINGERPRINT,
        signing_subkey_fingerprint: SIGNING_SUBKEY_FINGERPRINT,
        expected_native_signature_fingerprint: EXPECTED_NATIVE_SIGNATURE_FINGERPRINT,
    };

    let mut consumer_calls = 0;
    let result = verify_native_before_consumer(&request, || fake_consumer(&mut consumer_calls));
    assert_eq!(
        consumer_calls, 0,
        "disabled package signature checking must not reach the consumer"
    );

    let evidence = match result {
        Err(NativeProbeError::UnsafeNativeEvidence { evidence }) => *evidence,
        Err(NativeProbeError::Failed {
            attempt_dir,
            message,
        }) => panic!(
            "setup failed before disabled native package checking was observed; retained attempt {}: {}",
            attempt_dir.display(),
            message
        ),
        Err(NativeProbeError::Rejected {
            attempt_dir,
            receipt,
        }) => panic!(
            "native helper rejected instead of reporting its disabled package check; retained attempt {}: status={:?}, stdout={:?}, stderr={:?}",
            attempt_dir.display(),
            receipt.status_code,
            String::from_utf8_lossy(&receipt.stdout),
            String::from_utf8_lossy(&receipt.stderr)
        ),
        Ok(evidence) => panic!(
            "caller accepted actual native evidence with disabled package signature checking; consumer_calls={consumer_calls}: {evidence:?}"
        ),
    };

    assert_eq!(evidence.question_callback_calls, 0);
    assert_eq!(evidence.package.configured_siglevel, 0);
    assert!(evidence.package.full_archive_requested);
    assert_eq!(evidence.package.load_return, 0);
    assert!(evidence.package.errno_ok);
    assert!(evidence.package.package_non_null);
    assert!(evidence.package.signature.is_expected());
    assert_eq!(
        evidence.database.configured_siglevel,
        REQUIRED_DATABASE_SIGLEVEL
    );
    assert!(evidence.database.database_non_null);
    assert_eq!(evidence.database.valid_return, 0);
    assert!(evidence.database.signature.is_expected());
    assert_eq!(evidence.database.package.name, "codiquary17-native");
    assert_eq!(evidence.database.package.version, "1.0-1");
    assert_eq!(
        evidence.database.package.filename,
        "codiquary17-native-1.0-1-any.pkg.tar.zst"
    );
}
