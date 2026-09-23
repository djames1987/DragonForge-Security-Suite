//! Windows SCM and named-pipe host for the DragonForge privileged service.
//!
//! This module is the only Phase 17 location allowed to use unsafe Windows FFI.
//! Unsafe calls are limited to pipe ACL construction, named-pipe peer identity,
//! executable-path discovery, and Authenticode signer verification.

use std::env;
use std::ffi::{c_void, OsString};
use std::fs;
use std::mem::{size_of, zeroed};
use std::os::windows::ffi::OsStrExt;
use std::os::windows::io::AsRawHandle;
use std::path::{Path, PathBuf};
use std::ptr::{null, null_mut};
use std::sync::{
    Arc, Mutex,
    atomic::{AtomicBool, Ordering},
};
use std::time::{Duration, Instant};

use dragonforge_windows_boundary::{
    BoundaryPolicy, CallerIdentity, EXPECTED_AGENT_EXE, MAX_MESSAGE_BYTES, PIPE_NAME,
    PrivilegedPolicyDescription, PrivilegedRequest, PrivilegedResponse, PrivilegedServiceHealth,
    SERVICE_ACCOUNT, SERVICE_NAME, ServiceCommand,
};
use tokio::io::{AsyncBufReadExt, AsyncReadExt, AsyncWriteExt, BufReader};
use tokio::net::windows::named_pipe::{NamedPipeServer, PipeMode, ServerOptions};
use tokio::time::timeout;
use windows_service::{
    define_windows_service,
    service::{
        ServiceControl, ServiceControlAccept, ServiceExitCode, ServiceState, ServiceStatus,
        ServiceType,
    },
    service_control_handler::{self, ServiceControlHandlerResult},
    service_dispatcher,
};
use windows_sys::Win32::{
    Foundation::{CloseHandle, HANDLE},
    Security::{
        Authorization::{
            ConvertStringSecurityDescriptorToSecurityDescriptorW, SDDL_REVISION_1,
        },
        Cryptography::{CertNameToStrW, CERT_X500_NAME_STR, X509_ASN_ENCODING},
        WinTrust::{
            WinVerifyTrust, WTHelperGetProvCertFromChain, WTHelperGetProvSignerFromChain,
            WTHelperProvDataFromStateData, WINTRUST_ACTION_GENERIC_VERIFY_V2, WINTRUST_DATA,
            WINTRUST_DATA_0, WINTRUST_FILE_INFO, WTD_CHOICE_FILE, WTD_REVOKE_WHOLECHAIN,
            WTD_STATEACTION_CLOSE, WTD_STATEACTION_VERIFY, WTD_UI_NONE,
        },
        PSECURITY_DESCRIPTOR, SECURITY_ATTRIBUTES,
    },
    System::{
        Memory::LocalFree,
        Pipes::GetNamedPipeClientProcessId,
        Threading::{
            OpenProcess, QueryFullProcessImageNameW, PROCESS_QUERY_LIMITED_INFORMATION,
        },
    },
};

use crate::{
    AbuseGuard, AuditLogger, Result, ServiceConfig, ServiceError, load_config, rejected_response,
};

const SERVICE_TYPE: ServiceType = ServiceType::OWN_PROCESS;
const PIPE_DACL_SDDL: &str =
    "D:P(D;;GA;;;AN)(D;;GA;;;NU)(A;;GA;;;SY)(A;;GA;;;BA)(A;;GRGW;;;IU)";
const CONNECT_POLL: Duration = Duration::from_millis(500);
const REQUEST_TIMEOUT: Duration = Duration::from_secs(2);
const RESPONSE_TIMEOUT: Duration = Duration::from_secs(2);

define_windows_service!(ffi_service_main, service_main);

pub fn run_dispatcher() -> windows_service::Result<()> {
    service_dispatcher::start(SERVICE_NAME, ffi_service_main)
}

fn service_main(_arguments: Vec<OsString>) {
    let _ = run_service();
}

fn run_service() -> Result<()> {
    let stop = Arc::new(AtomicBool::new(false));
    let stop_for_handler = Arc::clone(&stop);

    let event_handler = move |control_event| -> ServiceControlHandlerResult {
        match control_event {
            ServiceControl::Stop | ServiceControl::Shutdown => {
                stop_for_handler.store(true, Ordering::Release);
                ServiceControlHandlerResult::NoError
            }
            ServiceControl::Interrogate => ServiceControlHandlerResult::NoError,
            _ => ServiceControlHandlerResult::NotImplemented,
        }
    };

    let status_handle = service_control_handler::register(SERVICE_NAME, event_handler)
        .map_err(|_| ServiceError::Platform)?;

    status_handle
        .set_service_status(ServiceStatus {
            service_type: SERVICE_TYPE,
            current_state: ServiceState::StartPending,
            controls_accepted: ServiceControlAccept::empty(),
            exit_code: ServiceExitCode::Win32(0),
            checkpoint: 1,
            wait_hint: Duration::from_secs(10),
            process_id: None,
        })
        .map_err(|_| ServiceError::Platform)?;

    let paths = ServicePaths::discover()?;
    let config = load_config(&paths.config_file)?;
    let install_dir = current_install_directory()?;
    let policy = BoundaryPolicy::new(&install_dir, config.expected_publisher_subject.clone());
    if !policy.expected_agent_path().ends_with(EXPECTED_AGENT_EXE) {
        return Err(ServiceError::InvalidConfiguration);
    }

    let audit = AuditLogger::new(&paths.audit_file, config.audit_max_bytes);
    let _ = audit.lifecycle("service-start", "starting");

    status_handle
        .set_service_status(ServiceStatus {
            service_type: SERVICE_TYPE,
            current_state: ServiceState::Running,
            controls_accepted: ServiceControlAccept::STOP | ServiceControlAccept::SHUTDOWN,
            exit_code: ServiceExitCode::Win32(0),
            checkpoint: 0,
            wait_hint: Duration::default(),
            process_id: None,
        })
        .map_err(|_| ServiceError::Platform)?;
    let _ = audit.lifecycle("service-start", "running");

    let runtime = tokio::runtime::Builder::new_current_thread()
        .enable_io()
        .enable_time()
        .build()
        .map_err(|_| ServiceError::Platform)?;

    let server_result = runtime.block_on(run_pipe_server(
        Arc::clone(&stop),
        policy,
        config,
        audit.clone(),
    ));

    let _ = audit.lifecycle(
        "service-stop",
        if server_result.is_ok() { "requested" } else { "error" },
    );

    status_handle
        .set_service_status(ServiceStatus {
            service_type: SERVICE_TYPE,
            current_state: ServiceState::Stopped,
            controls_accepted: ServiceControlAccept::empty(),
            exit_code: if server_result.is_ok() {
                ServiceExitCode::Win32(0)
            } else {
                ServiceExitCode::ServiceSpecific(1)
            },
            checkpoint: 0,
            wait_hint: Duration::default(),
            process_id: None,
        })
        .map_err(|_| ServiceError::Platform)?;

    server_result
}

async fn run_pipe_server(
    stop: Arc<AtomicBool>,
    policy: BoundaryPolicy,
    config: ServiceConfig,
    audit: AuditLogger,
) -> Result<()> {
    let started = Instant::now();
    let guard = Arc::new(Mutex::new(AbuseGuard::new(
        config.max_requests_per_minute,
    )));

    while !stop.load(Ordering::Acquire) {
        let mut pipe = create_secured_pipe()?;
        match timeout(CONNECT_POLL, pipe.connect()).await {
            Ok(Ok(())) => {
                let response = handle_connected_client(
                    &mut pipe,
                    &policy,
                    &audit,
                    Arc::clone(&guard),
                    started,
                )
                .await;

                if let Ok(response) = response {
                    let _ = write_response(&mut pipe, &response).await;
                }
                let _ = pipe.disconnect();
            }
            Ok(Err(_)) => return Err(ServiceError::Io),
            Err(_) => {}
        }
    }

    Ok(())
}

async fn handle_connected_client(
    pipe: &mut NamedPipeServer,
    policy: &BoundaryPolicy,
    audit: &AuditLogger,
    guard: Arc<Mutex<AbuseGuard>>,
    started: Instant,
) -> Result<PrivilegedResponse> {
    let request = read_request(pipe).await?;
    let request_id = request.request_id;

    let peer = match verify_connected_peer(pipe) {
        Ok(peer) => peer,
        Err(_) => {
            let _ = audit.lifecycle("peer-rejected", "identity");
            return Ok(rejected_response(request_id, "unauthorized"));
        }
    };

    if policy.authorize_caller(&peer.identity).is_err() {
        let _ = audit.request(peer.pid, &request.action, "caller-denied");
        return Ok(rejected_response(request_id, "unauthorized"));
    }

    {
        let mut abuse = guard.lock().map_err(|_| ServiceError::Platform)?;
        if abuse.authorize(peer.pid, &request).is_err() {
            let _ = audit.request(peer.pid, &request.action, "abuse-denied");
            return Ok(rejected_response(request_id, "rate-or-replay-denied"));
        }
    }

    let command = match policy.authorize_command(&request.action) {
        Ok(command) => command,
        Err(_) => {
            let _ = audit.request(peer.pid, &request.action, "command-denied");
            return Ok(rejected_response(request_id, "command-denied"));
        }
    };

    let response = match command {
        ServiceCommand::Health => PrivilegedResponse {
            request_id,
            ok: true,
            code: "ok".to_owned(),
            health: Some(PrivilegedServiceHealth {
                state: "healthy".to_owned(),
                pid: std::process::id(),
                uptime_ms: started.elapsed().as_millis().min(u128::from(u64::MAX)) as u64,
                service_name: SERVICE_NAME.to_owned(),
                service_account: SERVICE_ACCOUNT.to_owned(),
                privileged_capabilities_enabled: false,
            }),
            policy: None,
        },
        ServiceCommand::DescribePolicy => PrivilegedResponse {
            request_id,
            ok: true,
            code: "ok".to_owned(),
            health: None,
            policy: Some(PrivilegedPolicyDescription {
                allowed_commands: vec![
                    ServiceCommand::Health.as_str().to_owned(),
                    ServiceCommand::DescribePolicy.as_str().to_owned(),
                ],
                privileged_capabilities_enabled: false,
                arbitrary_command_execution_prohibited: true,
                generic_shell_execution_prohibited: true,
                max_message_bytes: MAX_MESSAGE_BYTES,
            }),
        },
    };

    let _ = audit.request(peer.pid, &request.action, "allowed");
    Ok(response)
}

async fn read_request(pipe: &mut NamedPipeServer) -> Result<PrivilegedRequest> {
    let mut bytes = Vec::new();
    let limited = (&mut *pipe).take((MAX_MESSAGE_BYTES + 1) as u64);
    let mut reader = BufReader::new(limited);
    let count = timeout(REQUEST_TIMEOUT, reader.read_until(b'\n', &mut bytes))
        .await
        .map_err(|_| ServiceError::RequestRejected)?
        .map_err(|_| ServiceError::Io)?;

    if count == 0
        || bytes.len() > MAX_MESSAGE_BYTES
        || bytes.last().copied() != Some(b'\n')
    {
        return Err(ServiceError::RequestRejected);
    }
    bytes.pop();

    serde_json::from_slice(&bytes).map_err(|_| ServiceError::RequestRejected)
}

async fn write_response(pipe: &mut NamedPipeServer, response: &PrivilegedResponse) -> Result<()> {
    let mut encoded = serde_json::to_vec(response).map_err(|_| ServiceError::Io)?;
    if encoded.len() > MAX_MESSAGE_BYTES {
        return Err(ServiceError::Io);
    }
    encoded.push(b'\n');
    timeout(RESPONSE_TIMEOUT, pipe.write_all(&encoded))
        .await
        .map_err(|_| ServiceError::Io)?
        .map_err(|_| ServiceError::Io)?;
    timeout(RESPONSE_TIMEOUT, pipe.flush())
        .await
        .map_err(|_| ServiceError::Io)?
        .map_err(|_| ServiceError::Io)
}

#[derive(Debug)]
struct VerifiedPeer {
    pid: u32,
    identity: CallerIdentity,
}

fn verify_connected_peer(pipe: &NamedPipeServer) -> Result<VerifiedPeer> {
    let pipe_handle = pipe.as_raw_handle() as HANDLE;
    let pid = client_process_id(pipe_handle)?;
    let executable_path = process_image_path(pid)?;
    let publisher_subject = verify_authenticode_and_subject(&executable_path)?;

    Ok(VerifiedPeer {
        pid,
        identity: CallerIdentity {
            executable_path,
            authenticode_valid: true,
            publisher_subject: Some(publisher_subject),
            operating_system_peer_verified: true,
        },
    })
}

fn client_process_id(pipe_handle: HANDLE) -> Result<u32> {
    let mut pid = 0_u32;
    let ok = unsafe {
        // SAFETY: pipe_handle is a live server handle owned by NamedPipeServer.
        GetNamedPipeClientProcessId(pipe_handle, &mut pid)
    };
    if ok == 0 || pid == 0 {
        return Err(ServiceError::Platform);
    }
    Ok(pid)
}

fn process_image_path(pid: u32) -> Result<PathBuf> {
    let process = unsafe {
        // SAFETY: request only PROCESS_QUERY_LIMITED_INFORMATION for the peer PID.
        OpenProcess(PROCESS_QUERY_LIMITED_INFORMATION, 0, pid)
    };
    if process == 0 {
        return Err(ServiceError::Platform);
    }

    let result = (|| {
        let mut buffer = vec![0_u16; 32_768];
        let mut length = u32::try_from(buffer.len()).map_err(|_| ServiceError::Platform)?;
        let ok = unsafe {
            // SAFETY: process is live and buffer is writable for length UTF-16 units.
            QueryFullProcessImageNameW(process, 0, buffer.as_mut_ptr(), &mut length)
        };
        if ok == 0 || length == 0 {
            return Err(ServiceError::Platform);
        }
        buffer.truncate(length as usize);
        Ok(PathBuf::from(String::from_utf16_lossy(&buffer)))
    })();

    unsafe {
        // SAFETY: process is the handle returned by OpenProcess above.
        CloseHandle(process);
    }
    result
}

fn verify_authenticode_and_subject(path: &Path) -> Result<String> {
    let mut path_wide: Vec<u16> = path.as_os_str().encode_wide().collect();
    path_wide.push(0);

    let mut file_info: WINTRUST_FILE_INFO = unsafe {
        // SAFETY: zero is the documented initialization state for this WinTrust structure.
        zeroed()
    };
    file_info.cbStruct = size_of::<WINTRUST_FILE_INFO>() as u32;
    file_info.pcwszFilePath = path_wide.as_ptr();
    file_info.hFile = 0;
    file_info.pgKnownSubject = null();

    let mut trust_data: WINTRUST_DATA = unsafe {
        // SAFETY: zero is the documented initialization state for this WinTrust structure.
        zeroed()
    };
    trust_data.cbStruct = size_of::<WINTRUST_DATA>() as u32;
    trust_data.dwUIChoice = WTD_UI_NONE;
    trust_data.fdwRevocationChecks = WTD_REVOKE_WHOLECHAIN;
    trust_data.dwUnionChoice = WTD_CHOICE_FILE;
    trust_data.Anonymous = WINTRUST_DATA_0 {
        pFile: &mut file_info,
    };
    trust_data.dwStateAction = WTD_STATEACTION_VERIFY;

    let mut action = WINTRUST_ACTION_GENERIC_VERIFY_V2;
    let status = unsafe {
        // SAFETY: WinTrust pointers reference live stack data and a NUL-terminated path.
        WinVerifyTrust(
            0,
            &mut action,
            &mut trust_data as *mut WINTRUST_DATA as *mut c_void,
        )
    };
    if status != 0 {
        close_wintrust_state(&mut action, &mut trust_data);
        return Err(ServiceError::Platform);
    }

    let subject = extract_signer_subject(&trust_data);
    close_wintrust_state(&mut action, &mut trust_data);
    subject
}

fn extract_signer_subject(trust_data: &WINTRUST_DATA) -> Result<String> {
    let provider = unsafe {
        // SAFETY: hWVTStateData belongs to the successful WinVerifyTrust state.
        WTHelperProvDataFromStateData(trust_data.hWVTStateData)
    };
    if provider.is_null() {
        return Err(ServiceError::Platform);
    }

    let signer = unsafe {
        // SAFETY: provider is a live WinTrust provider and index zero is the primary signer.
        WTHelperGetProvSignerFromChain(provider, 0, 0, 0)
    };
    if signer.is_null() {
        return Err(ServiceError::Platform);
    }

    let cert = unsafe {
        // SAFETY: signer is live and certificate index zero is the leaf signing cert.
        WTHelperGetProvCertFromChain(signer, 0)
    };
    if cert.is_null() {
        return Err(ServiceError::Platform);
    }

    let cert_context = unsafe {
        // SAFETY: cert remains owned by the active WinTrust state.
        (*cert).pCert
    };
    if cert_context.is_null() {
        return Err(ServiceError::Platform);
    }

    let cert_info = unsafe {
        // SAFETY: certificate context is live and pCertInfo is owned by it.
        (*cert_context).pCertInfo
    };
    if cert_info.is_null() {
        return Err(ServiceError::Platform);
    }

    let subject = unsafe {
        // SAFETY: cert_info is live while the WinTrust state is open.
        &(*cert_info).Subject
    };
    let length = unsafe {
        // SAFETY: subject is a valid CERT_NAME_BLOB owned by the certificate.
        CertNameToStrW(
            X509_ASN_ENCODING,
            subject,
            CERT_X500_NAME_STR,
            null_mut(),
            0,
        )
    };
    if length <= 1 || length > 513 {
        return Err(ServiceError::Platform);
    }

    let mut buffer = vec![0_u16; length as usize];
    let written = unsafe {
        // SAFETY: buffer contains length writable UTF-16 code units.
        CertNameToStrW(
            X509_ASN_ENCODING,
            subject,
            CERT_X500_NAME_STR,
            buffer.as_mut_ptr(),
            length,
        )
    };
    if written != length {
        return Err(ServiceError::Platform);
    }

    buffer.truncate((written - 1) as usize);
    let subject = String::from_utf16_lossy(&buffer);
    if subject.trim().is_empty() {
        return Err(ServiceError::Platform);
    }
    Ok(subject)
}

fn close_wintrust_state(
    action: &mut windows_sys::core::GUID,
    trust_data: &mut WINTRUST_DATA,
) {
    trust_data.dwStateAction = WTD_STATEACTION_CLOSE;
    unsafe {
        // SAFETY: closes the WinTrust state opened by WinVerifyTrust.
        let _ = WinVerifyTrust(
            0,
            action,
            trust_data as *mut WINTRUST_DATA as *mut c_void,
        );
    }
}

fn create_secured_pipe() -> Result<NamedPipeServer> {
    let sddl_wide = wide_null(PIPE_DACL_SDDL);
    let mut descriptor: PSECURITY_DESCRIPTOR = null_mut();

    let converted = unsafe {
        // SAFETY: SDDL is NUL-terminated and descriptor is a valid output pointer.
        ConvertStringSecurityDescriptorToSecurityDescriptorW(
            sddl_wide.as_ptr(),
            SDDL_REVISION_1,
            &mut descriptor,
            null_mut(),
        )
    };
    if converted == 0 || descriptor.is_null() {
        return Err(ServiceError::Platform);
    }

    let mut attributes = SECURITY_ATTRIBUTES {
        nLength: size_of::<SECURITY_ATTRIBUTES>() as u32,
        lpSecurityDescriptor: descriptor,
        bInheritHandle: 0,
    };

    let created = unsafe {
        // SAFETY: attributes and descriptor remain valid for CreateNamedPipe.
        ServerOptions::new()
            .pipe_mode(PipeMode::Message)
            .reject_remote_clients(true)
            .max_instances(4)
            .create_with_security_attributes_raw(
                PIPE_NAME,
                &mut attributes as *mut SECURITY_ATTRIBUTES as *mut c_void,
            )
    };

    unsafe {
        // SAFETY: descriptor was allocated by the SDDL conversion API.
        let _ = LocalFree(descriptor as isize);
    }

    created.map_err(|_| ServiceError::Io)
}

fn wide_null(value: &str) -> Vec<u16> {
    value.encode_utf16().chain(std::iter::once(0)).collect()
}

fn current_install_directory() -> Result<PathBuf> {
    let executable = env::current_exe().map_err(|_| ServiceError::Platform)?;
    executable
        .parent()
        .map(Path::to_path_buf)
        .ok_or(ServiceError::Platform)
}

#[derive(Debug)]
struct ServicePaths {
    config_file: PathBuf,
    audit_file: PathBuf,
}

impl ServicePaths {
    fn discover() -> Result<Self> {
        let program_data = env::var_os("PROGRAMDATA")
            .map(PathBuf::from)
            .ok_or(ServiceError::InvalidConfiguration)?;
        let root = program_data
            .join("DragonForge")
            .join("Security")
            .join("privileged-service");
        fs::create_dir_all(&root).map_err(|_| ServiceError::Io)?;
        Ok(Self {
            config_file: root.join("service-config.json"),
            audit_file: root.join("audit.jsonl"),
        })
    }
}
