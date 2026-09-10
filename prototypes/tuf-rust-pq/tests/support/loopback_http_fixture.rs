use std::collections::BTreeMap;
use std::io::{self, ErrorKind, Read, Write};
use std::net::{Ipv4Addr, Shutdown, SocketAddr, TcpListener, TcpStream};
use std::sync::mpsc::{self, Receiver, RecvTimeoutError, Sender, TryRecvError};
use std::thread::{self, JoinHandle};
use std::time::{Duration, Instant};

const ACCEPT_POLL: Duration = Duration::from_millis(10);
const IO_DEADLINE: Duration = Duration::from_secs(2);
const MAX_REQUEST_BYTES: usize = 64 * 1024;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct HttpExchangeRecord {
    pub request_bytes: Vec<u8>,
    pub response_bytes: Vec<u8>,
}

#[derive(Debug, Default, PartialEq, Eq)]
pub struct LoopbackHttpEvidence {
    pub exchanges: Vec<HttpExchangeRecord>,
    pub server_errors: Vec<String>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct HttpFixtureRoute {
    pub path: String,
    pub body: Vec<u8>,
}

impl HttpFixtureRoute {
    pub fn new(path: impl Into<String>, body: impl Into<Vec<u8>>) -> Self {
        Self {
            path: path.into(),
            body: body.into(),
        }
    }
}

#[derive(Debug)]
pub struct LoopbackHttpFixture {
    address: SocketAddr,
    shutdown: Option<Sender<()>>,
    join: Option<JoinHandle<LoopbackHttpEvidence>>,
}

impl LoopbackHttpFixture {
    pub fn start(routes: impl IntoIterator<Item = HttpFixtureRoute>) -> io::Result<Self> {
        let mut route_map = BTreeMap::new();
        for route in routes {
            validate_route_path(&route.path)?;
            if route_map
                .insert(route.path.into_bytes(), route.body)
                .is_some()
            {
                return Err(io::Error::new(
                    ErrorKind::InvalidInput,
                    "duplicate loopback HTTP fixture route",
                ));
            }
        }

        let listener = TcpListener::bind(SocketAddr::from((Ipv4Addr::LOCALHOST, 0)))?;
        listener.set_nonblocking(true)?;
        let address = listener.local_addr()?;
        let (shutdown_tx, shutdown_rx) = mpsc::channel();
        let join = thread::Builder::new()
            .name("codiquary-loopback-http-fixture".to_owned())
            .spawn(move || serve(listener, route_map, shutdown_rx))?;

        Ok(Self {
            address,
            shutdown: Some(shutdown_tx),
            join: Some(join),
        })
    }

    pub fn address(&self) -> SocketAddr {
        self.address
    }

    pub fn stop_and_join(mut self) -> io::Result<LoopbackHttpEvidence> {
        self.join_inner()
    }

    fn join_inner(&mut self) -> io::Result<LoopbackHttpEvidence> {
        if let Some(shutdown) = self.shutdown.take() {
            let _ = shutdown.send(());
        }

        match self.join.take() {
            Some(join) => join
                .join()
                .map_err(|_| io::Error::other("loopback HTTP fixture thread panicked")),
            None => Ok(LoopbackHttpEvidence::default()),
        }
    }
}

impl Drop for LoopbackHttpFixture {
    fn drop(&mut self) {
        let _ = self.join_inner();
    }
}

fn validate_route_path(path: &str) -> io::Result<()> {
    if !path.starts_with('/')
        || !path.is_ascii()
        || path
            .bytes()
            .any(|byte| matches!(byte, b'?' | b'#' | b'\r' | b'\n'))
    {
        return Err(io::Error::new(
            ErrorKind::InvalidInput,
            "loopback HTTP fixture routes must be ASCII origin paths without query or fragment",
        ));
    }
    Ok(())
}

fn serve(
    listener: TcpListener,
    routes: BTreeMap<Vec<u8>, Vec<u8>>,
    shutdown: Receiver<()>,
) -> LoopbackHttpEvidence {
    let mut evidence = LoopbackHttpEvidence::default();

    loop {
        match shutdown.try_recv() {
            Ok(()) | Err(TryRecvError::Disconnected) => break,
            Err(TryRecvError::Empty) => {}
        }

        match listener.accept() {
            Ok((mut stream, peer)) => {
                if let Err(error) = handle_connection(&mut stream, &routes, &mut evidence) {
                    evidence
                        .server_errors
                        .push(format!("connection from {peer} failed: {error}"));
                }
            }
            Err(error) if error.kind() == ErrorKind::WouldBlock => {
                match shutdown.recv_timeout(ACCEPT_POLL) {
                    Ok(()) | Err(RecvTimeoutError::Disconnected) => break,
                    Err(RecvTimeoutError::Timeout) => {}
                }
            }
            Err(error) => {
                evidence
                    .server_errors
                    .push(format!("listener accept failed: {error}"));
                break;
            }
        }
    }

    evidence
}

fn handle_connection(
    stream: &mut TcpStream,
    routes: &BTreeMap<Vec<u8>, Vec<u8>>,
    evidence: &mut LoopbackHttpEvidence,
) -> io::Result<()> {
    let mut request_bytes = Vec::new();
    if let Err(error) = read_request(stream, &mut request_bytes) {
        evidence.exchanges.push(HttpExchangeRecord {
            request_bytes,
            response_bytes: Vec::new(),
        });
        return Err(error);
    }

    let response = response_for(&request_bytes, routes);
    let mut response_bytes = Vec::new();
    let write_result = write_response(stream, &response, &mut response_bytes);
    evidence.exchanges.push(HttpExchangeRecord {
        request_bytes,
        response_bytes,
    });
    write_result
}

fn read_request(stream: &mut TcpStream, request: &mut Vec<u8>) -> io::Result<()> {
    let deadline = Instant::now() + IO_DEADLINE;
    let mut buffer = [0_u8; 4096];

    loop {
        if request
            .windows(b"\r\n\r\n".len())
            .any(|window| window == b"\r\n\r\n")
        {
            return Ok(());
        }
        if request.len() >= MAX_REQUEST_BYTES {
            return Err(io::Error::new(
                ErrorKind::InvalidData,
                "loopback HTTP request headers exceeded the fixture limit",
            ));
        }

        let remaining = deadline
            .checked_duration_since(Instant::now())
            .ok_or_else(|| {
                io::Error::new(ErrorKind::TimedOut, "loopback HTTP request timed out")
            })?;
        stream.set_read_timeout(Some(remaining))?;
        let available = buffer.len().min(MAX_REQUEST_BYTES - request.len());
        match stream.read(&mut buffer[..available]) {
            Ok(0) => {
                return Err(io::Error::new(
                    ErrorKind::UnexpectedEof,
                    "loopback HTTP client closed before completing request headers",
                ));
            }
            Ok(count) => request.extend_from_slice(&buffer[..count]),
            Err(error) => return Err(error),
        }
    }
}

fn response_for(request: &[u8], routes: &BTreeMap<Vec<u8>, Vec<u8>>) -> Vec<u8> {
    let body = request_target(request).and_then(|target| routes.get(target));
    match body {
        Some(body) => response_bytes("200 OK", body),
        None => response_bytes("404 Not Found", &[]),
    }
}

fn request_target(request: &[u8]) -> Option<&[u8]> {
    let line_end = request.windows(2).position(|window| window == b"\r\n")?;
    let mut fields = request[..line_end].split(|byte| *byte == b' ');
    let method = fields.next()?;
    let target = fields.next()?;
    let version = fields.next()?;
    if fields.next().is_some()
        || method != b"GET"
        || version != b"HTTP/1.1"
        || !target.starts_with(b"/")
    {
        return None;
    }
    Some(target)
}

fn response_bytes(status: &str, body: &[u8]) -> Vec<u8> {
    let mut response = format!(
        "HTTP/1.1 {status}\r\nContent-Length: {}\r\nConnection: close\r\n\r\n",
        body.len()
    )
    .into_bytes();
    response.extend_from_slice(body);
    response
}

fn write_response(
    stream: &mut TcpStream,
    response: &[u8],
    written: &mut Vec<u8>,
) -> io::Result<()> {
    let deadline = Instant::now() + IO_DEADLINE;

    while written.len() < response.len() {
        let remaining = deadline
            .checked_duration_since(Instant::now())
            .ok_or_else(|| {
                io::Error::new(ErrorKind::TimedOut, "loopback HTTP response timed out")
            })?;
        stream.set_write_timeout(Some(remaining))?;
        match stream.write(&response[written.len()..]) {
            Ok(0) => {
                return Err(io::Error::new(
                    ErrorKind::WriteZero,
                    "loopback HTTP response write made no progress",
                ));
            }
            Ok(count) => {
                let start = written.len();
                written.extend_from_slice(&response[start..start + count]);
            }
            Err(error) => return Err(error),
        }
    }

    stream.flush()?;
    let _ = stream.shutdown(Shutdown::Write);
    Ok(())
}
