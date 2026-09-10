use async_trait::async_trait;
use aws_lc_rs::digest::{digest, SHA256};
use aws_lc_rs::rand::{SecureRandom, SystemRandom};
use sequoia_openpgp as openpgp;
use std::collections::HashMap;
use std::error::Error;
use std::fmt;
use std::num::NonZeroU64;
use std::sync::{Arc, Mutex};

use openpgp::cert::{CertBuilder, CipherSuite};
use openpgp::packet::signature::SignatureBuilder;
use openpgp::policy::StandardPolicy;
use openpgp::serialize::SerializeInto;
use openpgp::types::{HashAlgorithm, KeyFlags, SignatureType};
use openpgp::{Cert, Packet, Profile};
use tough::editor::signed::SignedRole;
use tough::key_source::KeySource;
use tough::schema::key::{Key, OpenPgpKey, OpenPgpScheme};
use tough::schema::{
    DelegatedRole, DelegatedTargets, Delegations, Hashes, KeyHolder, Metafile, PathPattern,
    PathSet, Role, RoleKeys, RoleType, Root, Snapshot, Target, Targets, Timestamp,
};
use tough::sign::Sign;
use tough::TargetName;

pub(crate) type FixtureError = Box<dyn Error + Send + Sync + 'static>;

#[derive(Clone)]
pub(crate) struct CompositeSigner {
    pub(crate) cert: Cert,
    pub(crate) public_projection: Vec<u8>,
    pub(crate) signed_messages: Arc<Mutex<Vec<Vec<u8>>>>,
}

impl fmt::Debug for CompositeSigner {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("CompositeSigner")
            .field("fingerprint", &self.cert.fingerprint())
            .finish_non_exhaustive()
    }
}

impl CompositeSigner {
    pub(crate) fn generate() -> openpgp::Result<Self> {
        let (cert, _) = CertBuilder::new()
            .set_profile(Profile::RFC9580)?
            .set_cipher_suite(CipherSuite::MLDSA65_Ed25519)
            .set_primary_key_flags(KeyFlags::empty().set_certification())
            .add_userid("Disposable TUF RFC 9980 fixture <fixture.invalid>")
            .add_signing_subkey()
            .generate()?;
        let public_projection = cert.clone().strip_secret_key_material().to_vec()?;

        Ok(Self {
            cert,
            public_projection,
            signed_messages: Arc::new(Mutex::new(Vec::new())),
        })
    }
}

pub(crate) fn openpgp_key(public_projection: Vec<u8>) -> Key {
    Key::OpenPgp {
        keyval: OpenPgpKey {
            public: public_projection.into(),
            _extra: HashMap::new(),
        },
        scheme: OpenPgpScheme::OpenPgpRfc9980MlDsa65Ed25519Sha512,
        _extra: HashMap::new(),
    }
}

#[async_trait]
impl Sign for CompositeSigner {
    fn tuf_key(&self) -> Key {
        openpgp_key(self.public_projection.clone())
    }

    async fn sign(
        &self,
        msg: &[u8],
        _rng: &(dyn SecureRandom + Sync),
    ) -> Result<Vec<u8>, FixtureError> {
        self.signed_messages.lock().unwrap().push(msg.to_vec());

        let policy = StandardPolicy::new();
        let mut keypair = self
            .cert
            .keys()
            .secret()
            .with_policy(&policy, None)
            .supported()
            .alive()
            .revoked(false)
            .for_signing()
            .next()
            .ok_or_else(|| "fixture has no signing key".to_string())?
            .key()
            .clone()
            .into_keypair()?;
        let issuer = keypair.public().fingerprint();
        let signature = SignatureBuilder::new(SignatureType::Binary)
            .set_hash_algo(HashAlgorithm::SHA512)
            .set_issuer_fingerprint(issuer)?
            .sign_message(&mut keypair, msg)?;

        Ok(Packet::from(signature).to_vec()?)
    }
}

#[async_trait]
impl KeySource for CompositeSigner {
    async fn as_sign(&self) -> Result<Box<dyn Sign>, FixtureError> {
        Ok(Box::new(self.clone()))
    }

    async fn write(&self, _value: &str, _key_id_hex: &str) -> Result<(), FixtureError> {
        Err("the disposable in-memory key source is read-only".into())
    }
}

#[derive(Debug)]
pub(crate) struct PublicRepositoryFixture {
    pub(crate) public_root: Vec<u8>,
    pub(crate) metadata_by_filename: HashMap<String, Vec<u8>>,
    pub(crate) consistent_snapshot_target_path: String,
    pub(crate) target_bytes: Vec<u8>,
}

fn role_keys(key_id: tough::schema::decoded::Decoded<tough::schema::decoded::Hex>) -> RoleKeys {
    RoleKeys {
        keyids: vec![key_id],
        threshold: NonZeroU64::new(1).unwrap(),
        _extra: HashMap::new(),
    }
}

fn metafile(bytes: &[u8]) -> Metafile {
    Metafile {
        length: Some(bytes.len() as u64),
        hashes: Some(Hashes {
            sha256: digest(&SHA256, bytes).as_ref().to_vec().into(),
            _extra: HashMap::new(),
        }),
        version: NonZeroU64::new(1).unwrap(),
        _extra: HashMap::new(),
    }
}

pub(crate) async fn publish_single_delegated_target(
    selected_target_name: &str,
    selected_target_bytes: &[u8],
) -> Result<PublicRepositoryFixture, FixtureError> {
    let signer = CompositeSigner::generate()?;
    let key = signer.tuf_key();
    let key_id = key.key_id()?;
    let role_key = role_keys(key_id.clone());
    let mut roles = HashMap::new();
    roles.insert(RoleType::Root, role_key.clone());
    roles.insert(RoleType::Targets, role_key.clone());
    roles.insert(RoleType::Snapshot, role_key.clone());
    roles.insert(RoleType::Timestamp, role_key);
    let root = Root {
        spec_version: "1.0.36".to_owned(),
        consistent_snapshot: true,
        version: NonZeroU64::new(1).unwrap(),
        expires: "2999-01-01T00:00:00Z".parse()?,
        keys: HashMap::from([(key_id.clone(), key)]),
        roles,
        _extra: HashMap::new(),
    };
    let key_sources: Vec<Box<dyn KeySource>> = vec![Box::new(signer)];
    let rng = SystemRandom::new();
    let signed_root = SignedRole::new(
        root.clone(),
        &KeyHolder::Root(root.clone()),
        &key_sources,
        &rng,
    )
    .await?;

    let role_name = "delegated";
    let delegations = Delegations {
        keys: root.keys.clone(),
        roles: vec![DelegatedRole {
            name: role_name.to_owned(),
            keyids: vec![key_id],
            threshold: NonZeroU64::new(1).unwrap(),
            paths: PathSet::Paths(vec![PathPattern::new("*")?]),
            terminating: false,
            targets: None,
        }],
    };
    let top_targets = Targets {
        spec_version: "1.0.36".to_owned(),
        version: NonZeroU64::new(1).unwrap(),
        expires: "2999-01-01T00:00:00Z".parse()?,
        targets: HashMap::new(),
        delegations: Some(delegations.clone()),
        _extra: HashMap::new(),
    };
    let signed_targets = SignedRole::new(
        top_targets,
        &KeyHolder::Root(root.clone()),
        &key_sources,
        &rng,
    )
    .await?;

    let target_name = TargetName::new(selected_target_name)?;
    let target_sha256 = digest(&SHA256, selected_target_bytes);
    let target = Target {
        length: selected_target_bytes.len() as u64,
        hashes: Hashes {
            sha256: target_sha256.as_ref().to_vec().into(),
            _extra: HashMap::new(),
        },
        custom: HashMap::from([("experimental-profile".to_owned(), serde_json::json!(true))]),
        _extra: HashMap::new(),
    };
    let delegated_targets = DelegatedTargets {
        name: role_name.to_owned(),
        targets: Targets {
            spec_version: "1.0.36".to_owned(),
            version: NonZeroU64::new(1).unwrap(),
            expires: "2999-01-01T00:00:00Z".parse()?,
            targets: HashMap::from([(target_name.clone(), target)]),
            delegations: None,
            _extra: HashMap::new(),
        },
    };
    let signed_delegated = SignedRole::new(
        delegated_targets,
        &KeyHolder::Delegations(delegations),
        &key_sources,
        &rng,
    )
    .await?;

    let snapshot = Snapshot {
        spec_version: "1.0.36".to_owned(),
        version: NonZeroU64::new(1).unwrap(),
        expires: "2999-01-01T00:00:00Z".parse()?,
        meta: HashMap::from([
            ("targets.json".to_owned(), metafile(signed_targets.buffer())),
            (
                format!("{role_name}.json"),
                metafile(signed_delegated.buffer()),
            ),
        ]),
        _extra: HashMap::new(),
    };
    let signed_snapshot =
        SignedRole::new(snapshot, &KeyHolder::Root(root.clone()), &key_sources, &rng).await?;

    let mut timestamp = Timestamp::new(
        "1.0.36".to_owned(),
        NonZeroU64::new(1).unwrap(),
        "2999-01-01T00:00:00Z".parse()?,
    );
    timestamp.meta.insert(
        "snapshot.json".to_owned(),
        metafile(signed_snapshot.buffer()),
    );
    let signed_timestamp =
        SignedRole::new(timestamp, &KeyHolder::Root(root), &key_sources, &rng).await?;

    let targets_filename = signed_targets.signed().signed.filename(true);
    let delegated_filename = signed_delegated.signed().signed.filename(true);
    let snapshot_filename = signed_snapshot.signed().signed.filename(true);
    let timestamp_filename = signed_timestamp.signed().signed.filename(true);
    let metadata_by_filename = HashMap::from([
        (targets_filename, signed_targets.buffer().clone()),
        (delegated_filename, signed_delegated.buffer().clone()),
        (snapshot_filename, signed_snapshot.buffer().clone()),
        (timestamp_filename, signed_timestamp.buffer().clone()),
    ]);
    let digest_prefix = target_sha256
        .as_ref()
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect::<String>();

    Ok(PublicRepositoryFixture {
        public_root: signed_root.buffer().clone(),
        metadata_by_filename,
        consistent_snapshot_target_path: format!("{digest_prefix}.{}", target_name.resolved()),
        target_bytes: selected_target_bytes.to_vec(),
    })
}
