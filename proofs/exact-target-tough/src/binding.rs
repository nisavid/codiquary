use serde::Deserialize;

/// The closed application intent used to select one authenticated target.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct ApplicationIntent {
    product: String,
    version: String,
    channel: String,
    purpose: String,
    platform: String,
    policy_profile: String,
}
