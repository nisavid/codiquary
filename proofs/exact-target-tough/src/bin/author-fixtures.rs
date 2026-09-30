//! Authors the single public positive fixture for the disposable exact-target proof.

#![forbid(unsafe_code)]

use sequoia_openpgp as openpgp;
use sha2::{Digest, Sha256};
use std::collections::{HashMap, HashSet};
use std::env;
use std::error::Error;
use std::ffi::OsStr;
use std::fmt;
use std::fs::{self, OpenOptions};
use std::io::Write;
use std::num::NonZeroU64;
use std::path::{Path, PathBuf};
use std::process::ExitCode;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use openpgp::cert::{CertBuilder, CipherSuite};
use openpgp::packet::signature::SignatureBuilder;
use openpgp::parse::Parse;
use openpgp::policy::StandardPolicy;
use openpgp::serialize::SerializeInto;
use openpgp::types::{HashAlgorithm, KeyFlags, PublicKeyAlgorithm, SignatureType};
use openpgp::{Cert, Fingerprint, Packet, PacketPile, Profile};
use serde::Serialize;
use serde_json::{Value, json};
use tough::TargetName;
use tough::schema::decoded::{Decoded, Hex};
use tough::schema::{
    Hashes, Metafile, Role, RoleKeys, RoleType, Signature, Signed, Snapshot, Target, Targets,
    Timestamp,
};

type AuthorResult<T> = Result<T, Box<dyn Error + Send + Sync + 'static>>;

const SPEC_VERSION: &str = "1.0.36";
const FIXTURE_VERIFICATION_INSTANT: &str = "2030-01-01T00:00:00Z";
const METADATA_EXPIRES: &str = "2031-01-01T00:00:00Z";
const OPENPGP_CREATION_UNIX_SECONDS: u64 = 1_735_689_600;

const TARGET_NAME: &str = "exact-match.txt";
const TARGET_BODY: &[u8] = b"Codiquary synthetic exact-target fixture.\n";
const BINDING_NAME: &str = "io.nisavid.codiquary.exact-target/v1";

const PRODUCT: &str = "io.nisavid.codiquary.synthetic-widget";
const VERSION: &str = "1.0.0";
const CHANNEL: &str = "fixture";
const PURPOSE: &str = "verification";
const PLATFORM: &str = "portable";
const POLICY_PROFILE: &str = "synthetic-public-v1";

const OPENPGP_KEY_TYPE: &str = "openpgp-rfc9580";
const OPENPGP_SCHEME: &str = "openpgp-rfc9980-ml-dsa-65+ed25519-sha512";

#[derive(Debug)]
struct AuthorError(String);

impl fmt::Display for AuthorError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(&self.0)
    }
}

impl Error for AuthorError {}

fn fail(message: impl Into<String>) -> Box<dyn Error + Send + Sync + 'static> {
    Box::new(AuthorError(message.into()))
}

#[derive(Clone, Serialize)]
struct AuthorOpenPgpKey {
    keytype: &'static str,
    keyval: AuthorOpenPgpKeyValue,
    scheme: &'static str,
}

#[derive(Clone, Serialize)]
struct AuthorOpenPgpKeyValue {
    public: Decoded<Hex>,
}

#[derive(Serialize)]
#[serde(tag = "_type", rename = "root")]
struct AuthorRoot {
    spec_version: &'static str,
    consistent_snapshot: bool,
    version: NonZeroU64,
    expires: &'static str,
    keys: HashMap<Decoded<Hex>, AuthorOpenPgpKey>,
    roles: HashMap<RoleType, RoleKeys>,
}

impl Role for AuthorRoot {
    const TYPE: RoleType = RoleType::Root;

    fn expires(&self) -> jiff::Timestamp {
        self.expires
            .parse()
            .expect("the fixed metadata expiry is valid RFC 3339")
    }

    fn version(&self) -> NonZeroU64 {
        self.version
    }

    fn filename(&self, _consistent_snapshot: bool) -> String {
        format!("{}.root.json", self.version)
    }
}

/// Adapts a closed key wire value to Tough's public canonicalizer.
#[derive(Serialize)]
#[serde(transparent)]
struct CanonicalKey<'a>(&'a AuthorOpenPgpKey);

impl Role for CanonicalKey<'_> {
    const TYPE: RoleType = RoleType::Root;

    fn expires(&self) -> jiff::Timestamp {
        unreachable!("canonical key adapter has no expiry")
    }

    fn version(&self) -> NonZeroU64 {
        unreachable!("canonical key adapter has no version")
    }

    fn filename(&self, _consistent_snapshot: bool) -> String {
        unreachable!("canonical key adapter has no filename")
    }
}

struct RoleSigner {
    role: &'static str,
    secret_certificate: Cert,
    public_certificate: Vec<u8>,
    tuf_key: AuthorOpenPgpKey,
    key_id: Decoded<Hex>,
    signing_fingerprint: Fingerprint,
}

impl fmt::Debug for RoleSigner {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("RoleSigner")
            .field("role", &self.role)
            .field("signing_fingerprint", &self.signing_fingerprint)
            .finish_non_exhaustive()
    }
}

impl RoleSigner {
    fn generate(role: &'static str) -> AuthorResult<Self> {
        let creation_time = openpgp_creation_time();
        let (secret_certificate, _) = CertBuilder::new()
            .set_profile(Profile::RFC9580)?
            .set_cipher_suite(CipherSuite::MLDSA65_Ed25519)
            .set_creation_time(creation_time)
            .set_primary_key_flags(KeyFlags::empty().set_certification())
            .add_userid(format!("Codiquary disposable {role} fixture"))
            .add_signing_subkey()
            .generate()?;

        if !secret_certificate.is_tsk() {
            return Err(fail("generated certificate has no in-memory secret material"));
        }

        let public_certificate = secret_certificate
            .clone()
            .strip_secret_key_material()
            .to_vec()?;
        let signing_fingerprint = validate_public_certificate(&public_certificate)?;
        let tuf_key = AuthorOpenPgpKey {
            keytype: OPENPGP_KEY_TYPE,
            keyval: AuthorOpenPgpKeyValue {
                public: public_certificate.clone().into(),
            },
            scheme: OPENPGP_SCHEME,
        };
        let tuf_key_wire = serde_json::to_value(&tuf_key)?;
        if tuf_key_wire["keytype"] != OPENPGP_KEY_TYPE
            || tuf_key_wire["scheme"] != OPENPGP_SCHEME
        {
            return Err(fail("TUF key wire labels do not match the accepted profile"));
        }
        let canonical_key = CanonicalKey(&tuf_key).canonical_form()?;
        let key_id = Sha256::digest(canonical_key).to_vec().into();

        Ok(Self {
            role,
            secret_certificate,
            public_certificate,
            tuf_key,
            key_id,
            signing_fingerprint,
        })
    }

    fn sign(&self, message: &[u8]) -> AuthorResult<Vec<u8>> {
        let policy = StandardPolicy::new();
        let mut signing_keys = self
            .secret_certificate
            .keys()
            .secret()
            .with_policy(&policy, Some(fixture_verification_time()))
            .supported()
            .alive()
            .revoked(false)
            .for_signing();
        let signing_key = signing_keys.next().ok_or_else(|| {
            fail(format!(
                "{} certificate has no eligible composite signing key",
                self.role
            ))
        })?;
        if signing_keys.next().is_some() {
            return Err(fail(format!(
                "{} certificate has more than one eligible signing key",
                self.role
            )));
        }
        if signing_key.key().fingerprint() != self.signing_fingerprint {
            return Err(fail(format!(
                "{} signing-key identity changed after public projection",
                self.role
            )));
        }

        let mut keypair = signing_key.key().clone().into_keypair()?;
        let signature = SignatureBuilder::new(SignatureType::Binary)
            .set_hash_algo(HashAlgorithm::SHA512)
            .set_signature_creation_time(openpgp_creation_time())?
            .set_issuer_fingerprint(self.signing_fingerprint.clone())?
            .sign_message(&mut keypair, message)?;
        let signature_bytes = Packet::from(signature).to_vec()?;
        validate_detached_signature(&signature_bytes, &self.signing_fingerprint)?;
        Ok(signature_bytes)
    }
}

struct FixtureFile {
    relative_path: PathBuf,
    bytes: Vec<u8>,
}

fn main() -> ExitCode {
    match run() {
        Ok(()) => ExitCode::SUCCESS,
        Err(error) => {
            eprintln!("author-fixtures failed: {error}");
            ExitCode::FAILURE
        }
    }
}

fn run() -> AuthorResult<()> {
    let output = parse_output_argument()?;
    require_empty_directory(&output)?;
    let files = author_positive_fixture()?;
    write_fixture(&output, files)
}

fn parse_output_argument() -> AuthorResult<PathBuf> {
    let mut arguments = env::args_os();
    let _program = arguments.next();
    let flag = arguments
        .next()
        .ok_or_else(|| fail("usage: author-fixtures --output <empty-directory>"))?;
    if flag != OsStr::new("--output") {
        return Err(fail(
            "usage: author-fixtures --output <empty-directory>",
        ));
    }
    let output = arguments
        .next()
        .ok_or_else(|| fail("usage: author-fixtures --output <empty-directory>"))?;
    if arguments.next().is_some() {
        return Err(fail(
            "usage: author-fixtures --output <empty-directory>",
        ));
    }
    Ok(PathBuf::from(output))
}

fn require_empty_directory(output: &Path) -> AuthorResult<()> {
    let metadata = fs::symlink_metadata(output).map_err(|error| {
        fail(format!(
            "output must be an existing empty directory ({}): {error}",
            output.display()
        ))
    })?;
    if !metadata.file_type().is_dir() {
        return Err(fail(format!(
            "output is not a directory: {}",
            output.display()
        )));
    }
    if fs::read_dir(output)?.next().transpose()?.is_some() {
        return Err(fail(format!(
            "output directory is not empty: {}",
            output.display()
        )));
    }
    Ok(())
}

#[expect(
    clippy::too_many_lines,
    reason = "the one positive fixture is authored as a single auditable transaction"
)]
fn author_positive_fixture() -> AuthorResult<Vec<FixtureFile>> {
    let root_signer = RoleSigner::generate("root")?;
    let targets_signer = RoleSigner::generate("targets")?;
    let snapshot_signer = RoleSigner::generate("snapshot")?;
    let timestamp_signer = RoleSigner::generate("timestamp")?;
    ensure_independent_roles([
        &root_signer,
        &targets_signer,
        &snapshot_signer,
        &timestamp_signer,
    ])?;

    let root = AuthorRoot {
        spec_version: SPEC_VERSION,
        consistent_snapshot: true,
        version: NonZeroU64::MIN,
        expires: METADATA_EXPIRES,
        keys: [
            &root_signer,
            &targets_signer,
            &snapshot_signer,
            &timestamp_signer,
        ]
        .into_iter()
        .map(|signer| (signer.key_id.clone(), signer.tuf_key.clone()))
        .collect(),
        roles: HashMap::from([
            (RoleType::Root, role_keys(&root_signer)),
            (RoleType::Targets, role_keys(&targets_signer)),
            (RoleType::Snapshot, role_keys(&snapshot_signer)),
            (RoleType::Timestamp, role_keys(&timestamp_signer)),
        ]),
    };
    let signed_root = sign_role(root, &root_signer)?;

    let target_digest = Sha256::digest(TARGET_BODY).to_vec();
    let target_digest_hex = lowercase_hex(&target_digest);
    let target_length = u64::try_from(TARGET_BODY.len())
        .map_err(|_| fail("target length cannot be represented as u64"))?;
    let target_name = TargetName::new(TARGET_NAME)?;
    let target = Target {
        length: target_length,
        hashes: Hashes {
            sha256: target_digest.clone().into(),
            _extra: HashMap::new(),
        },
        custom: HashMap::from([(BINDING_NAME.to_owned(), application_intent())]),
        _extra: HashMap::new(),
    };
    let targets = Targets {
        spec_version: SPEC_VERSION.to_owned(),
        version: NonZeroU64::MIN,
        expires: METADATA_EXPIRES.parse()?,
        targets: HashMap::from([(target_name, target)]),
        delegations: None,
        _extra: HashMap::new(),
    };
    let signed_targets = sign_role(targets, &targets_signer)?;

    let snapshot = Snapshot {
        spec_version: SPEC_VERSION.to_owned(),
        version: NonZeroU64::MIN,
        expires: METADATA_EXPIRES.parse()?,
        meta: HashMap::from([(
            "targets.json".to_owned(),
            metadata_descriptor(&signed_targets)?,
        )]),
        _extra: HashMap::new(),
    };
    let signed_snapshot = sign_role(snapshot, &snapshot_signer)?;

    let timestamp = Timestamp {
        spec_version: SPEC_VERSION.to_owned(),
        version: NonZeroU64::MIN,
        expires: METADATA_EXPIRES.parse()?,
        meta: HashMap::from([(
            "snapshot.json".to_owned(),
            metadata_descriptor(&signed_snapshot)?,
        )]),
        _extra: HashMap::new(),
    };
    let signed_timestamp = sign_role(timestamp, &timestamp_signer)?;

    let expected_target_request = format!("{target_digest_hex}.{TARGET_NAME}");
    let selection = json_bytes(&application_intent())?;
    let expected = json_bytes(&json!({
        "case_id": "exact-match",
        "expected_authenticated_target": {
            "identity": TARGET_NAME,
            "length": target_length,
            "sha256": target_digest_hex,
        },
        "expected_helper_call_count": 1,
        "expected_metadata_request_count": 4,
        "expected_metadata_requests": [
            "2.root.json",
            "timestamp.json",
            "1.snapshot.json",
            "1.targets.json",
        ],
        "expected_semantic_observation": "verified_exact_target",
        "expected_target_body_request": expected_target_request.clone(),
        "expected_target_body_request_count": 1,
        "fixed_verification_instant": FIXTURE_VERIFICATION_INSTANT,
        "same_held_allocation": true,
    }))?;

    Ok(vec![
        FixtureFile {
            relative_path: "public-keys/root.pgp".into(),
            bytes: root_signer.public_certificate,
        },
        FixtureFile {
            relative_path: "public-keys/targets.pgp".into(),
            bytes: targets_signer.public_certificate,
        },
        FixtureFile {
            relative_path: "public-keys/snapshot.pgp".into(),
            bytes: snapshot_signer.public_certificate,
        },
        FixtureFile {
            relative_path: "public-keys/timestamp.pgp".into(),
            bytes: timestamp_signer.public_certificate,
        },
        FixtureFile {
            relative_path: "metadata/base/1.root.json".into(),
            bytes: signed_root,
        },
        FixtureFile {
            relative_path: "metadata/base/1.targets.json".into(),
            bytes: signed_targets,
        },
        FixtureFile {
            relative_path: "metadata/base/1.snapshot.json".into(),
            bytes: signed_snapshot,
        },
        FixtureFile {
            relative_path: "metadata/base/timestamp.json".into(),
            bytes: signed_timestamp,
        },
        FixtureFile {
            relative_path: format!("targets/{expected_target_request}").into(),
            bytes: TARGET_BODY.to_vec(),
        },
        FixtureFile {
            relative_path: "selections/exact-match.json".into(),
            bytes: selection,
        },
        FixtureFile {
            relative_path: "expected/exact-match.json".into(),
            bytes: expected,
        },
    ])
}

fn ensure_independent_roles(signers: [&RoleSigner; 4]) -> AuthorResult<()> {
    let key_ids: HashSet<Vec<u8>> = signers
        .iter()
        .map(|signer| signer.key_id.as_ref().to_vec())
        .collect();
    let fingerprints: HashSet<String> = signers
        .iter()
        .map(|signer| signer.signing_fingerprint.to_string())
        .collect();
    if key_ids.len() != signers.len() || fingerprints.len() != signers.len() {
        return Err(fail(
            "role generation did not produce four independent signing identities",
        ));
    }
    Ok(())
}

fn role_keys(signer: &RoleSigner) -> RoleKeys {
    RoleKeys {
        keyids: vec![signer.key_id.clone()],
        threshold: NonZeroU64::MIN,
        _extra: HashMap::new(),
    }
}

fn sign_role<T>(role: T, signer: &RoleSigner) -> AuthorResult<Vec<u8>>
where
    T: Role,
{
    let canonical = role.canonical_form()?;
    let signature = signer.sign(&canonical)?;
    let signed = Signed {
        signed: role,
        signatures: vec![Signature {
            keyid: signer.key_id.clone(),
            sig: signature.into(),
        }],
    };
    json_bytes(&signed)
}

fn metadata_descriptor(bytes: &[u8]) -> AuthorResult<Metafile> {
    let length = u64::try_from(bytes.len())
        .map_err(|_| fail("metadata length cannot be represented as u64"))?;
    Ok(Metafile {
        length: Some(length),
        hashes: Some(Hashes {
            sha256: Sha256::digest(bytes).to_vec().into(),
            _extra: HashMap::new(),
        }),
        version: NonZeroU64::MIN,
        _extra: HashMap::new(),
    })
}

fn application_intent() -> Value {
    json!({
        "channel": CHANNEL,
        "platform": PLATFORM,
        "policy_profile": POLICY_PROFILE,
        "product": PRODUCT,
        "purpose": PURPOSE,
        "version": VERSION,
    })
}

fn validate_public_certificate(bytes: &[u8]) -> AuthorResult<Fingerprint> {
    let certificate = Cert::from_bytes(bytes)?;
    if certificate.is_tsk() {
        return Err(fail("public projection contains secret key material"));
    }
    if certificate.clone().to_vec()? != bytes {
        return Err(fail(
            "public certificate is not in canonical unarmored binary form",
        ));
    }
    let primary = certificate.primary_key().key();
    if primary.version() != 6 || primary.pk_algo() != PublicKeyAlgorithm::MLDSA65_Ed25519 {
        return Err(fail(
            "public certificate primary key is not v6 algorithm 30",
        ));
    }

    let policy = StandardPolicy::new();
    let signing_keys: Vec<_> = certificate
        .keys()
        .with_policy(&policy, Some(fixture_verification_time()))
        .supported()
        .alive()
        .revoked(false)
        .for_signing()
        .collect();
    if signing_keys.len() != 1 {
        return Err(fail(
            "public certificate must contain exactly one eligible signing key",
        ));
    }
    let signing_key = signing_keys[0].key();
    if signing_key.version() != 6
        || signing_key.pk_algo() != PublicKeyAlgorithm::MLDSA65_Ed25519
    {
        return Err(fail("eligible signing key is not v6 algorithm 30"));
    }
    Ok(signing_key.fingerprint())
}

fn validate_detached_signature(bytes: &[u8], expected_issuer: &Fingerprint) -> AuthorResult<()> {
    if !matches!(expected_issuer, Fingerprint::V6(_)) {
        return Err(fail("detached signature issuer fingerprint is not v6"));
    }
    let packets: Vec<Packet> = PacketPile::from_bytes(bytes)?.into_children().collect();
    let [Packet::Signature(signature)] = packets.as_slice() else {
        return Err(fail(
            "detached signature is not exactly one signature packet",
        ));
    };
    if Packet::from(signature.clone()).to_vec()? != bytes {
        return Err(fail(
            "detached signature is not in canonical unarmored binary form",
        ));
    }
    if signature.version() != 6
        || signature.typ() != SignatureType::Binary
        || signature.pk_algo() != PublicKeyAlgorithm::MLDSA65_Ed25519
        || signature.hash_algo() != HashAlgorithm::SHA512
    {
        return Err(fail(
            "detached signature does not match the accepted composite profile",
        ));
    }
    let issuers: Vec<Fingerprint> = signature.issuer_fingerprints().cloned().collect();
    if issuers.as_slice() != [expected_issuer.clone()] {
        return Err(fail(
            "detached signature must contain one matching v6 issuer fingerprint",
        ));
    }
    Ok(())
}

fn json_bytes<T: Serialize>(value: &T) -> AuthorResult<Vec<u8>> {
    let mut bytes = serde_json::to_vec_pretty(value)?;
    bytes.push(b'\n');
    Ok(bytes)
}

fn lowercase_hex(bytes: &[u8]) -> String {
    const DIGITS: &[u8; 16] = b"0123456789abcdef";
    let mut encoded = String::with_capacity(bytes.len() * 2);
    for byte in bytes {
        encoded.push(char::from(DIGITS[usize::from(byte >> 4)]));
        encoded.push(char::from(DIGITS[usize::from(byte & 0x0f)]));
    }
    encoded
}

fn openpgp_creation_time() -> SystemTime {
    UNIX_EPOCH + Duration::from_secs(OPENPGP_CREATION_UNIX_SECONDS)
}

fn fixture_verification_time() -> SystemTime {
    UNIX_EPOCH + Duration::from_secs(1_893_456_000)
}

fn write_fixture(output: &Path, files: Vec<FixtureFile>) -> AuthorResult<()> {
    for directory in [
        "public-keys",
        "metadata",
        "metadata/base",
        "targets",
        "selections",
        "expected",
    ] {
        fs::create_dir(output.join(directory))?;
    }

    for file in files {
        let path = output.join(&file.relative_path);
        let mut destination = OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(&path)?;
        destination.write_all(&file.bytes)?;
        destination.sync_all()?;
    }
    Ok(())
}
