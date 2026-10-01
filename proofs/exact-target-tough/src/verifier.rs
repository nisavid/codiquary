use crate::ApplicationIntent;
use jiff::Timestamp;
use thiserror::Error;
use tough::{RepositoryLoader, Transport};
use url::Url;

/// Authenticated-metadata locations supplied to [`ExactTargetVerifier`].
#[derive(Clone, Debug)]
pub struct AuthenticatedMetadata {
    trusted_root: Vec<u8>,
    metadata_base_url: Url,
    targets_base_url: Url,
}

impl AuthenticatedMetadata {
    /// Creates locations for one repository without accepting a target path.
    pub fn new(trusted_root: Vec<u8>, metadata_base_url: Url, targets_base_url: Url) -> Self {
        Self {
            trusted_root,
            metadata_base_url,
            targets_base_url,
        }
    }
}

/// The fixed instant confined to the proof harness.
#[derive(Debug)]
pub struct FixtureClock {
    fixed_instant: Timestamp,
}

impl FixtureClock {
    /// Creates a proof-only clock value for a committed fixture instant.
    pub fn new(fixed_instant: Timestamp) -> Self {
        Self { fixed_instant }
    }

    /// Returns the fixed fixture instant.
    pub fn fixed_instant(&self) -> &Timestamp {
        &self.fixed_instant
    }
}

/// The capability-restricted target view supplied to an observation helper.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct AuthenticatedTargetObservation<'a> {
    identity: &'a str,
    length: u64,
    sha256: &'a str,
}

impl AuthenticatedTargetObservation<'_> {
    /// Returns the authenticated target identity.
    pub fn identity(&self) -> &str {
        self.identity
    }

    /// Returns the authenticated target byte length.
    pub fn length(&self) -> u64 {
        self.length
    }

    /// Returns the authenticated target SHA-256.
    pub fn sha256(&self) -> &str {
        self.sha256
    }
}

/// An observation-only helper with no acquisition or actuation capability.
pub trait ObservationHelper: Send + Sync {
    /// Observes an authenticated target after the verifier has held its bytes.
    fn observe(&self, target: AuthenticatedTargetObservation<'_>);
}

/// Errors returned while entering the accepted verifier seam.
#[derive(Debug, Error)]
pub enum VerificationError {
    /// Tough rejected the authenticated metadata while loading the repository.
    #[error("Tough RepositoryLoader rejected authenticated metadata")]
    RepositoryLoad {
        /// The unmodified Tough diagnostic and its causal chain.
        #[source]
        source: tough::error::Error,
    },
}

/// The deep module for the disposable exact-target proof.
#[derive(Clone, Copy, Debug, Default)]
pub struct ExactTargetVerifier;

impl ExactTargetVerifier {
    /// Enters Tough's real loader with the accepted injected boundaries.
    pub async fn verify<T, H>(
        metadata: AuthenticatedMetadata,
        intent: &ApplicationIntent,
        transport: T,
        helper: &H,
        clock: &FixtureClock,
    ) -> Result<(), VerificationError>
    where
        T: Transport + Send + Sync + 'static,
        H: ObservationHelper,
    {
        let AuthenticatedMetadata {
            trusted_root,
            metadata_base_url,
            targets_base_url,
        } = metadata;

        RepositoryLoader::new(&trusted_root, metadata_base_url, targets_base_url)
            .transport(transport)
            .load()
            .await
            .map_err(|source| VerificationError::RepositoryLoad { source })?;

        let _accepted_inputs = (intent, helper, clock);
        Ok(())
    }
}
