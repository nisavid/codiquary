use async_trait::async_trait;
use aws_lc_rs::digest::{digest, SHA256};
use aws_lc_rs::rand::{SecureRandom, SystemRandom};
use aws_lc_rs::signature::{EcdsaKeyPair, ECDSA_P256_SHA256_ASN1_SIGNING};
use sequoia_openpgp as openpgp;
use std::collections::HashMap;
use std::error::Error;
use std::fmt;
use std::num::NonZeroU64;
use std::sync::{Arc, Mutex};

use openpgp::cert::{CertBuilder, CipherSuite};
use openpgp::packet::signature::SignatureBuilder;
use openpgp::parse::Parse;
use openpgp::policy::StandardPolicy;
use openpgp::serialize::SerializeInto;
use openpgp::types::{HashAlgorithm, KeyFlags, SignatureType};
use openpgp::{Cert, Packet, PacketPile, Profile};
use tough::editor::signed::SignedRole;
use tough::key_source::KeySource;
use tough::schema::key::{Ed25519Key, Ed25519Scheme, Key, OpenPgpKey, OpenPgpScheme};
use tough::schema::{
    DelegatedRole, DelegatedTargets, Delegations, Hashes, KeyHolder, Metafile, PathPattern,
    PathSet, Role, RoleKeys, RoleType, Root, Signature, Signed, Snapshot, Target, Targets,
    Timestamp,
};
use tough::sign::Sign;
use tough::TargetName;
use tuf_rust_pq_prototype::experimental_profile::{
    parse_experimental_profile, Canonicalization, ClosedExtensionHandling, DescriptorHash,
    DescriptorRequirement, DescriptorRules, DetachedSignatureEncoding, OpenPgpKeyType,
    PublicCertificateEncoding, SignatureComponent, SignatureHash, SignatureScheme,
    TargetCustomHandling, ThresholdIdentity, TufSpecVersion, EXPERIMENTAL_PROFILE_JSON,
};

const ED25519_SIGNATURE_BYTES: usize = 64;
const ML_DSA_65_SIGNATURE_BYTES: usize = 3309;

#[derive(Clone)]
struct CompositeSigner {
    cert: Cert,
    public_projection: Vec<u8>,
    signed_messages: Arc<Mutex<Vec<Vec<u8>>>>,
}

impl fmt::Debug for CompositeSigner {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("CompositeSigner")
            .field("fingerprint", &self.cert.fingerprint())
            .finish_non_exhaustive()
    }
}

impl CompositeSigner {
    fn generate() -> openpgp::Result<Self> {
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

fn openpgp_key(public_projection: Vec<u8>) -> Key {
    Key::OpenPgp {
        keyval: OpenPgpKey {
            public: public_projection.into(),
            _extra: HashMap::new(),
        },
        scheme: OpenPgpScheme::OpenPgpRfc9980MlDsa65Ed25519Sha512,
        _extra: HashMap::new(),
    }
}

fn alternate_public_projection(cert: &Cert) -> openpgp::Result<Vec<u8>> {
    cert.clone()
        .insert_packets(openpgp::packet::UserID::from(
            "Alternate disposable projection <alias.invalid>",
        ))?
        .0
        .strip_secret_key_material()
        .to_vec()
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
    ) -> Result<Vec<u8>, Box<dyn Error + Send + Sync + 'static>> {
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
    async fn as_sign(&self) -> Result<Box<dyn Sign>, Box<dyn Error + Send + Sync + 'static>> {
        Ok(Box::new(self.clone()))
    }

    async fn write(
        &self,
        _value: &str,
        _key_id_hex: &str,
    ) -> Result<(), Box<dyn Error + Send + Sync + 'static>> {
        Err("the disposable in-memory key source is read-only".into())
    }
}

fn role_keys(key_id: tough::schema::decoded::Decoded<tough::schema::decoded::Hex>) -> RoleKeys {
    RoleKeys {
        keyids: vec![key_id],
        threshold: NonZeroU64::new(1).unwrap(),
        _extra: HashMap::new(),
    }
}

fn corrupt_component(signature: &[u8], from_end: usize) -> Vec<u8> {
    let mut corrupted = signature.to_vec();
    let index = corrupted
        .len()
        .checked_sub(from_end)
        .expect("signature packet contains the selected RFC 9980 component");
    corrupted[index] ^= 0x01;
    corrupted
}

fn with_undefined_openpgp_extension(mut key: Key) -> Key {
    let Key::OpenPgp { _extra, .. } = &mut key else {
        unreachable!("the composite fixture always returns an OpenPGP key");
    };
    _extra.insert("rr1-alias".to_owned(), serde_json::json!(true));
    key
}

fn with_undefined_openpgp_keyval_extension(mut key: Key) -> Key {
    let Key::OpenPgp { keyval, .. } = &mut key else {
        unreachable!("the composite fixture always returns an OpenPGP key");
    };
    keyval
        ._extra
        .insert("rr1-keyval-alias".to_owned(), serde_json::json!(true));
    key
}

fn generate_ecdsa_signer() -> Result<EcdsaKeyPair, Box<dyn Error + Send + Sync + 'static>> {
    let rng = SystemRandom::new();
    let document = EcdsaKeyPair::generate_pkcs8(&ECDSA_P256_SHA256_ASN1_SIGNING, &rng)?;
    Ok(EcdsaKeyPair::from_pkcs8(
        &ECDSA_P256_SHA256_ASN1_SIGNING,
        document.as_ref(),
    )?)
}

fn with_ecdsa_compatibility_extension(mut key: Key) -> Key {
    let Key::Ecdsa { _extra, .. } = &mut key else {
        unreachable!("the conventional fixture always returns an ECDSA key");
    };
    _extra.insert("existing-key-extension".to_owned(), serde_json::json!(true));
    key
}

#[tokio::test(flavor = "current_thread")]
async fn root_rejects_undefined_openpgp_extensions(
) -> Result<(), Box<dyn Error + Send + Sync + 'static>> {
    let profile = parse_experimental_profile(EXPERIMENTAL_PROFILE_JSON.as_bytes())?;
    assert_eq!(
        profile.extensions.openpgp_key,
        ClosedExtensionHandling::RejectUndefined
    );
    assert_eq!(
        profile.extensions.openpgp_keyval,
        ClosedExtensionHandling::RejectUndefined
    );
    let signer = CompositeSigner::generate()?;
    let key = signer.tuf_key();
    let invalid_keys = [
        with_undefined_openpgp_extension(key.clone()),
        with_undefined_openpgp_keyval_extension(key),
    ];

    for invalid_key in invalid_keys {
        let invalid_key_id = invalid_key.key_id()?;
        let root_keys = role_keys(invalid_key_id.clone());
        let mut roles = HashMap::new();
        roles.insert(RoleType::Root, root_keys.clone());
        roles.insert(RoleType::Targets, root_keys.clone());
        roles.insert(RoleType::Snapshot, root_keys.clone());
        roles.insert(RoleType::Timestamp, root_keys);
        let root = Root {
            spec_version: "1.0.36".to_owned(),
            consistent_snapshot: true,
            version: NonZeroU64::new(1).unwrap(),
            expires: "2999-01-01T00:00:00Z".parse().unwrap(),
            keys: HashMap::from([(invalid_key_id.clone(), invalid_key)]),
            roles,
            _extra: HashMap::new(),
        };
        let signature = signer
            .sign(&root.canonical_form()?, &SystemRandom::new())
            .await?;
        let signed = Signed {
            signed: root,
            signatures: vec![tough::schema::Signature {
                keyid: invalid_key_id,
                sig: signature.into(),
            }],
        };
        signed
            .signed
            .verify_role(&signed)
            .expect_err("undefined OpenPGP key fields must not verify");
        let encoded = serde_json::to_vec(&signed)?;
        assert!(
            serde_json::from_slice::<Signed<Root>>(&encoded).is_err(),
            "undefined OpenPGP key fields must be rejected during metadata ingestion"
        );
    }

    Ok(())
}

#[test]
fn conventional_keys_retain_existing_extra_field_compatibility(
) -> Result<(), Box<dyn Error + Send + Sync + 'static>> {
    let key = Key::Ed25519 {
        keyval: Ed25519Key {
            public: vec![0; 32].into(),
            _extra: HashMap::from([(
                "existing-keyval-extension".to_owned(),
                serde_json::json!(true),
            )]),
        },
        scheme: Ed25519Scheme::Ed25519,
        _extra: HashMap::from([("existing-key-extension".to_owned(), serde_json::json!(true))]),
    };
    let key_id = key.key_id()?;
    let root_keys = role_keys(key_id.clone());
    let mut roles = HashMap::new();
    roles.insert(RoleType::Root, root_keys.clone());
    roles.insert(RoleType::Targets, root_keys.clone());
    roles.insert(RoleType::Snapshot, root_keys.clone());
    roles.insert(RoleType::Timestamp, root_keys);
    let root = Root {
        spec_version: "1.0.36".to_owned(),
        consistent_snapshot: true,
        version: NonZeroU64::new(1).unwrap(),
        expires: "2999-01-01T00:00:00Z".parse().unwrap(),
        keys: HashMap::from([(key_id, key)]),
        roles,
        _extra: HashMap::new(),
    };

    let encoded = serde_json::to_vec(&root)?;
    let reparsed: Root = serde_json::from_slice(&encoded)?;
    assert_eq!(reparsed.keys.len(), 1);

    Ok(())
}

#[tokio::test(flavor = "current_thread")]
async fn root_retains_conventional_ecdsa_tuf_id_threshold_identity(
) -> Result<(), Box<dyn Error + Send + Sync + 'static>> {
    let signer = generate_ecdsa_signer()?;
    let key = signer.tuf_key();
    let alias = with_ecdsa_compatibility_extension(key.clone());
    let key_id = key.key_id()?;
    let alias_id = alias.key_id()?;
    assert_ne!(key_id, alias_id);

    let root_keys = RoleKeys {
        keyids: vec![key_id.clone(), alias_id.clone()],
        threshold: NonZeroU64::new(2).unwrap(),
        _extra: HashMap::new(),
    };
    let mut roles = HashMap::new();
    roles.insert(RoleType::Root, root_keys);
    roles.insert(RoleType::Targets, role_keys(key_id.clone()));
    roles.insert(RoleType::Snapshot, role_keys(key_id.clone()));
    roles.insert(RoleType::Timestamp, role_keys(key_id.clone()));
    let root = Root {
        spec_version: "1.0.36".to_owned(),
        consistent_snapshot: true,
        version: NonZeroU64::new(1).unwrap(),
        expires: "2999-01-01T00:00:00Z".parse().unwrap(),
        keys: HashMap::from([(key_id.clone(), key), (alias_id.clone(), alias)]),
        roles,
        _extra: HashMap::new(),
    };
    let canonical = root.canonical_form()?;
    let rng = SystemRandom::new();
    let signatures = vec![
        Signature {
            keyid: key_id,
            sig: Sign::sign(&signer, &canonical, &rng).await?.into(),
        },
        Signature {
            keyid: alias_id,
            sig: Sign::sign(&signer, &canonical, &rng).await?.into(),
        },
    ];

    let one_signature = Signed {
        signed: root.clone(),
        signatures: vec![signatures[0].clone()],
    };
    root.verify_role(&one_signature)
        .expect_err("one conventional TUF key ID must not satisfy threshold 2");

    let encoded = serde_json::to_vec(&Signed {
        signed: root,
        signatures,
    })?;
    let reparsed: Signed<Root> = serde_json::from_slice(&encoded)?;
    reparsed.signed.verify_role(&reparsed)?;

    let mut duplicate_key_id = reparsed.clone();
    duplicate_key_id
        .signatures
        .push(duplicate_key_id.signatures[0].clone());
    duplicate_key_id
        .signed
        .verify_role(&duplicate_key_id)
        .expect_err("root verification must reject a repeated signature key ID");

    Ok(())
}

#[tokio::test(flavor = "current_thread")]
async fn delegations_retain_conventional_ecdsa_tuf_id_threshold_identity(
) -> Result<(), Box<dyn Error + Send + Sync + 'static>> {
    let signer = generate_ecdsa_signer()?;
    let key = signer.tuf_key();
    let alias = with_ecdsa_compatibility_extension(key.clone());
    let key_id = key.key_id()?;
    let alias_id = alias.key_id()?;
    assert_ne!(key_id, alias_id);

    let role_name = "delegated";
    let delegations = Delegations {
        keys: HashMap::from([(key_id.clone(), key), (alias_id.clone(), alias)]),
        roles: vec![DelegatedRole {
            name: role_name.to_owned(),
            keyids: vec![key_id.clone(), alias_id.clone()],
            threshold: NonZeroU64::new(2).unwrap(),
            paths: PathSet::Paths(vec![PathPattern::new("*")?]),
            terminating: false,
            targets: None,
        }],
    };
    let encoded = serde_json::to_vec(&delegations)?;
    let reparsed: Delegations = serde_json::from_slice(&encoded)?;
    let targets = Targets {
        spec_version: "1.0.36".to_owned(),
        version: NonZeroU64::new(1).unwrap(),
        expires: "2999-01-01T00:00:00Z".parse().unwrap(),
        targets: HashMap::new(),
        delegations: None,
        _extra: HashMap::new(),
    };
    let canonical = targets.canonical_form()?;
    let rng = SystemRandom::new();
    let signatures = vec![
        Signature {
            keyid: key_id,
            sig: Sign::sign(&signer, &canonical, &rng).await?.into(),
        },
        Signature {
            keyid: alias_id,
            sig: Sign::sign(&signer, &canonical, &rng).await?.into(),
        },
    ];

    let one_signature = Signed {
        signed: targets.clone(),
        signatures: vec![signatures[0].clone()],
    };
    reparsed
        .verify_role(&one_signature, role_name)
        .expect_err("one conventional TUF key ID must not satisfy delegated threshold 2");

    let signed = Signed {
        signed: targets,
        signatures,
    };
    reparsed.verify_role(&signed, role_name)?;

    let mut duplicate_key_id = signed;
    duplicate_key_id
        .signatures
        .push(duplicate_key_id.signatures[0].clone());
    reparsed
        .verify_role(&duplicate_key_id, role_name)
        .expect_err("delegation verification must reject a repeated signature key ID");

    Ok(())
}

#[tokio::test(flavor = "current_thread")]
async fn root_rejects_unused_undefined_openpgp_extensions(
) -> Result<(), Box<dyn Error + Send + Sync + 'static>> {
    let signer = CompositeSigner::generate()?;
    let clean_key = signer.tuf_key();
    let clean_key_id = clean_key.key_id()?;
    let unused_key = CompositeSigner::generate()?.tuf_key();
    let invalid_keys = [
        with_undefined_openpgp_extension(unused_key.clone()),
        with_undefined_openpgp_keyval_extension(unused_key),
    ];

    for invalid_key in invalid_keys {
        let invalid_key_id = invalid_key.key_id()?;
        assert_ne!(clean_key_id, invalid_key_id);
        let root_keys = role_keys(clean_key_id.clone());
        let mut roles = HashMap::new();
        roles.insert(RoleType::Root, root_keys.clone());
        roles.insert(RoleType::Targets, root_keys.clone());
        roles.insert(RoleType::Snapshot, root_keys.clone());
        roles.insert(RoleType::Timestamp, root_keys);
        let root = Root {
            spec_version: "1.0.36".to_owned(),
            consistent_snapshot: true,
            version: NonZeroU64::new(1).unwrap(),
            expires: "2999-01-01T00:00:00Z".parse().unwrap(),
            keys: HashMap::from([
                (clean_key_id.clone(), clean_key.clone()),
                (invalid_key_id, invalid_key),
            ]),
            roles,
            _extra: HashMap::new(),
        };
        let key_sources: Vec<Box<dyn KeySource>> = vec![Box::new(signer.clone())];
        let signed = SignedRole::new(
            root.clone(),
            &KeyHolder::Root(root),
            &key_sources,
            &SystemRandom::new(),
        )
        .await?;
        signed.signed().signed.verify_role(signed.signed())?;

        let encoded = serde_json::to_vec(signed.signed())?;
        assert!(
            serde_json::from_slice::<Signed<Root>>(&encoded).is_err(),
            "root metadata must reject an unused provisional OpenPGP key with undefined fields"
        );
    }

    Ok(())
}

#[tokio::test(flavor = "current_thread")]
async fn delegations_reject_unused_undefined_openpgp_extensions(
) -> Result<(), Box<dyn Error + Send + Sync + 'static>> {
    let signer = CompositeSigner::generate()?;
    let clean_key = signer.tuf_key();
    let clean_key_id = clean_key.key_id()?;
    let unused_key = CompositeSigner::generate()?.tuf_key();
    let invalid_keys = [
        with_undefined_openpgp_extension(unused_key.clone()),
        with_undefined_openpgp_keyval_extension(unused_key),
    ];

    for invalid_key in invalid_keys {
        let invalid_key_id = invalid_key.key_id()?;
        assert_ne!(clean_key_id, invalid_key_id);
        let role_name = "delegated";
        let delegations = Delegations {
            keys: HashMap::from([
                (clean_key_id.clone(), clean_key.clone()),
                (invalid_key_id, invalid_key),
            ]),
            roles: vec![DelegatedRole {
                name: role_name.to_owned(),
                keyids: vec![clean_key_id.clone()],
                threshold: NonZeroU64::new(1).unwrap(),
                paths: PathSet::Paths(vec![PathPattern::new("*")?]),
                terminating: false,
                targets: None,
            }],
        };
        let delegated_targets = Targets {
            spec_version: "1.0.36".to_owned(),
            version: NonZeroU64::new(1).unwrap(),
            expires: "2999-01-01T00:00:00Z".parse().unwrap(),
            targets: HashMap::new(),
            delegations: None,
            _extra: HashMap::new(),
        };
        let key_sources: Vec<Box<dyn KeySource>> = vec![Box::new(signer.clone())];
        let signed = SignedRole::new(
            DelegatedTargets {
                name: role_name.to_owned(),
                targets: delegated_targets,
            },
            &KeyHolder::Delegations(delegations.clone()),
            &key_sources,
            &SystemRandom::new(),
        )
        .await?;
        let signed_targets = signed.signed().clone().targets().1;
        delegations.verify_role(&signed_targets, role_name)?;

        let containing_targets = Targets {
            spec_version: "1.0.36".to_owned(),
            version: NonZeroU64::new(1).unwrap(),
            expires: "2999-01-01T00:00:00Z".parse().unwrap(),
            targets: HashMap::new(),
            delegations: Some(delegations),
            _extra: HashMap::new(),
        };
        let encoded = serde_json::to_vec(&containing_targets)?;
        assert!(
            serde_json::from_slice::<Targets>(&encoded).is_err(),
            "delegations metadata must reject an unused provisional OpenPGP key with undefined fields"
        );
    }

    Ok(())
}

#[tokio::test(flavor = "current_thread")]
async fn root_rejects_distinct_tuf_ids_for_one_composite_signing_key() -> openpgp::Result<()> {
    let profile = parse_experimental_profile(EXPERIMENTAL_PROFILE_JSON.as_bytes())?;
    assert_eq!(
        profile.threshold_identity,
        ThresholdIdentity::VerifiedV6OpenPgpSigningKeyFingerprint
    );
    let signer = CompositeSigner::generate()?;
    let key = signer.tuf_key();
    let alias = openpgp_key(alternate_public_projection(&signer.cert)?);
    let key_id = key.key_id()?;
    let alias_id = alias.key_id()?;
    assert_ne!(key_id, alias_id);

    let root_keys = RoleKeys {
        keyids: vec![key_id.clone(), alias_id.clone()],
        threshold: NonZeroU64::new(2).unwrap(),
        _extra: HashMap::new(),
    };
    let mut roles = HashMap::new();
    roles.insert(RoleType::Root, root_keys);
    roles.insert(RoleType::Targets, role_keys(key_id.clone()));
    roles.insert(RoleType::Snapshot, role_keys(key_id.clone()));
    roles.insert(RoleType::Timestamp, role_keys(key_id.clone()));
    let root = Root {
        spec_version: "1.0.36".to_owned(),
        consistent_snapshot: true,
        version: NonZeroU64::new(1).unwrap(),
        expires: "2999-01-01T00:00:00Z".parse().unwrap(),
        keys: HashMap::from([(key_id.clone(), key), (alias_id.clone(), alias)]),
        roles,
        _extra: HashMap::new(),
    };
    let key_sources: Vec<Box<dyn KeySource>> = vec![Box::new(signer)];
    let signed = SignedRole::new(
        root.clone(),
        &KeyHolder::Root(root),
        &key_sources,
        &SystemRandom::new(),
    )
    .await?;
    assert_eq!(signed.signed().signatures.len(), 1);

    let mut counterexample = signed.signed().clone();
    let mut relabeled = counterexample.signatures[0].clone();
    relabeled.keyid = alias_id;
    counterexample.signatures.push(relabeled);

    let encoded = serde_json::to_vec(&counterexample)?;
    let reparsed: Signed<Root> = serde_json::from_slice(&encoded)?;
    reparsed
        .signed
        .verify_role(&reparsed)
        .expect_err("one composite signing key must not satisfy threshold 2");

    Ok(())
}

#[tokio::test(flavor = "current_thread")]
async fn delegations_reject_distinct_tuf_ids_for_one_composite_signing_key() -> openpgp::Result<()>
{
    let signer = CompositeSigner::generate()?;
    let key = signer.tuf_key();
    let alias = openpgp_key(alternate_public_projection(&signer.cert)?);
    let key_id = key.key_id()?;
    let alias_id = alias.key_id()?;
    assert_ne!(key_id, alias_id);

    let role_name = "delegated";
    let mut delegations = Delegations {
        keys: HashMap::from([(key_id.clone(), key), (alias_id.clone(), alias)]),
        roles: vec![DelegatedRole {
            name: role_name.to_owned(),
            keyids: vec![key_id, alias_id.clone()],
            threshold: NonZeroU64::new(1).unwrap(),
            paths: PathSet::Paths(vec![PathPattern::new("*")?]),
            terminating: false,
            targets: None,
        }],
    };
    let targets = Targets {
        spec_version: "1.0.36".to_owned(),
        version: NonZeroU64::new(1).unwrap(),
        expires: "2999-01-01T00:00:00Z".parse().unwrap(),
        targets: HashMap::new(),
        delegations: None,
        _extra: HashMap::new(),
    };
    let key_sources: Vec<Box<dyn KeySource>> = vec![Box::new(signer)];
    let signed = SignedRole::new(
        DelegatedTargets {
            name: role_name.to_owned(),
            targets,
        },
        &KeyHolder::Delegations(delegations.clone()),
        &key_sources,
        &SystemRandom::new(),
    )
    .await?;
    assert_eq!(signed.signed().signatures.len(), 1);

    delegations.roles[0].threshold = NonZeroU64::new(2).unwrap();
    let encoded_delegations = serde_json::to_vec(&delegations)?;
    let reparsed: Delegations = serde_json::from_slice(&encoded_delegations)?;
    let mut counterexample = signed.signed().clone().targets().1;
    let mut relabeled = counterexample.signatures[0].clone();
    relabeled.keyid = alias_id;
    counterexample.signatures.push(relabeled);
    reparsed
        .verify_role(&counterexample, role_name)
        .expect_err("one composite signing key must not satisfy delegated threshold 2");

    Ok(())
}

#[tokio::test(flavor = "current_thread")]
async fn publisher_and_verifier_require_both_rfc9980_components() -> openpgp::Result<()> {
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
        expires: "2999-01-01T00:00:00Z".parse().unwrap(),
        keys: HashMap::from([(key_id.clone(), key)]),
        roles,
        _extra: HashMap::new(),
    };
    let canonical_signed_bytes = root.canonical_form()?;
    let key_sources: Vec<Box<dyn KeySource>> = vec![Box::new(signer.clone())];

    let signed = SignedRole::new(
        root.clone(),
        &KeyHolder::Root(root.clone()),
        &key_sources,
        &SystemRandom::new(),
    )
    .await?;

    assert_eq!(
        signer.signed_messages.lock().unwrap().as_slice(),
        &[canonical_signed_bytes]
    );
    root.verify_role(signed.signed())?;

    let mut duplicate_signature = signed.signed().clone();
    duplicate_signature
        .signatures
        .push(duplicate_signature.signatures[0].clone());
    let mut threshold_two_root = root.clone();
    threshold_two_root
        .roles
        .get_mut(&RoleType::Root)
        .unwrap()
        .threshold = NonZeroU64::new(2).unwrap();
    assert!(threshold_two_root
        .verify_role(&duplicate_signature)
        .is_err());

    let compact_outer = serde_json::to_vec(signed.signed())?;
    let reparsed: Signed<Root> = serde_json::from_slice(&compact_outer)?;
    root.verify_role(&reparsed)?;

    let signature_packet = signed.signed().signatures[0].sig.to_vec();
    let packets: Vec<Packet> = PacketPile::from_bytes(&signature_packet)?
        .into_children()
        .collect();
    assert_eq!(packets.len(), 1, "the TUF signature is exactly one packet");
    let Packet::Signature(signature) = &packets[0] else {
        panic!("the only packet must be a detached signature");
    };
    assert_eq!(signature.version(), 6);
    assert_eq!(signature.typ(), SignatureType::Binary);
    assert_eq!(
        signature.pk_algo(),
        openpgp::types::PublicKeyAlgorithm::MLDSA65_Ed25519
    );
    assert_eq!(signature.hash_algo(), HashAlgorithm::SHA512);
    assert_eq!(signature.issuer_fingerprints().count(), 1);

    let mut bad_ed25519 = signed.signed().clone();
    bad_ed25519.signatures[0].sig = corrupt_component(
        &signature_packet,
        ED25519_SIGNATURE_BYTES + ML_DSA_65_SIGNATURE_BYTES,
    )
    .into();
    assert!(root.verify_role(&bad_ed25519).is_err());

    let mut bad_ml_dsa = signed.signed().clone();
    bad_ml_dsa.signatures[0].sig =
        corrupt_component(&signature_packet, ML_DSA_65_SIGNATURE_BYTES).into();
    assert!(root.verify_role(&bad_ml_dsa).is_err());

    println!(
        "{}",
        serde_json::json!({
            "canonicalSignedBytes": signer.signed_messages.lock().unwrap()[0].len(),
            "compositeSigningKeyThresholdContributions": 1,
            "conventionalKeyExtraFieldCompatibilityRetained": true,
            "delegatedDistinctTufIdsForOneCompositeSigningKeyRejected": true,
            "distinctTufIdsForOneCompositeSigningKeyRejected": true,
            "duplicateCompositeSignatureRejected": true,
            "ed25519ComponentCorruptionRejected": true,
            "hash": "SHA-512",
            "issuerFingerprint": signature.issuer_fingerprints().next().unwrap().to_string(),
            "keyId": serde_json::to_value(&key_id)?,
            "keyType": "openpgp-rfc9580",
            "mlDsa65ComponentCorruptionRejected": true,
            "outerReformatVerified": true,
            "signaturePacketAlgorithm": 30,
            "publicProjectionBytes": signer.public_projection.len(),
            "scheme": "openpgp-rfc9980-ml-dsa-65+ed25519-sha512",
            "signaturePacketBytes": signature_packet.len(),
            "signaturePacketCount": 1,
            "signaturePacketType": 0,
            "signaturePacketVersion": 6,
            "syntheticIndependence": "one generated composite key; no operator, custody, or underlying-key independence claim",
            "thresholdIdentity": "verified-v6-openpgp-signing-key-fingerprint",
            "thresholdTwoSatisfiedByOneCompositeKey": false,
            "undefinedOpenPgpExtensionsRejected": true,
            "unusedUndefinedOpenPgpKeysRejectedBeforeMetadataAcceptance": true,
            "verified": true
        })
    );

    Ok(())
}

fn assert_top_level_role_verified<T>(root: &Root, signed: &Signed<T>)
where
    T: Role + Clone,
{
    root.verify_role(signed).unwrap();

    for from_end in [
        1,
        ED25519_SIGNATURE_BYTES + ML_DSA_65_SIGNATURE_BYTES,
        ML_DSA_65_SIGNATURE_BYTES,
    ] {
        let mut corrupted = signed.clone();
        corrupted.signatures[0].sig = corrupt_component(&signed.signatures[0].sig, from_end).into();
        assert!(root.verify_role(&corrupted).is_err());
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

fn assert_metadata_descriptor(rule: &DescriptorRules, descriptor: &Metafile, bytes: &[u8]) {
    assert_eq!(rule.length, DescriptorRequirement::Required);
    assert_eq!(rule.hashes, [DescriptorHash::Sha256]);
    assert_eq!(descriptor.length, Some(bytes.len() as u64));
    assert_eq!(
        descriptor.hashes.as_ref().unwrap().sha256.as_ref(),
        digest(&SHA256, bytes).as_ref()
    );

    let wire = serde_json::to_value(descriptor).unwrap();
    let fields = wire.as_object().unwrap();
    assert_eq!(fields.len(), 3);
    assert!(fields.contains_key("length"));
    assert!(fields.contains_key("version"));
    let hashes = fields["hashes"].as_object().unwrap();
    assert_eq!(hashes.len(), 1);
    assert!(hashes.contains_key("sha256"));
}

fn assert_profile_spec_version<T>(expected: TufSpecVersion, role: &T)
where
    T: serde::Serialize,
{
    assert_eq!(
        serde_json::to_value(role).unwrap()["spec_version"],
        serde_json::to_value(expected).unwrap()
    );
}

#[test]
fn experimental_profile_rejects_malformed_or_ambiguous_input() {
    let profile = parse_experimental_profile(EXPERIMENTAL_PROFILE_JSON.as_bytes()).unwrap();
    assert_eq!(
        serde_json::to_string(&profile).unwrap(),
        EXPERIMENTAL_PROFILE_JSON,
        "the accepted typed fixture must serialize to the exact deterministic bytes"
    );

    let malformed = [
        EXPERIMENTAL_PROFILE_JSON.replace("1.0.36", "1.0.35"),
        EXPERIMENTAL_PROFILE_JSON.replace(
            "\"lifecycle_fog\":[\"accepted_time\",\"consistent_snapshot\",\"expiry\",\"root_bootstrap\",\"root_rotation\",\"rollback\"]",
            "\"lifecycle_fog\":null",
        ),
        EXPERIMENTAL_PROFILE_JSON.replace(
            "\"lifecycle_fog\":[\"accepted_time\",\"consistent_snapshot\",\"expiry\",\"root_bootstrap\",\"root_rotation\",\"rollback\"]",
            "\"lifecycle_fog\":[]",
        ),
        EXPERIMENTAL_PROFILE_JSON.replace(
            "{\"canonicalization\"",
            "{\"tuf_spec_version\":\"1.0.36\",\"canonicalization\"",
        ),
        EXPERIMENTAL_PROFILE_JSON.replace(
            "\"certificate_primary_key_version\":6",
            "\"certificate_primary_key_version\":6,\"undefined\":true",
        ),
        EXPERIMENTAL_PROFILE_JSON.replace(
            ",\"threshold_identity\":\"verified-v6-openpgp-signing-key-fingerprint\"",
            "",
        ),
    ];

    for bytes in malformed {
        assert!(parse_experimental_profile(bytes.as_bytes()).is_err());
    }
}

#[test]
fn experimental_profile_requires_canonical_openpgp_encodings() {
    let accepted: serde_json::Value = serde_json::from_str(EXPERIMENTAL_PROFILE_JSON).unwrap();
    assert!(parse_experimental_profile(&serde_json::to_vec(&accepted).unwrap()).is_ok());

    for (field, malformed) in [
        ("public_certificate_encoding", "ascii-armored-certificate"),
        ("detached_signature_encoding", "ascii-armored-signature"),
    ] {
        let mut rejected = accepted.clone();
        rejected["openpgp"][field] = serde_json::json!(malformed);
        assert!(parse_experimental_profile(&serde_json::to_vec(&rejected).unwrap()).is_err());
    }
}

#[test]
fn experimental_profile_requires_one_signing_key_and_signature_packet() {
    let accepted: serde_json::Value = serde_json::from_str(EXPERIMENTAL_PROFILE_JSON).unwrap();
    assert!(parse_experimental_profile(&serde_json::to_vec(&accepted).unwrap()).is_ok());

    for (field, malformed) in [
        ("eligible_signing_keys", 0),
        ("eligible_signing_keys", 2),
        ("signature_packets", 0),
        ("signature_packets", 2),
    ] {
        let mut rejected = accepted.clone();
        rejected["openpgp"][field] = serde_json::json!(malformed);
        assert!(parse_experimental_profile(&serde_json::to_vec(&rejected).unwrap()).is_err());
    }
}

#[test]
fn experimental_profile_scopes_openpgp_versions_and_algorithms() {
    let accepted: serde_json::Value = serde_json::from_str(EXPERIMENTAL_PROFILE_JSON).unwrap();
    assert!(parse_experimental_profile(&serde_json::to_vec(&accepted).unwrap()).is_ok());

    for (field, malformed) in [
        ("certificate_primary_key_version", 4),
        ("certificate_primary_key_algorithm", 22),
        ("eligible_signing_key_version", 4),
        ("eligible_signing_key_algorithm", 22),
        ("signature_packet_version", 4),
        ("signature_packet_algorithm", 22),
    ] {
        let mut rejected = accepted.clone();
        rejected["openpgp"][field] = serde_json::json!(malformed);
        assert!(parse_experimental_profile(&serde_json::to_vec(&rejected).unwrap()).is_err());
    }
}

#[test]
fn experimental_profile_requires_each_lifecycle_question_once() {
    let invalid_profiles = [
        EXPERIMENTAL_PROFILE_JSON.replace(",\"root_rotation\"", ""),
        EXPERIMENTAL_PROFILE_JSON
            .replace("\"root_rotation\"", "\"root_rotation\",\"root_rotation\""),
    ];

    for invalid_profile in invalid_profiles {
        assert_ne!(invalid_profile, EXPERIMENTAL_PROFILE_JSON);
        assert!(parse_experimental_profile(invalid_profile.as_bytes()).is_err());
    }
}

#[tokio::test(flavor = "current_thread")]
async fn all_top_level_roles_use_the_composite_profile() -> openpgp::Result<()> {
    let profile = parse_experimental_profile(EXPERIMENTAL_PROFILE_JSON.as_bytes())?;
    assert_eq!(profile.canonicalization, Canonicalization::TufCanonicalJson);
    assert_eq!(profile.tuf_spec_version, TufSpecVersion::V1_0_36);
    assert_eq!(
        profile.extensions.target_custom,
        TargetCustomHandling::PreserveOpaque
    );
    let signer = CompositeSigner::generate()?;
    assert_eq!(
        profile.openpgp.public_certificate_encoding,
        PublicCertificateEncoding::CanonicalUnarmoredPublicCertificate
    );
    let parsed_public_projection = Cert::from_bytes(&signer.public_projection)?;
    assert!(!parsed_public_projection.is_tsk());
    assert_eq!(
        parsed_public_projection.clone().to_vec()?,
        signer.public_projection
    );
    let key = signer.tuf_key();
    let key_wire = serde_json::to_value(&key)?;
    assert_eq!(
        key_wire["keytype"],
        serde_json::to_value(profile.openpgp.key_type)?
    );
    assert_eq!(profile.openpgp.key_type, OpenPgpKeyType::OpenPgpRfc9580);
    assert_eq!(
        key_wire["scheme"],
        serde_json::to_value(profile.openpgp.signature_scheme)?
    );
    assert_eq!(
        profile.openpgp.signature_scheme,
        SignatureScheme::OpenPgpRfc9980MlDsa65Ed25519Sha512
    );
    let primary_key = signer.cert.primary_key();
    assert_eq!(
        primary_key.key().version(),
        profile.openpgp.certificate_primary_key_version
    );
    assert_eq!(
        u8::from(primary_key.key().pk_algo()),
        profile.openpgp.certificate_primary_key_algorithm
    );
    let policy = StandardPolicy::new();
    let eligible_signing_keys: Vec<_> = signer
        .cert
        .keys()
        .with_policy(&policy, None)
        .supported()
        .alive()
        .revoked(false)
        .for_signing()
        .collect();
    assert_eq!(
        eligible_signing_keys.len(),
        profile.openpgp.eligible_signing_keys
    );
    assert_eq!(
        eligible_signing_keys[0].key().version(),
        profile.openpgp.eligible_signing_key_version
    );
    assert_eq!(
        u8::from(eligible_signing_keys[0].key().pk_algo()),
        profile.openpgp.eligible_signing_key_algorithm
    );
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
        expires: "2999-01-01T00:00:00Z".parse().unwrap(),
        keys: HashMap::from([(key_id, key)]),
        roles,
        _extra: HashMap::new(),
    };
    assert_profile_spec_version(profile.tuf_spec_version, &root);
    let reparsed_root: Root = serde_json::from_slice(&serde_json::to_vec(&root)?)?;
    assert_profile_spec_version(profile.tuf_spec_version, &reparsed_root);
    let key_sources: Vec<Box<dyn KeySource>> = vec![Box::new(signer.clone())];
    let rng = SystemRandom::new();

    let role_name = "delegated";
    let delegations = Delegations {
        keys: root.keys.clone(),
        roles: vec![DelegatedRole {
            name: role_name.to_owned(),
            keyids: root.keys.keys().cloned().collect(),
            threshold: NonZeroU64::new(1).unwrap(),
            paths: PathSet::Paths(vec![PathPattern::new("*")?]),
            terminating: false,
            targets: None,
        }],
    };
    let artifact = b"codiquary issue 20 fixture\n";
    let artifact_target = Target {
        length: artifact.len() as u64,
        hashes: Hashes {
            sha256: digest(&SHA256, artifact).as_ref().to_vec().into(),
            _extra: HashMap::new(),
        },
        custom: HashMap::from([("experimental-profile".to_owned(), serde_json::json!(true))]),
        _extra: HashMap::new(),
    };
    assert_eq!(
        serde_json::to_value(&artifact_target)?,
        serde_json::json!({
            "custom": {"experimental-profile": true},
            "hashes": {
                "sha256": "d13ecc865c37b23650615038c232eccfa5770c8bc345dbe98db595273fecda4a"
            },
            "length": 27,
        }),
        "target descriptors must use the exact length, hashes.sha256, and custom wire labels"
    );
    assert_eq!(
        profile.descriptors.target.length,
        DescriptorRequirement::Required
    );
    assert_eq!(profile.descriptors.target.hashes, [DescriptorHash::Sha256]);
    let top_targets = Targets {
        spec_version: "1.0.36".to_owned(),
        version: NonZeroU64::new(1).unwrap(),
        expires: "2999-01-01T00:00:00Z".parse().unwrap(),
        targets: HashMap::new(),
        delegations: Some(delegations.clone()),
        _extra: HashMap::new(),
    };
    assert_profile_spec_version(profile.tuf_spec_version, &top_targets);

    let signed_targets = SignedRole::new(
        top_targets.clone(),
        &KeyHolder::Root(root.clone()),
        &key_sources,
        &rng,
    )
    .await?;
    assert_top_level_role_verified(&root, signed_targets.signed());
    assert_eq!(
        profile.openpgp.detached_signature_encoding,
        DetachedSignatureEncoding::CanonicalUnarmoredDetachedSignature
    );
    let target_signature_bytes = signed_targets.signed().signatures[0].sig.to_vec();
    let target_signature_packets: Vec<_> = PacketPile::from_bytes(&target_signature_bytes)?
        .into_children()
        .collect();
    assert_eq!(
        target_signature_packets.len(),
        profile.openpgp.signature_packets
    );
    let [Packet::Signature(target_signature)] = target_signature_packets.as_slice() else {
        panic!("the targets role must carry one OpenPGP signature packet");
    };
    assert_eq!(
        Packet::from(target_signature.clone()).to_vec()?,
        target_signature_bytes
    );
    assert_eq!(
        target_signature.version(),
        profile.openpgp.signature_packet_version
    );
    assert_eq!(
        u8::from(target_signature.typ()),
        profile.openpgp.signature_type
    );
    assert_eq!(
        u8::from(target_signature.pk_algo()),
        profile.openpgp.signature_packet_algorithm
    );
    assert_eq!(target_signature.hash_algo(), HashAlgorithm::SHA512);
    assert_eq!(profile.openpgp.signature_hash, SignatureHash::Sha512);
    assert_eq!(
        target_signature.issuer_fingerprints().count(),
        profile.openpgp.issuer_fingerprints
    );
    assert_eq!(
        target_signature
            .issuer_fingerprints()
            .next()
            .unwrap()
            .clone(),
        eligible_signing_keys[0].key().fingerprint()
    );
    assert_eq!(
        profile.openpgp.signature_components,
        [SignatureComponent::Ed25519, SignatureComponent::MlDsa65]
    );
    assert_ne!(
        serde_json::to_value(profile.openpgp.signature_hash)?,
        serde_json::to_value(profile.descriptors.metadata.hashes[0])?,
        "the SHA-512 signature digest and SHA-256 descriptor digest are distinct layers"
    );

    let delegated_targets = DelegatedTargets {
        name: role_name.to_owned(),
        targets: Targets {
            spec_version: "1.0.36".to_owned(),
            version: NonZeroU64::new(1).unwrap(),
            expires: "2999-01-01T00:00:00Z".parse().unwrap(),
            targets: HashMap::from([(TargetName::new("artifact.bin")?, artifact_target)]),
            delegations: None,
            _extra: HashMap::new(),
        },
    };
    assert_profile_spec_version(profile.tuf_spec_version, &delegated_targets.targets);
    let signed_delegated = SignedRole::new(
        delegated_targets.clone(),
        &KeyHolder::Delegations(delegations.clone()),
        &key_sources,
        &rng,
    )
    .await?;
    let delegated_targets_bytes = signed_delegated.buffer().clone();
    let reparsed_delegated: Signed<DelegatedTargets> =
        serde_json::from_slice(&delegated_targets_bytes)?;
    let (_, reparsed_role) = reparsed_delegated.targets();
    assert_profile_spec_version(profile.tuf_spec_version, &reparsed_role.signed);
    delegations.verify_role(&reparsed_role, role_name)?;
    let delegated_signed = signed_delegated.signed().clone().targets().1;
    delegations.verify_role(&delegated_signed, role_name)?;
    for from_end in [
        1,
        ED25519_SIGNATURE_BYTES + ML_DSA_65_SIGNATURE_BYTES,
        ML_DSA_65_SIGNATURE_BYTES,
    ] {
        let mut corrupted_delegated = delegated_signed.clone();
        corrupted_delegated.signatures[0].sig =
            corrupt_component(&delegated_signed.signatures[0].sig, from_end).into();
        delegations
            .verify_role(&corrupted_delegated, role_name)
            .expect_err("delegated targets must reject a corrupted composite signature");
    }

    let signed_targets_bytes = signed_targets.buffer().clone();
    let reparsed_targets: Signed<Targets> = serde_json::from_slice(&signed_targets_bytes)?;
    assert_profile_spec_version(profile.tuf_spec_version, &reparsed_targets.signed);
    root.verify_role(&reparsed_targets)?;
    let targets_metafile = metafile(&signed_targets_bytes);
    assert_metadata_descriptor(
        &profile.descriptors.metadata,
        &targets_metafile,
        &signed_targets_bytes,
    );
    let delegated_metafile = metafile(&delegated_targets_bytes);
    assert_metadata_descriptor(
        &profile.descriptors.metadata,
        &delegated_metafile,
        &delegated_targets_bytes,
    );
    let snapshot = Snapshot {
        spec_version: "1.0.36".to_owned(),
        version: NonZeroU64::new(1).unwrap(),
        expires: "2999-01-01T00:00:00Z".parse().unwrap(),
        meta: HashMap::from([
            ("targets.json".to_owned(), targets_metafile),
            ("delegated.json".to_owned(), delegated_metafile),
        ]),
        _extra: HashMap::new(),
    };
    assert_profile_spec_version(profile.tuf_spec_version, &snapshot);
    let signed_snapshot = SignedRole::new(
        snapshot.clone(),
        &KeyHolder::Root(root.clone()),
        &key_sources,
        &rng,
    )
    .await?;
    assert_top_level_role_verified(&root, signed_snapshot.signed());
    let snapshot_bytes = signed_snapshot.buffer().clone();
    let reparsed_snapshot: Signed<Snapshot> = serde_json::from_slice(&snapshot_bytes)?;
    assert_profile_spec_version(profile.tuf_spec_version, &reparsed_snapshot.signed);
    root.verify_role(&reparsed_snapshot)?;

    let mut timestamp = Timestamp::new(
        "1.0.36".to_owned(),
        NonZeroU64::new(1).unwrap(),
        "2999-01-01T00:00:00Z".parse().unwrap(),
    );
    let snapshot_metafile = metafile(&snapshot_bytes);
    assert_metadata_descriptor(
        &profile.descriptors.metadata,
        &snapshot_metafile,
        &snapshot_bytes,
    );
    timestamp
        .meta
        .insert("snapshot.json".to_owned(), snapshot_metafile);
    assert_profile_spec_version(profile.tuf_spec_version, &timestamp);
    let signed_timestamp = SignedRole::new(
        timestamp.clone(),
        &KeyHolder::Root(root.clone()),
        &key_sources,
        &rng,
    )
    .await?;
    assert_top_level_role_verified(&root, signed_timestamp.signed());
    let reparsed_timestamp: Signed<Timestamp> = serde_json::from_slice(signed_timestamp.buffer())?;
    assert_profile_spec_version(profile.tuf_spec_version, &reparsed_timestamp.signed);
    root.verify_role(&reparsed_timestamp)?;

    let mut linked_delegations = delegations;
    linked_delegations.roles[0].targets = Some(reparsed_role);
    let mut linked_targets = signed_targets.signed().clone();
    linked_targets.signed.delegations = Some(linked_delegations);
    let found_target = linked_targets
        .signed
        .find_target(&TargetName::new("artifact.bin")?, false)?;
    assert_eq!(found_target.length, artifact.len() as u64);
    assert_eq!(
        found_target.custom.get("experimental-profile"),
        Some(&serde_json::json!(true)),
        "opaque target custom data must survive serialization and delegated traversal"
    );

    let expected_canonical = vec![
        top_targets.canonical_form()?,
        delegated_targets.canonical_form()?,
        snapshot.canonical_form()?,
        timestamp.canonical_form()?,
    ];
    assert_eq!(
        signer.signed_messages.lock().unwrap().as_slice(),
        expected_canonical.as_slice(),
        "each role must be signed over its canonical TUF bytes"
    );

    Ok(())
}
