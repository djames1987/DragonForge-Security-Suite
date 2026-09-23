use std::collections::{HashSet, VecDeque};
use std::fs::{self, OpenOptions};
use std::io::{BufRead, BufReader, Read, Write};
use std::net::{Ipv4Addr, TcpListener, TcpStream};
use std::path::Path;
use std::time::{Duration, Instant};

use base64::Engine;
use base64::engine::general_purpose::STANDARD as BASE64;
use rand_core::{OsRng, RngCore};

use crate::client::AgentClient;
use crate::error::{AgentError, Result};
use crate::paths::AgentPaths;
use crate::protocol::{
    AGENT_PROTOCOL_MAJOR, AGENT_PROTOCOL_MINOR, HealthWire, MAX_CLOCK_SKEW_MS, MAX_WIRE_BYTES,
    RequestWire, ResponseWire, RuntimeDescriptor, SESSION_KEY_BYTES, authorize_request,
    decode_nonce, now_ms, request_message, response_message, sign_hex, verify_hex,
};

const READ_TIMEOUT: Duration = Duration::from_secs(2);
const WRITE_TIMEOUT: Duration = Duration::from_secs(2);
const MAX_REPLAY_NONCES: usize = 4_096;

#[derive(Debug)]
pub struct AgentServer {
    paths: AgentPaths,
    integrity: crate::AgentIntegrityRuntime,
}

impl AgentServer {
    #[must_use]
    pub fn from_paths(paths: AgentPaths) -> Self {
        let integrity = crate::AgentIntegrityRuntime::from_paths(
            paths.root().join("continuous-integrity-v1.json"),
            paths.root().join("integrity-baseline-v1.json"),
        );
        Self { paths, integrity }
    }

    pub fn discover() -> Result<Self> {
        Ok(Self {
            paths: AgentPaths::discover()?,
            integrity: crate::AgentIntegrityRuntime::discover()?,
        })
    }

    pub fn run(&self) -> Result<()> {
        self.run_internal(None)
    }

    fn run_internal(&self, max_connections: Option<usize>) -> Result<()> {
        fs::create_dir_all(self.paths.root())
            .map_err(|_| AgentError::Io("agent runtime directory could not be created"))?;
        let lock = acquire_runtime_lock(&self.paths)?;
        cleanup_runtime_files(&self.paths);
        let _guard = RuntimeGuard {
            paths: self.paths.clone(),
            _lock: Some(lock),
        };

        let listener = TcpListener::bind((Ipv4Addr::LOCALHOST, 0))
            .map_err(|_| AgentError::Io("agent loopback listener could not be created"))?;
        listener.set_nonblocking(true).map_err(|_| {
            AgentError::Io("agent listener nonblocking mode could not be configured")
        })?;
        let port = listener
            .local_addr()
            .map_err(|_| AgentError::Io("agent listener address is unavailable"))?
            .port();

        let mut session_key = [0_u8; SESSION_KEY_BYTES];
        OsRng.fill_bytes(&mut session_key);
        write_runtime_files(&self.paths, port, &session_key)?;

        let started = Instant::now();
        let mut replay = ReplayCache::default();
        let mut handled = 0_usize;
        let mut next_integrity_poll = Instant::now();
        loop {
            if Instant::now() >= next_integrity_poll {
                let _ = self.integrity.tick();
                next_integrity_poll = Instant::now() + Duration::from_secs(5);
            }
            match listener.accept() {
                Ok((mut stream, _)) => {
                    let shutdown =
                        handle_connection(&mut stream, &session_key, started, &mut replay)
                            .unwrap_or(false);
                    handled += 1;
                    if shutdown || max_connections.is_some_and(|limit| handled >= limit) {
                        break;
                    }
                }
                Err(error) if error.kind() == std::io::ErrorKind::WouldBlock => {
                    std::thread::sleep(Duration::from_millis(250));
                }
                Err(_) => return Err(AgentError::Io("agent client connection failed")),
            }
        }
        Ok(())
    }
}

#[derive(Debug)]
struct RuntimeGuard {
    paths: AgentPaths,
    _lock: Option<fs::File>,
}

impl Drop for RuntimeGuard {
    fn drop(&mut self) {
        let _ = self._lock.take();
        cleanup_runtime_files(&self.paths);
        let _ = fs::remove_file(self.paths.lock_file());
    }
}

#[derive(Debug, Default)]
struct ReplayCache {
    order: VecDeque<String>,
    seen: HashSet<String>,
}

impl ReplayCache {
    fn insert_new(&mut self, nonce: String) -> bool {
        if self.seen.contains(&nonce) {
            return false;
        }
        self.seen.insert(nonce.clone());
        self.order.push_back(nonce);
        while self.order.len() > MAX_REPLAY_NONCES {
            if let Some(oldest) = self.order.pop_front() {
                self.seen.remove(&oldest);
            }
        }
        true
    }
}

fn handle_connection(
    stream: &mut TcpStream,
    session_key: &[u8],
    started: Instant,
    replay: &mut ReplayCache,
) -> Result<bool> {
    stream
        .set_read_timeout(Some(READ_TIMEOUT))
        .map_err(|_| AgentError::Io("agent read timeout could not be configured"))?;
    stream
        .set_write_timeout(Some(WRITE_TIMEOUT))
        .map_err(|_| AgentError::Io("agent write timeout could not be configured"))?;

    let mut reader = BufReader::new(
        stream
            .try_clone()
            .map_err(|_| AgentError::Io("agent connection could not be cloned"))?,
    );
    let mut line = String::new();
    let count = reader
        .by_ref()
        .take(MAX_WIRE_BYTES as u64)
        .read_line(&mut line)
        .map_err(|_| AgentError::Io("agent request could not be read"))?;
    if count == 0 || count >= MAX_WIRE_BYTES {
        return Err(AgentError::Protocol(
            "agent request is missing or oversized",
        ));
    }
    let request: RequestWire = serde_json::from_str(line.trim_end())
        .map_err(|_| AgentError::Protocol("agent request is malformed"))?;

    let response = match validate_request(&request, session_key, replay) {
        Ok(()) if request.action == "health" => ResponseWire {
            request_id: request.request_id,
            ok: true,
            health: Some(HealthWire {
                state: "healthy".to_owned(),
                pid: std::process::id(),
                uptime_ms: started.elapsed().as_millis().min(u128::from(u64::MAX)) as u64,
                capabilities: health_capabilities(),
            }),
            error: None,
            auth_tag_hex: String::new(),
        },
        Ok(()) if request.action == "shutdown" => ResponseWire {
            request_id: request.request_id,
            ok: true,
            health: None,
            error: None,
            auth_tag_hex: String::new(),
        },
        Ok(()) => ResponseWire {
            request_id: request.request_id,
            ok: false,
            health: None,
            error: Some("unsupported agent action".to_owned()),
            auth_tag_hex: String::new(),
        },
        Err(_) => ResponseWire {
            request_id: request.request_id,
            ok: false,
            health: None,
            error: Some("agent request rejected".to_owned()),
            auth_tag_hex: String::new(),
        },
    };

    let mut signed = response;
    signed.auth_tag_hex = sign_hex(session_key, &response_message(&signed))?;
    let encoded = serde_json::to_vec(&signed)
        .map_err(|_| AgentError::Protocol("agent response could not be serialized"))?;
    stream
        .write_all(&encoded)
        .and_then(|_| stream.write_all(b"\n"))
        .map_err(|_| AgentError::Io("agent response could not be written"))?;
    Ok(request.action == "shutdown" && signed.ok)
}

fn health_capabilities() -> Vec<String> {
    let mut capabilities = vec![
        "health".to_owned(),
        "authenticated-ipc".to_owned(),
        "background-lifetime".to_owned(),
        "graceful-shutdown".to_owned(),
        "restartable-session".to_owned(),
        "continuous-integrity-monitoring".to_owned(),
    ];
    #[cfg(windows)]
    capabilities.push("privileged-service-client".to_owned());
    capabilities
}

fn validate_request(
    request: &RequestWire,
    session_key: &[u8],
    replay: &mut ReplayCache,
) -> Result<()> {
    authorize_request(request)?;
    decode_nonce(&request.nonce_b64)?;
    let current = now_ms();
    let skew = current.abs_diff(request.timestamp_ms);
    if skew > MAX_CLOCK_SKEW_MS {
        return Err(AgentError::Authentication(
            "agent request timestamp is stale",
        ));
    }
    verify_hex(
        session_key,
        &request_message(request),
        &request.auth_tag_hex,
    )?;
    if !replay.insert_new(request.nonce_b64.clone()) {
        return Err(AgentError::Authentication(
            "agent request nonce was replayed",
        ));
    }
    Ok(())
}

fn write_runtime_files(paths: &AgentPaths, port: u16, session_key: &[u8]) -> Result<()> {
    let descriptor = RuntimeDescriptor {
        format_version: 1,
        protocol_major: AGENT_PROTOCOL_MAJOR,
        protocol_minor: AGENT_PROTOCOL_MINOR,
        port,
        pid: std::process::id(),
        started_at_ms: now_ms(),
    };
    let descriptor_bytes = serde_json::to_vec_pretty(&descriptor)
        .map_err(|_| AgentError::Protocol("agent runtime descriptor could not be serialized"))?;

    atomic_write(
        &paths.credential_file(),
        format!("{}\n", BASE64.encode(session_key)).as_bytes(),
    )?;
    atomic_write(&paths.runtime_file(), &descriptor_bytes)?;
    Ok(())
}

fn atomic_write(path: &Path, bytes: &[u8]) -> Result<()> {
    let parent = path.parent().ok_or(AgentError::Io(
        "agent runtime parent directory is unavailable",
    ))?;
    fs::create_dir_all(parent)
        .map_err(|_| AgentError::Io("agent runtime directory could not be created"))?;
    let temporary = path.with_extension("tmp");
    let mut file = OpenOptions::new()
        .write(true)
        .create(true)
        .truncate(true)
        .open(&temporary)
        .map_err(|_| AgentError::Io("agent runtime file could not be created"))?;
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        file.set_permissions(fs::Permissions::from_mode(0o600))
            .map_err(|_| AgentError::Io("agent runtime permissions could not be restricted"))?;
    }
    file.write_all(bytes)
        .and_then(|_| file.sync_all())
        .map_err(|_| AgentError::Io("agent runtime file could not be written"))?;
    fs::rename(&temporary, path)
        .map_err(|_| AgentError::Io("agent runtime file could not be finalized"))
}

fn acquire_runtime_lock(paths: &AgentPaths) -> Result<fs::File> {
    acquire_runtime_lock_with_stale_after(paths, Duration::from_secs(5))
}

fn acquire_runtime_lock_with_stale_after(
    paths: &AgentPaths,
    stale_after: Duration,
) -> Result<fs::File> {
    let open_new = || {
        OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(paths.lock_file())
    };
    match open_new() {
        Ok(lock) => Ok(lock),
        Err(_) => {
            if AgentClient::from_paths(paths.clone()).health().is_ok() {
                return Err(AgentError::InvalidState(
                    "DragonForge Agent is already running",
                ));
            }
            let stale = fs::metadata(paths.lock_file())
                .ok()
                .and_then(|metadata| metadata.modified().ok())
                .and_then(|modified| modified.elapsed().ok())
                .is_some_and(|age| age >= stale_after);
            if !stale {
                return Err(AgentError::InvalidState(
                    "DragonForge Agent is already starting",
                ));
            }
            let _ = fs::remove_file(paths.lock_file());
            cleanup_runtime_files(paths);
            open_new().map_err(|_| {
                AgentError::InvalidState("DragonForge Agent runtime lock could not be acquired")
            })
        }
    }
}

fn cleanup_runtime_files(paths: &AgentPaths) {
    let _ = fs::remove_file(paths.runtime_file());
    let _ = fs::remove_file(paths.credential_file());
}

#[cfg(test)]
mod tests {
    use std::thread;
    use std::time::{Duration, Instant};

    use base64::Engine;
    use base64::engine::general_purpose::STANDARD as BASE64;
    use tempfile::tempdir;

    use super::{
        AgentServer, ReplayCache, acquire_runtime_lock_with_stale_after, validate_request,
    };
    use crate::protocol::{
        AGENT_PROTOCOL_MAJOR, AGENT_PROTOCOL_MINOR, NONCE_BYTES, RequestWire, now_ms,
        request_message, sign_hex,
    };

    fn signed_request(key: &[u8], nonce_byte: u8) -> RequestWire {
        let mut request = RequestWire {
            protocol_major: AGENT_PROTOCOL_MAJOR,
            protocol_minor: AGENT_PROTOCOL_MINOR,
            request_id: 7,
            source: "security-center".to_owned(),
            action: "health".to_owned(),
            timestamp_ms: now_ms(),
            nonce_b64: BASE64.encode([nonce_byte; NONCE_BYTES]),
            auth_tag_hex: String::new(),
        };
        request.auth_tag_hex = sign_hex(key, &request_message(&request)).expect("sign");
        request
    }

    #[test]
    fn valid_authenticated_request_is_accepted_once() {
        let key = [9_u8; 32];
        let request = signed_request(&key, 1);
        let mut replay = ReplayCache::default();
        assert!(validate_request(&request, &key, &mut replay).is_ok());
        assert!(validate_request(&request, &key, &mut replay).is_err());
    }

    #[test]
    fn modified_request_authentication_fails() {
        let key = [9_u8; 32];
        let mut request = signed_request(&key, 2);
        request.action = "other".to_owned();
        assert!(validate_request(&request, &key, &mut ReplayCache::default()).is_err());
    }

    #[test]
    fn stale_request_is_rejected() {
        let key = [9_u8; 32];
        let mut request = signed_request(&key, 3);
        request.timestamp_ms = 1;
        request.auth_tag_hex = sign_hex(&key, &request_message(&request)).expect("sign");
        assert!(validate_request(&request, &key, &mut ReplayCache::default()).is_err());
    }

    #[test]
    fn missing_agent_runtime_remains_unavailable() {
        let dir = tempdir().expect("tempdir");
        let paths = crate::AgentPaths::from_root(dir.path());
        assert!(crate::AgentClient::from_paths(paths).health().is_err());
    }

    #[test]
    fn active_runtime_lock_refuses_second_owner() {
        let dir = tempdir().expect("tempdir");
        let paths = crate::AgentPaths::from_root(dir.path());
        std::fs::create_dir_all(paths.root()).expect("runtime dir");
        let first = acquire_runtime_lock_with_stale_after(&paths, Duration::from_secs(60))
            .expect("first lock");
        let second = acquire_runtime_lock_with_stale_after(&paths, Duration::from_secs(60));
        assert!(second.is_err());
        drop(first);
        let _ = std::fs::remove_file(paths.lock_file());
    }

    #[test]
    fn stale_lock_recovery_removes_orphaned_runtime_files() {
        let dir = tempdir().expect("tempdir");
        let paths = crate::AgentPaths::from_root(dir.path());
        std::fs::create_dir_all(paths.root()).expect("runtime dir");
        std::fs::write(paths.lock_file(), b"stale").expect("stale lock");
        std::fs::write(paths.runtime_file(), b"stale").expect("stale runtime");
        std::fs::write(paths.credential_file(), b"stale").expect("stale credential");

        let lock = acquire_runtime_lock_with_stale_after(&paths, Duration::ZERO)
            .expect("reclaim stale lock");
        assert!(!paths.runtime_file().exists());
        assert!(!paths.credential_file().exists());
        drop(lock);
        let _ = std::fs::remove_file(paths.lock_file());
    }

    #[test]
    fn authenticated_health_round_trip_works() {
        let dir = tempdir().expect("tempdir");
        let paths = crate::AgentPaths::from_root(dir.path());
        let server_paths = paths.clone();
        let handle = thread::spawn(move || {
            AgentServer::from_paths(server_paths)
                .run_internal(Some(1))
                .expect("server");
        });

        let deadline = Instant::now() + Duration::from_secs(10);
        while !paths.runtime_file().is_file() {
            assert!(
                Instant::now() < deadline,
                "agent runtime file was not created"
            );
            thread::sleep(Duration::from_millis(10));
        }

        let health = crate::AgentClient::from_paths(paths.clone())
            .health()
            .expect("health");
        assert!(health.available);
        assert_eq!(health.state, "healthy");
        assert!(
            health
                .capabilities
                .iter()
                .any(|item| item == "authenticated-ipc")
        );
        assert!(
            health
                .capabilities
                .iter()
                .any(|item| item == "graceful-shutdown")
        );
        assert!(
            health
                .capabilities
                .iter()
                .any(|item| item == "restartable-session")
        );

        handle.join().expect("server thread");
        assert!(!paths.runtime_file().exists());
        assert!(!paths.credential_file().exists());
        assert!(!paths.lock_file().exists());
    }

    #[test]
    fn authenticated_shutdown_stops_server_and_cleans_runtime() {
        let dir = tempdir().expect("tempdir");
        let paths = crate::AgentPaths::from_root(dir.path());
        let server_paths = paths.clone();
        let handle = thread::spawn(move || {
            AgentServer::from_paths(server_paths)
                .run_internal(None)
                .expect("server");
        });

        let deadline = Instant::now() + Duration::from_secs(10);
        while !paths.runtime_file().is_file() {
            assert!(
                Instant::now() < deadline,
                "agent runtime file was not created"
            );
            thread::sleep(Duration::from_millis(10));
        }

        crate::AgentClient::from_paths(paths.clone())
            .shutdown()
            .expect("authenticated shutdown");

        handle.join().expect("server thread");
        assert!(!paths.runtime_file().exists());
        assert!(!paths.credential_file().exists());
        assert!(!paths.lock_file().exists());
    }
}
