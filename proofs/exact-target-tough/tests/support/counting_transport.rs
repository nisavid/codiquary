use async_trait::async_trait;
use futures::stream;
use std::sync::{Arc, Mutex};
use tough::{Bytes, Transport, TransportError, TransportErrorKind, TransportStream};
use url::Url;

const SNAPSHOT: &[u8] = include_bytes!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/../../fixtures/conformance/exact-target-v1/metadata/base/1.snapshot.json"
));
const TARGETS: &[u8] = include_bytes!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/../../fixtures/conformance/exact-target-v1/metadata/base/1.targets.json"
));
const TIMESTAMP: &[u8] = include_bytes!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/../../fixtures/conformance/exact-target-v1/metadata/base/timestamp.json"
));
const TARGET_BODY: &[u8] = include_bytes!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/../../fixtures/conformance/exact-target-v1/targets/",
    "e2980f3df2f5f644360a732ce7a09e07af4d91010360a933077918622c57b326.exact-match.txt"
));

const METADATA_PREFIX: &str = "/metadata/";
const TARGETS_PREFIX: &str = "/targets/";

#[derive(Debug, Default)]
struct RequestLog {
    metadata: Vec<String>,
    target_bodies: Vec<String>,
}

/// Serves only the committed fixture corpus and records metadata separately
/// from target-body requests.
#[derive(Clone, Debug, Default)]
pub struct CountingTransport {
    requests: Arc<Mutex<RequestLog>>,
}

impl CountingTransport {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn metadata_requests(&self) -> Vec<String> {
        self.requests
            .lock()
            .expect("request log lock poisoned")
            .metadata
            .clone()
    }

    pub fn target_body_requests(&self) -> Vec<String> {
        self.requests
            .lock()
            .expect("request log lock poisoned")
            .target_bodies
            .clone()
    }

    fn record_metadata(&self, name: &str) {
        self.requests
            .lock()
            .expect("request log lock poisoned")
            .metadata
            .push(name.to_owned());
    }

    fn record_target_body(&self, name: &str) {
        self.requests
            .lock()
            .expect("request log lock poisoned")
            .target_bodies
            .push(name.to_owned());
    }

    fn reject(url: &Url, kind: TransportErrorKind) -> TransportError {
        TransportError::new(kind, url.as_str())
    }
}

#[async_trait]
impl Transport for CountingTransport {
    async fn fetch(&self, url: Url) -> Result<TransportStream, TransportError> {
        if url.scheme() != "fixture" || url.host_str() != Some("exact-target") {
            return Err(Self::reject(&url, TransportErrorKind::UnsupportedUrlScheme));
        }

        let body = if let Some(name) = url.path().strip_prefix(METADATA_PREFIX) {
            self.record_metadata(name);
            match name {
                "2.root.json" => {
                    return Err(Self::reject(&url, TransportErrorKind::FileNotFound));
                }
                "timestamp.json" => TIMESTAMP,
                "1.snapshot.json" => SNAPSHOT,
                "1.targets.json" => TARGETS,
                _ => return Err(Self::reject(&url, TransportErrorKind::Other)),
            }
        } else if let Some(name) = url.path().strip_prefix(TARGETS_PREFIX) {
            self.record_target_body(name);
            match name {
                "e2980f3df2f5f644360a732ce7a09e07af4d91010360a933077918622c57b326.exact-match.txt" => {
                    TARGET_BODY
                }
                _ => return Err(Self::reject(&url, TransportErrorKind::Other)),
            }
        } else {
            return Err(Self::reject(&url, TransportErrorKind::Other));
        };

        Ok(Box::pin(stream::once(async move {
            Ok(Bytes::from_static(body))
        })))
    }
}
