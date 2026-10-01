mod support;

use codiquary_exact_target_tough_proof::{
    ApplicationIntent, AuthenticatedMetadata, ExactTargetVerifier, FixtureClock,
};
use serde::Deserialize;
use std::error::Error;
use support::{CountingTransport, RecordedTarget, RecordingHelper};
use url::Url;

const TRUSTED_ROOT: &[u8] = include_bytes!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/../../fixtures/conformance/exact-target-v1/metadata/base/1.root.json"
));
const SELECTION: &str = include_str!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/../../fixtures/conformance/exact-target-v1/selections/exact-match.json"
));
const EXPECTED: &str = include_str!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/../../fixtures/conformance/exact-target-v1/expected/exact-match.json"
));
const MANIFEST: &str = include_str!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/../../fixtures/conformance/exact-target-v1/manifest.json"
));

#[derive(Debug, Deserialize)]
struct FixtureManifest {
    cases: Vec<FixtureCase>,
}

#[derive(Debug, Deserialize)]
struct FixtureCase {
    application_intent: ApplicationIntent,
    case_id: String,
    expected: ExpectedResult,
    expected_result: String,
    selection: String,
}

#[derive(Debug, Deserialize, Eq, PartialEq)]
struct ExpectedResult {
    case_id: String,
    expected_authenticated_target: ExpectedTarget,
    expected_helper_call_count: usize,
    expected_metadata_request_count: usize,
    expected_metadata_requests: Vec<String>,
    expected_semantic_observation: String,
    expected_target_body_request: String,
    expected_target_body_request_count: usize,
    fixed_verification_instant: String,
    same_held_allocation: bool,
}

#[derive(Debug, Deserialize, Eq, PartialEq)]
struct ExpectedTarget {
    identity: String,
    length: u64,
    sha256: String,
}

fn error_chain(mut error: &(dyn Error + 'static)) -> String {
    let mut messages = vec![error.to_string()];
    while let Some(source) = error.source() {
        messages.push(source.to_string());
        error = source;
    }
    messages.join(": ")
}

#[tokio::test]
async fn exact_match_reaches_observation_helper_once() {
    let intent: ApplicationIntent =
        serde_json::from_str(SELECTION).expect("committed exact-match selection must parse");
    let manifest: FixtureManifest =
        serde_json::from_str(MANIFEST).expect("committed fixture manifest must parse");
    let expected: ExpectedResult =
        serde_json::from_str(EXPECTED).expect("committed exact-match expectation must parse");
    let case = manifest
        .cases
        .into_iter()
        .find(|case| case.case_id == "exact-match")
        .expect("fixture manifest must contain the exact-match case");
    assert_eq!(case.selection, "selections/exact-match.json");
    assert_eq!(case.expected_result, "expected/exact-match.json");
    assert_eq!(case.application_intent, intent);
    assert_eq!(case.expected, expected);
    assert_eq!(expected.case_id, "exact-match");
    assert_eq!(
        expected.expected_semantic_observation,
        "verified_exact_target"
    );
    assert_eq!(
        expected.expected_helper_call_count, 1,
        "the first tracer is bound to the one-call fixture oracle"
    );
    assert_eq!(
        expected.expected_metadata_requests.len(),
        expected.expected_metadata_request_count
    );
    assert!(expected.same_held_allocation);

    let transport = CountingTransport::new();
    let helper = RecordingHelper::new();
    let clock = FixtureClock::new(
        expected
            .fixed_verification_instant
            .parse()
            .expect("committed fixture instant must parse"),
    );
    let metadata = AuthenticatedMetadata::new(
        TRUSTED_ROOT.to_vec(),
        Url::parse("fixture://exact-target/metadata/")
            .expect("inert metadata identifier must parse"),
        Url::parse("fixture://exact-target/targets/")
            .expect("inert target identifier must parse"),
    );

    let result = ExactTargetVerifier::verify(metadata, &intent, transport.clone(), &helper, &clock)
        .await;

    if let Err(error) = result {
        let diagnostic = error_chain(&error);
        assert!(
            diagnostic.contains("Failed to parse trusted root metadata")
                && diagnostic.contains("openpgp-rfc9580"),
            "unexpected loader failure: {diagnostic}"
        );
        eprintln!("{diagnostic}");
    }

    let helper_calls = helper.call_count();
    if helper_calls == 0 {
        println!("EXACT_TARGET_BEHAVIOR_RED: helper_calls=0 expected=1");
    }
    assert_eq!(helper_calls, expected.expected_helper_call_count);

    assert_eq!(
        transport.metadata_requests(),
        expected.expected_metadata_requests
    );
    assert_eq!(
        transport.target_body_requests(),
        vec![expected.expected_target_body_request.clone()]
    );
    assert_eq!(
        transport.target_body_requests().len(),
        expected.expected_target_body_request_count
    );
    assert_eq!(
        helper.observations(),
        vec![RecordedTarget {
            identity: expected.expected_authenticated_target.identity,
            length: expected.expected_authenticated_target.length,
            sha256: expected.expected_authenticated_target.sha256,
        }]
    );
}
