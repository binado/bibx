//! A minimal in-process HTTP server that replays canned responses, for provider tests.

use std::io::{BufRead, BufReader, Write};
use std::net::{Shutdown, TcpListener, TcpStream};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use std::thread::JoinHandle;

/// A canned response.
#[derive(Debug, Clone)]
pub struct Reply {
    pub status: u16,
    pub content_type: &'static str,
    pub body: String,
}

impl Reply {
    pub fn bibtex(body: impl Into<String>) -> Reply {
        Reply {
            status: 200,
            content_type: "application/x-bibtex",
            body: body.into(),
        }
    }

    pub fn status(status: u16, body: impl Into<String>) -> Reply {
        Reply {
            status,
            content_type: "application/json",
            body: body.into(),
        }
    }
}

/// One received request: its request line target and its headers, lowercased by name.
#[derive(Debug, Clone)]
pub struct Request {
    pub target: String,
    pub headers: Vec<(String, String)>,
}

impl Request {
    pub fn header(&self, name: &str) -> Option<&str> {
        self.headers
            .iter()
            .find(|(k, _)| k == name)
            .map(|(_, v)| v.as_str())
    }
}

pub struct MockServer {
    /// `http://127.0.0.1:<port>`
    pub url: String,
    requests: Arc<Mutex<Vec<Request>>>,
    stop: Arc<AtomicBool>,
    addr: std::net::SocketAddr,
    thread: Option<JoinHandle<()>>,
}

impl MockServer {
    /// Serve `routes`, keyed by the exact request target (path and query). Anything else
    /// gets a 404 with a JSON body.
    pub fn start(routes: Vec<(String, Reply)>) -> MockServer {
        let listener = TcpListener::bind("127.0.0.1:0").expect("bind a local port");
        let addr = listener.local_addr().expect("local address");
        let requests = Arc::new(Mutex::new(Vec::new()));
        let stop = Arc::new(AtomicBool::new(false));
        let thread = {
            let (requests, stop) = (requests.clone(), stop.clone());
            std::thread::spawn(move || {
                for stream in listener.incoming() {
                    if stop.load(Ordering::SeqCst) {
                        break;
                    }
                    if let Ok(stream) = stream {
                        let _ = handle(stream, &routes, &requests);
                    }
                }
            })
        };
        MockServer {
            url: format!("http://{addr}"),
            requests,
            stop,
            addr,
            thread: Some(thread),
        }
    }

    pub fn requests(&self) -> Vec<Request> {
        self.requests.lock().map(|r| r.clone()).unwrap_or_default()
    }
}

impl Drop for MockServer {
    fn drop(&mut self) {
        self.stop.store(true, Ordering::SeqCst);
        // Wake the accept loop so it sees the flag.
        let _ = TcpStream::connect(self.addr);
        if let Some(thread) = self.thread.take() {
            let _ = thread.join();
        }
    }
}

fn handle(
    mut stream: TcpStream,
    routes: &[(String, Reply)],
    requests: &Mutex<Vec<Request>>,
) -> std::io::Result<()> {
    let mut reader = BufReader::new(stream.try_clone()?);
    let mut line = String::new();
    reader.read_line(&mut line)?;
    let target = line.split_whitespace().nth(1).unwrap_or("").to_string();
    let mut headers = Vec::new();
    loop {
        line.clear();
        if reader.read_line(&mut line)? == 0 || line.trim().is_empty() {
            break;
        }
        if let Some((k, v)) = line.split_once(':') {
            headers.push((k.trim().to_ascii_lowercase(), v.trim().to_string()));
        }
    }
    let reply = routes
        .iter()
        .find(|(t, _)| *t == target)
        .map(|(_, r)| r.clone())
        .unwrap_or_else(|| Reply::status(404, r#"{"status": 404, "message": "no such route"}"#));
    if let Ok(mut all) = requests.lock() {
        all.push(Request { target, headers });
    }
    write!(
        stream,
        "HTTP/1.1 {} X\r\nContent-Type: {}\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}",
        reply.status,
        reply.content_type,
        reply.body.len(),
        reply.body
    )?;
    stream.flush()?;
    stream.shutdown(Shutdown::Both)
}
