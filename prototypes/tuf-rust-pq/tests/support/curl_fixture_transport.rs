use async_trait::async_trait;
use std::fs::{self, OpenOptions};
use std::io::{self, ErrorKind};
use std::net::{IpAddr, Ipv4Addr, SocketAddr};
use std::path::{Path, PathBuf};
use std::process::{Command, ExitStatus, Stdio};
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, Mutex};
use tough::{FilesystemTransport, Transport, TransportError, TransportErrorKind, TransportStream};
use url::Url;

const CURL: &str = "/usr/bin/curl";

#[derive(Clone, Debug)]
pub struct CurlFixtureTransport {
    inner: Arc<Inner>,
}

#[derive(Debug)]
struct Inner {
    origin: Url,
    port: u16,
    scratch_dir: PathBuf,
    next_sequence: AtomicU64,
    evidence: Mutex<Vec<CurlFetchEvidence>>,
}

#[derive(Clone, Debug)]
pub struct CurlFetchEvidence {
    pub sequence: u64,
    pub request_url: String,
    pub body_path: PathBuf,
    pub curl_status: Option<ExitStatus>,
    pub http_status: Option<u16>,
    pub stdout: Vec<u8>,
    pub stderr: Vec<u8>,
    pub body: Vec<u8>,
    pub invocation_error: Option<String>,
    pub body_read_error: Option<String>,
}

impl CurlFixtureTransport {
    pub fn new(origin: SocketAddr, scratch_dir: impl Into<PathBuf>) -> io::Result<Self> {
        if origin.ip() != IpAddr::V4(Ipv4Addr::LOCALHOST) || origin.port() == 0 {
            return Err(io::Error::new(
                ErrorKind::InvalidInput,
                "curl fixture origin must be 127.0.0.1 with an assigned port",
            ));
        }

        let scratch_dir = scratch_dir.into();
        if !scratch_dir.is_absolute() {
            return Err(io::Error::new(
                ErrorKind::InvalidInput,
                "curl fixture scratch directory must be absolute",
            ));
        }
        fs::create_dir_all(&scratch_dir)?;

        let port = origin.port();
        let origin = Url::parse(&format!("http://127.0.0.1:{port}/")).map_err(|error| {
            io::Error::new(
                ErrorKind::InvalidInput,
                format!("invalid fixture origin: {error}"),
            )
        })?;

        Ok(Self {
            inner: Arc::new(Inner {
                origin,
                port,
                scratch_dir,
                next_sequence: AtomicU64::new(0),
                evidence: Mutex::new(Vec::new()),
            }),
        })
    }

    pub fn origin(&self) -> Url {
        self.inner.origin.clone()
    }

    pub fn evidence(&self) -> Vec<CurlFetchEvidence> {
        match self.inner.evidence.lock() {
            Ok(entries) => entries.clone(),
            Err(poisoned) => poisoned.into_inner().clone(),
        }
    }

    fn accepts(&self, url: &Url) -> bool {
        url.scheme() == "http"
            && url.username().is_empty()
            && url.password().is_none()
            && url.host_str() == Some("127.0.0.1")
            && url.port_or_known_default() == Some(self.inner.port)
            && url.query().is_none()
            && url.fragment().is_none()
    }

    fn reserve_body_file(&self) -> io::Result<(u64, PathBuf)> {
        loop {
            let sequence = self.inner.next_sequence.fetch_add(1, Ordering::Relaxed);
            let body_path = self
                .inner
                .scratch_dir
                .join(format!("curl-response-{sequence:020}.body"));

            match OpenOptions::new()
                .write(true)
                .create_new(true)
                .open(&body_path)
            {
                Ok(_) => return Ok((sequence, body_path)),
                Err(error) if error.kind() == ErrorKind::AlreadyExists => continue,
                Err(error) => return Err(error),
            }
        }
    }

    fn record(&self, evidence: CurlFetchEvidence) {
        match self.inner.evidence.lock() {
            Ok(mut entries) => entries.push(evidence),
            Err(poisoned) => {
                let mut entries = poisoned.into_inner();
                entries.push(evidence);
            }
        }
    }
}

#[async_trait]
impl Transport for CurlFixtureTransport {
    async fn fetch(&self, url: Url) -> Result<TransportStream, TransportError> {
        if !self.accepts(&url) {
            return Err(TransportError::new(
                TransportErrorKind::UnsupportedUrlScheme,
                url.as_str(),
            ));
        }

        let (sequence, body_path) = self
            .reserve_body_file()
            .map_err(|error| transport_error(&url, error))?;
        let request_url = url.to_string();

        let output = Command::new(CURL)
            .arg("--disable")
            .arg("--noproxy")
            .arg("*")
            .arg("--proto")
            .arg("=http")
            .arg("--http1.1")
            .arg("--connect-timeout")
            .arg("2")
            .arg("--max-time")
            .arg("5")
            .arg("--max-redirs")
            .arg("0")
            .arg("--silent")
            .arg("--show-error")
            .arg("--output")
            .arg(&body_path)
            .arg("--write-out")
            .arg("%{http_code}")
            .arg("--url")
            .arg(url.as_str())
            .env_clear()
            .current_dir(&self.inner.scratch_dir)
            .stdin(Stdio::null())
            .output();

        let output = match output {
            Ok(output) => output,
            Err(error) => {
                let (body, body_read_error) = read_body(&body_path);
                self.record(CurlFetchEvidence {
                    sequence,
                    request_url,
                    body_path,
                    curl_status: None,
                    http_status: None,
                    stdout: Vec::new(),
                    stderr: Vec::new(),
                    body,
                    invocation_error: Some(error.to_string()),
                    body_read_error,
                });
                return Err(transport_error(&url, error));
            }
        };

        let http_status = parse_http_status(&output.stdout);
        let (body, body_read_error) = read_body(&body_path);
        self.record(CurlFetchEvidence {
            sequence,
            request_url,
            body_path: body_path.clone(),
            curl_status: Some(output.status),
            http_status,
            stdout: output.stdout.clone(),
            stderr: output.stderr.clone(),
            body,
            invocation_error: None,
            body_read_error: body_read_error.clone(),
        });

        if !output.status.success() {
            return Err(transport_message(
                &url,
                format!("curl exited with {}", output.status),
            ));
        }
        if let Some(error) = body_read_error {
            return Err(transport_message(&url, error));
        }

        let http_status = http_status.ok_or_else(|| {
            transport_message(&url, "curl did not emit one three-digit HTTP status")
        })?;
        if http_status == 404 {
            return Err(TransportError::new(
                TransportErrorKind::FileNotFound,
                url.as_str(),
            ));
        }
        if !(200..300).contains(&http_status) {
            return Err(transport_message(
                &url,
                format!("HTTP response status was {http_status}"),
            ));
        }

        let file_url = Url::from_file_path(&body_path).map_err(|_| {
            transport_message(&url, "response body path is not an absolute file path")
        })?;
        FilesystemTransport.fetch(file_url).await
    }
}

fn parse_http_status(stdout: &[u8]) -> Option<u16> {
    let text = std::str::from_utf8(stdout).ok()?;
    if text.len() != 3 || !text.bytes().all(|byte| byte.is_ascii_digit()) {
        return None;
    }
    text.parse().ok()
}

fn read_body(path: &Path) -> (Vec<u8>, Option<String>) {
    match fs::read(path) {
        Ok(body) => (body, None),
        Err(error) => (
            Vec::new(),
            Some(format!("could not read curl response body: {error}")),
        ),
    }
}

fn transport_error(url: &Url, error: io::Error) -> TransportError {
    TransportError::new_with_cause(TransportErrorKind::Other, url.as_str(), error)
}

fn transport_message(url: &Url, message: impl Into<String>) -> TransportError {
    transport_error(url, io::Error::other(message.into()))
}
