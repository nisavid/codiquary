//! Experimental support for the disposable Tough exact-target proof.

#![forbid(unsafe_code)]

mod binding;
mod verifier;

pub use binding::ApplicationIntent;
pub use verifier::{
    AuthenticatedMetadata, AuthenticatedTargetObservation, ExactTargetVerifier, FixtureClock,
    ObservationHelper, VerificationError,
};
