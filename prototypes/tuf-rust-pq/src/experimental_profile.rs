//! Closed, synthetic profile fixture for the behavior exercised by this prototype.

use serde::{Deserialize, Serialize};
use std::error::Error;
use std::fmt;

pub const EXPERIMENTAL_PROFILE_JSON: &str = r#"{"canonicalization":"tuf-canonical-json","descriptors":{"metadata":{"hashes":["sha256"],"length":"required"},"target":{"hashes":["sha256"],"length":"required"}},"extensions":{"openpgp_key":"reject-undefined","openpgp_keyval":"reject-undefined","target_custom":"preserve-opaque"},"lifecycle_fog":["accepted_time","consistent_snapshot","expiry","root_bootstrap","rollback"],"openpgp":{"certificate_version":6,"issuer_fingerprints":1,"key_type":"openpgp-rfc9580","public_key_algorithm":30,"signature_components":["ed25519","ml-dsa-65"],"signature_hash":"sha512","signature_scheme":"openpgp-rfc9980-ml-dsa-65+ed25519-sha512","signature_type":0,"signature_version":6},"threshold_identity":"verified-v6-openpgp-signing-key-fingerprint","tuf_spec_version":"1.0.36"}"#;

#[derive(Debug, Clone, Deserialize, Serialize, Eq, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct ExperimentalProfile {
    pub canonicalization: Canonicalization,
    pub descriptors: Descriptors,
    pub extensions: Extensions,
    pub lifecycle_fog: [LifecycleQuestion; 5],
    pub openpgp: OpenPgpProfile,
    pub threshold_identity: ThresholdIdentity,
    pub tuf_spec_version: TufSpecVersion,
}

#[derive(Debug, Clone, Copy, Deserialize, Serialize, Eq, PartialEq)]
pub enum Canonicalization {
    #[serde(rename = "tuf-canonical-json")]
    TufCanonicalJson,
}

#[derive(Debug, Clone, Deserialize, Serialize, Eq, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct Descriptors {
    pub metadata: DescriptorRules,
    pub target: DescriptorRules,
}

#[derive(Debug, Clone, Deserialize, Serialize, Eq, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct DescriptorRules {
    pub hashes: [DescriptorHash; 1],
    pub length: DescriptorRequirement,
}

#[derive(Debug, Clone, Copy, Deserialize, Serialize, Eq, PartialEq)]
pub enum DescriptorHash {
    #[serde(rename = "sha256")]
    Sha256,
}

#[derive(Debug, Clone, Copy, Deserialize, Serialize, Eq, PartialEq)]
pub enum DescriptorRequirement {
    #[serde(rename = "required")]
    Required,
}

#[derive(Debug, Clone, Deserialize, Serialize, Eq, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct Extensions {
    pub openpgp_key: ClosedExtensionHandling,
    pub openpgp_keyval: ClosedExtensionHandling,
    pub target_custom: TargetCustomHandling,
}

#[derive(Debug, Clone, Copy, Deserialize, Serialize, Eq, PartialEq)]
pub enum ClosedExtensionHandling {
    #[serde(rename = "reject-undefined")]
    RejectUndefined,
}

#[derive(Debug, Clone, Copy, Deserialize, Serialize, Eq, PartialEq)]
pub enum TargetCustomHandling {
    #[serde(rename = "preserve-opaque")]
    PreserveOpaque,
}

#[derive(Debug, Clone, Copy, Deserialize, Serialize, Eq, PartialEq)]
pub enum LifecycleQuestion {
    #[serde(rename = "accepted_time")]
    AcceptedTime,
    #[serde(rename = "consistent_snapshot")]
    ConsistentSnapshot,
    #[serde(rename = "expiry")]
    Expiry,
    #[serde(rename = "root_bootstrap")]
    RootBootstrap,
    #[serde(rename = "rollback")]
    Rollback,
}

#[derive(Debug, Clone, Deserialize, Serialize, Eq, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct OpenPgpProfile {
    pub certificate_version: u8,
    pub issuer_fingerprints: usize,
    pub key_type: OpenPgpKeyType,
    pub public_key_algorithm: u8,
    pub signature_components: [SignatureComponent; 2],
    pub signature_hash: SignatureHash,
    pub signature_scheme: SignatureScheme,
    pub signature_type: u8,
    pub signature_version: u8,
}

#[derive(Debug, Clone, Copy, Deserialize, Serialize, Eq, PartialEq)]
pub enum OpenPgpKeyType {
    #[serde(rename = "openpgp-rfc9580")]
    OpenPgpRfc9580,
}

#[derive(Debug, Clone, Copy, Deserialize, Serialize, Eq, PartialEq)]
pub enum SignatureComponent {
    #[serde(rename = "ed25519")]
    Ed25519,
    #[serde(rename = "ml-dsa-65")]
    MlDsa65,
}

#[derive(Debug, Clone, Copy, Deserialize, Serialize, Eq, PartialEq)]
pub enum SignatureHash {
    #[serde(rename = "sha512")]
    Sha512,
}

#[derive(Debug, Clone, Copy, Deserialize, Serialize, Eq, PartialEq)]
pub enum SignatureScheme {
    #[serde(rename = "openpgp-rfc9980-ml-dsa-65+ed25519-sha512")]
    OpenPgpRfc9980MlDsa65Ed25519Sha512,
}

#[derive(Debug, Clone, Copy, Deserialize, Serialize, Eq, PartialEq)]
pub enum ThresholdIdentity {
    #[serde(rename = "verified-v6-openpgp-signing-key-fingerprint")]
    VerifiedV6OpenPgpSigningKeyFingerprint,
}

#[derive(Debug, Clone, Copy, Deserialize, Serialize, Eq, PartialEq)]
pub enum TufSpecVersion {
    #[serde(rename = "1.0.36")]
    V1_0_36,
}

#[derive(Debug)]
pub enum ProfileError {
    Json(serde_json::Error),
    Invalid(&'static str),
}

impl fmt::Display for ProfileError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Json(error) => error.fmt(formatter),
            Self::Invalid(reason) => formatter.write_str(reason),
        }
    }
}

impl Error for ProfileError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::Json(error) => Some(error),
            Self::Invalid(_) => None,
        }
    }
}

impl From<serde_json::Error> for ProfileError {
    fn from(error: serde_json::Error) -> Self {
        Self::Json(error)
    }
}

pub fn parse_experimental_profile(bytes: &[u8]) -> Result<ExperimentalProfile, ProfileError> {
    let profile: ExperimentalProfile = serde_json::from_slice(bytes)?;
    profile.validate()?;
    Ok(profile)
}

impl ExperimentalProfile {
    fn validate(&self) -> Result<(), ProfileError> {
        if self.openpgp.certificate_version != 6 {
            return Err(ProfileError::Invalid("certificate version must be 6"));
        }
        if self.openpgp.issuer_fingerprints != 1 {
            return Err(ProfileError::Invalid(
                "exactly one issuer fingerprint is required",
            ));
        }
        if self.openpgp.public_key_algorithm != 30 {
            return Err(ProfileError::Invalid("public-key algorithm must be 30"));
        }
        if self.openpgp.signature_type != 0 {
            return Err(ProfileError::Invalid("signature type must be binary (0)"));
        }
        if self.openpgp.signature_version != 6 {
            return Err(ProfileError::Invalid("signature version must be 6"));
        }
        if self.openpgp.signature_components
            != [SignatureComponent::Ed25519, SignatureComponent::MlDsa65]
        {
            return Err(ProfileError::Invalid(
                "signature components must be Ed25519 and ML-DSA-65",
            ));
        }
        if self.lifecycle_fog
            != [
                LifecycleQuestion::AcceptedTime,
                LifecycleQuestion::ConsistentSnapshot,
                LifecycleQuestion::Expiry,
                LifecycleQuestion::RootBootstrap,
                LifecycleQuestion::Rollback,
            ]
        {
            return Err(ProfileError::Invalid(
                "lifecycle fog must list each unresolved question once",
            ));
        }
        Ok(())
    }
}
