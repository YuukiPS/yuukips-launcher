/*
 * Source: https://github.com/Grasscutters/Cultivation/raw/refs/heads/main/src-tauri/src/proxy.rs
 */

use chrono::Utc;
use once_cell::sync::Lazy;
use serde::{Deserialize, Serialize};
use std::collections::VecDeque;
use std::{path::PathBuf, str::FromStr, sync::Mutex};
use tokio::runtime::Runtime;

use hudsucker::{
    async_trait::async_trait,
    certificate_authority::RcgenAuthority,
    hyper::{Body, Request, Response, StatusCode},
    *,
};
use rcgen::{
    BasicConstraints, CertificateParams, DistinguishedName, DnType, IsCa, KeyPair, KeyUsagePurpose,
};

use std::fs;
use std::net::SocketAddr;
use std::path::Path;
use std::process::Command;

use hudsucker::hyper::Uri;
use rustls_pemfile as pemfile;

use std::env;

// Helper function to get data directory
fn get_data_dir() -> Result<PathBuf, String> {
    if let Some(home) = env::var_os("USERPROFILE") {
        Ok(PathBuf::from(home).join("AppData").join("Local"))
    } else {
        Err("Could not determine data directory".to_string())
    }
}

#[cfg(windows)]
use registry::{Data, Hive, Security};

#[cfg(target_os = "linux")]
use std::process::Command;
#[cfg(target_os = "linux")]
use std::{fs::File, io::Write, process::Command};

// Global ver for getting server address.
static SERVER: Lazy<Mutex<String>> = Lazy::new(|| Mutex::new("https://ps.yuuki.me".to_string()));

// Global proxy state
static PROXY_STATE: Lazy<Mutex<Option<ProxyHandle>>> = Lazy::new(|| Mutex::new(None));

// Global proxy port storage
static PROXY_PORT: Lazy<Mutex<u16>> = Lazy::new(|| Mutex::new(8080));

// Global proxy logs storage
static PROXY_LOGS: Lazy<Mutex<VecDeque<ProxyLogEntry>>> = Lazy::new(|| Mutex::new(VecDeque::new()));

// Global domain list for proxy interception
static PROXY_DOMAINS: Lazy<Mutex<Vec<String>>> = Lazy::new(|| {
    Mutex::new(vec![
        "hoyoverse.com".to_string(),
        "mihoyo.com".to_string(),
        "yuanshen.com".to_string(),
        "starrails.com".to_string(),
        "bhsr.com".to_string(),
        "bh3.com".to_string(),
        "honkaiimpact3.com".to_string(),
        "zenlesszonezero.com".to_string(),
        "yuanshen.com:12401".to_string(),
    ])
});

#[derive(Clone, Serialize, Deserialize)]
pub struct ProxyLogEntry {
    pub timestamp: String,
    pub original_url: String,
    pub redirected_url: String,
}

struct ProxyHandle {
    _runtime: Runtime,
    shutdown_tx: tokio::sync::oneshot::Sender<()>,
}

#[derive(Clone)]
struct ProxyHandler;

#[tauri::command]
pub fn set_proxy_addr(addr: String) {
    if addr.contains(' ') {
        let addr2 = addr.replace(' ', "");
        *SERVER.lock().unwrap() = addr2;
    } else {
        *SERVER.lock().unwrap() = addr;
    }

    println!("Set server to {}", SERVER.lock().unwrap());
}

#[tauri::command]
pub fn get_proxy_addr() -> String {
    SERVER.lock().unwrap().clone()
}

#[tauri::command]
pub fn set_proxy_port(port: u16) -> Result<String, String> {
    if port < 1024 || port >= 65535 {
        return Err("Port must be between 1024 and 65535".to_string());
    }
    
    *PROXY_PORT.lock().unwrap() = port;
    Ok(format!("Proxy port set to {}", port))
}

#[tauri::command]
pub fn get_proxy_port() -> u16 {
    *PROXY_PORT.lock().unwrap()
}

#[tauri::command]
pub fn find_available_port() -> Result<u16, String> {
    use std::net::TcpListener;
    
    // Try to find an available port starting from 8080
    for port in 8080..=8999 {
        if let Ok(_) = TcpListener::bind(format!("127.0.0.1:{}", port)) {
            return Ok(port);
        }
    }
    
    // If no port found in the preferred range, try a random port
    match TcpListener::bind("127.0.0.1:0") {
        Ok(listener) => {
            let port = listener.local_addr().unwrap().port();
            Ok(port)
        }
        Err(_) => Err("Could not find any available port".to_string())
    }
}

#[tauri::command]
pub fn start_proxy_with_port(port: u16) -> Result<String, String> {
    // Set the port first
    set_proxy_port(port)?;
    // Then start the proxy
    start_proxy()
}

// Helper function to log proxy redirections
fn log_proxy_redirection(original_url: String, redirected_url: String) {
    let timestamp = Utc::now().format("%H:%M:%S").to_string();
    let log_entry = ProxyLogEntry {
        timestamp,
        original_url,
        redirected_url,
    };

    if let Ok(mut logs) = PROXY_LOGS.lock() {
        logs.push_back(log_entry);
        // Keep only the last 100 log entries to prevent memory issues
        if logs.len() > 100 {
            logs.pop_front();
        }
    }
}

#[tauri::command]
pub fn get_proxy_logs() -> Vec<ProxyLogEntry> {
    PROXY_LOGS.lock().unwrap().iter().cloned().collect()
}

#[tauri::command]
pub fn clear_proxy_logs() {
    PROXY_LOGS.lock().unwrap().clear();
}

#[tauri::command]
pub fn get_proxy_domains() -> Vec<String> {
    PROXY_DOMAINS.lock().unwrap().clone()
}

#[tauri::command]
pub fn add_proxy_domain(domain: String) -> Result<String, String> {
    let mut domains = PROXY_DOMAINS
        .lock()
        .map_err(|e| format!("Failed to lock domains: {}", e))?;
    let trimmed_domain = domain.trim().to_lowercase();

    if trimmed_domain.is_empty() {
        return Err("Domain cannot be empty".to_string());
    }

    if domains.contains(&trimmed_domain) {
        return Err("Domain already exists in the list".to_string());
    }

    domains.push(trimmed_domain.clone());
    Ok(format!("Domain '{}' added successfully", trimmed_domain))
}

#[tauri::command]
pub fn remove_proxy_domain(domain: String) -> Result<String, String> {
    let mut domains = PROXY_DOMAINS
        .lock()
        .map_err(|e| format!("Failed to lock domains: {}", e))?;
    let trimmed_domain = domain.trim().to_lowercase();

    if let Some(pos) = domains.iter().position(|d| d == &trimmed_domain) {
        domains.remove(pos);
        Ok(format!("Domain '{}' removed successfully", trimmed_domain))
    } else {
        Err("Domain not found in the list".to_string())
    }
}

// Helper function to check if URI should be intercepted based on domain list
fn should_intercept_uri(uri: &str) -> bool {
    if let Ok(domains) = PROXY_DOMAINS.lock() {
        for domain in domains.iter() {
            if uri.contains(domain) {
                return true;
            }
        }
    }
    false
}

#[async_trait]
impl HttpHandler for ProxyHandler {
    async fn handle_request(
        &mut self,
        _ctx: &HttpContext,
        mut req: Request<Body>,
    ) -> RequestOrResponse {
        let uri = req.uri().to_string();

        if should_intercept_uri(&uri) {
            // Handle CONNECTs
            if req.method().as_str() == "CONNECT" {
                let builder = Response::builder()
                    .header("DecryptEndpoint", "Created")
                    .status(StatusCode::OK);
                let res = builder.body(()).unwrap();

                // Respond to CONNECT
                *res.body()
            } else {
                let uri_path_and_query = req.uri().path_and_query().unwrap().as_str();
                let original_uri = req.uri().to_string();
                // Create new URI.
                let new_uri = Uri::from_str(
                    format!("{}{}", SERVER.lock().unwrap(), uri_path_and_query).as_str(),
                )
                .unwrap();

                // Log the proxy redirection
                log_proxy_redirection(original_uri, new_uri.to_string());

                // Set request URI to the new one.
                *req.uri_mut() = new_uri;
            }
        }

        req.into()
    }

    async fn handle_response(
        &mut self,
        _context: &HttpContext,
        response: Response<Body>,
    ) -> Response<Body> {
        response
    }

    async fn should_intercept(&mut self, _ctx: &HttpContext, _req: &Request<Body>) -> bool {
        let uri = _req.uri().to_string();
        should_intercept_uri(&uri)
    }
}

/**
 * Starts an HTTP(S) proxy server.
 */
pub async fn create_proxy_internal(
    proxy_port: u16,
    certificate_path: String,
    shutdown_rx: tokio::sync::oneshot::Receiver<()>,
) {
    let cert_path = PathBuf::from(certificate_path);
    let pk_path = cert_path.join("private.key");
    let ca_path = cert_path.join("cert.crt");

    // Get the certificate and private key.
    let mut private_key_bytes: &[u8] = &match fs::read(&pk_path) {
        // Try regenerating the CA stuff and read it again. If that doesn't work, quit.
        Ok(b) => b,
        Err(e) => {
            println!("Encountered {}. Regenerating CA cert and retrying...", e);
            generate_ca_files(&get_data_dir().unwrap().join("yuukips"));

            fs::read(&pk_path).expect("Could not read private key")
        }
    };

    let mut ca_cert_bytes: &[u8] = &match fs::read(&ca_path) {
        // Try regenerating the CA stuff and read it again. If that doesn't work, quit.
        Ok(b) => b,
        Err(e) => {
            println!("Encountered {}. Regenerating CA cert and retrying...", e);
            generate_ca_files(&get_data_dir().unwrap().join("yuukips"));

            fs::read(&ca_path).expect("Could not read certificate")
        }
    };

    // Parse the private key and certificate.
    let private_key_der = pemfile::pkcs8_private_keys(&mut private_key_bytes)
        .expect("Failed to parse private key")
        .into_iter()
        .next()
        .expect("No private key found");
    let private_key = rustls::PrivateKey(private_key_der);

    let ca_cert_der = pemfile::certs(&mut ca_cert_bytes)
        .expect("Failed to parse CA certificate")
        .into_iter()
        .next()
        .expect("No certificate found");
    let ca_cert = rustls::Certificate(ca_cert_der);

    // Create the certificate authority.
    let authority = RcgenAuthority::new(private_key, ca_cert, 1_000)
        .expect("Failed to create Certificate Authority");

    // Create an instance of the proxy.
    let proxy = ProxyBuilder::new()
        .with_addr(SocketAddr::from(([0, 0, 0, 0], proxy_port)))
        .with_rustls_client()
        .with_ca(authority)
        .with_http_handler(ProxyHandler)
        .build();

    // Start the proxy.
    let shutdown_signal = async {
        shutdown_rx.await.ok();
    };

    proxy.start(shutdown_signal).await.ok();
}

/**
 * Connects to the local HTTP(S) proxy server.
 */
#[cfg(windows)]
pub fn connect_to_proxy(proxy_port: u16) {
    // Create 'ProxyServer' string.
    let server_string: String = format!(
        "http=127.0.0.1:{};https=127.0.0.1:{}",
        proxy_port, proxy_port
    );

    // Fetch the 'Internet Settings' registry key.
    let settings = Hive::CurrentUser
        .open(
            r"Software\Microsoft\Windows\CurrentVersion\Internet Settings",
            // Only write should be needed but too many cases of Culti not being able to read/write proxy settings
            Security::AllAccess,
        )
        .unwrap();

    // Set registry values.
    settings
        .set_value("ProxyServer", &Data::String(server_string.parse().unwrap()))
        .unwrap();
    settings.set_value("ProxyEnable", &Data::U32(1)).unwrap();

    println!("Connected to the proxy.");
}

#[cfg(target_os = "linux")]
pub fn connect_to_proxy(proxy_port: u16) {
    let mut config = Config::get().unwrap();
    let proxy_addr = format!("127.0.0.1:{}", proxy_port);
    if !config.game.environment.contains_key("http_proxy") {
        config
            .game
            .environment
            .insert("http_proxy".to_string(), proxy_addr.clone());
    }
    if !config.game.environment.contains_key("https_proxy") {
        config
            .game
            .environment
            .insert("https_proxy".to_string(), proxy_addr);
    }
    Config::update(config);
}

#[cfg(target_os = "macos")]
pub fn connect_to_proxy(_proxy_port: u16) {
    println!("No Mac support yet. Someone mail me a Macbook and I will do it B)")
}

/**
 * Disconnects from the local HTTP(S) proxy server.
 */
#[cfg(windows)]
pub fn disconnect_from_proxy() {
    // Fetch the 'Internet Settings' registry key.
    let settings = Hive::CurrentUser
        .open(
            r"Software\Microsoft\Windows\CurrentVersion\Internet Settings",
            Security::AllAccess,
        )
        .unwrap();

    // Set registry values.
    settings.set_value("ProxyEnable", &Data::U32(0)).unwrap();

    println!("Disconnected from proxy.");
}

#[cfg(target_os = "linux")]
pub fn disconnect_from_proxy() {
    let mut config = Config::get().unwrap();
    if config.game.environment.contains_key("http_proxy") {
        config.game.environment.remove("http_proxy");
    }
    if config.game.environment.contains_key("https_proxy") {
        config.game.environment.remove("https_proxy");
    }
    Config::update(config);
}

#[cfg(target_os = "macos")]
pub fn disconnect_from_proxy() {}

/*
 * Generates a private key and certificate used by the certificate authority.
 * Additionally installs the certificate and private key in the Root CA store.
 * Source: https://github.com/zu1k/good-mitm/raw/master/src/ca/gen.rs
 */
#[tauri::command]
pub fn generate_ca_files(path: &Path) {
    let mut params = CertificateParams::default();
    let mut details = DistinguishedName::new();

    // Set certificate details.
    details.push(DnType::CommonName, "YuukiPS");
    details.push(DnType::OrganizationName, "Yuuki");
    details.push(DnType::CountryName, "ID");
    details.push(DnType::LocalityName, "ID");

    // Set details in the parameter.
    params.distinguished_name = details;
    // Set other properties.
    params.is_ca = IsCa::Ca(BasicConstraints::Unconstrained);
    params.key_usages = vec![
        KeyUsagePurpose::DigitalSignature,
        KeyUsagePurpose::KeyCertSign,
        KeyUsagePurpose::CrlSign,
    ];

    // Create certificate.
    let key_pair = KeyPair::generate().unwrap();
    let cert = params.self_signed(&key_pair).unwrap();
    let cert_crt = cert.pem();
    let private_key = key_pair.serialize_pem();

    // Make certificate directory.
    let cert_dir = path.join("ca");
    match fs::create_dir_all(&cert_dir) {
        Ok(_) => {}
        Err(e) => {
            println!("{}", e);
        }
    };

    // Write the certificate to a file.
    let cert_path = cert_dir.join("cert.crt");
    match fs::write(&cert_path, cert_crt) {
        Ok(_) => println!("Wrote certificate to {}", cert_path.to_str().unwrap()),
        Err(e) => println!(
            "Error writing certificate to {}: {}",
            cert_path.to_str().unwrap(),
            e
        ),
    }

    // Write the private key to a file.
    let private_key_path = cert_dir.join("private.key");
    match fs::write(&private_key_path, private_key) {
        Ok(_) => println!(
            "Wrote private key to {}",
            private_key_path.to_str().unwrap()
        ),
        Err(e) => println!(
            "Error writing private key to {}: {}",
            private_key_path.to_str().unwrap(),
            e
        ),
    }

    // Install certificate into the system's Root CA store.
    install_ca_files(&cert_path);
}

/*
 * Attempts to install the certificate authority's certificate into the Root CA store.
 */
#[cfg(windows)]
pub fn install_ca_files(cert_path: &Path) {
    Command::new("certutil")
        .args(["-addstore", "-f", "Root", &cert_path.to_string_lossy()])
        .output()
        .expect("Failed to install certificate");
    println!("Installed certificate.");
}

#[cfg(target_os = "macos")]
pub fn install_ca_files(cert_path: &Path) {
    Command::new("security")
        .args([
            "add-trusted-cert",
            "-d",
            "-r",
            "trustRoot",
            "-k",
            "/Library/Keychains/System.keychain",
            cert_path.to_str().unwrap(),
        ])
        .output()
        .expect("Failed to install certificate");
    println!("Installed certificate.");
}

#[cfg(target_os = "linux")]
pub fn install_ca_files(cert_path: &Path) {
    // Create a script to install the certificate.
    let script = Path::new("/tmp/yuukips-inject-ca-cert.sh");
    let mut file = File::create(script).expect("Failed to create script");

    // Write the script.
    file.write_all(
        format!(
            r#"#!/bin/bash

set -e

if [ -d /etc/ca-certificates/trust-source/anchors ]; then
  # Arch, Manjaro, etc.
  cp {} /etc/ca-certificates/trust-source/anchors/yuukips-ca.crt
  trust extract-compat
elif [ -d /usr/local/share/ca-certificates ]; then
  # Debian, Ubuntu, etc.
  cp {} /usr/local/share/ca-certificates/yuukips-ca.crt
  update-ca-certificates
elif [ -d /etc/pki/ca-trust/source/anchors ]; then
  # Fedora, RHEL, etc.
  cp {} /etc/pki/ca-trust/source/anchors/yuukips-ca.crt
  update-ca-trust
fi
"#,
            cert_path.to_string_lossy(),
            cert_path.to_string_lossy(),
            cert_path.to_string_lossy()
        )
        .as_bytes(),
    )
    .expect("Failed to write script");

    // Make the script executable.
    Command::new("chmod")
        .args(["a+x", script.to_str().unwrap()])
        .output()
        .expect("Failed to make script executable");

    // Run the script as root.
    Command::new("pkexec")
        .args([script.to_str().unwrap()])
        .output()
        .expect("Failed to run script");

    println!("Installed certificate.");
}

// Additional functions required by lib.rs

pub fn start_proxy() -> Result<String, String> {
    let mut state = PROXY_STATE
        .lock()
        .map_err(|e| format!("Failed to lock proxy state: {}", e))?;

    // If proxy is already running, stop it first
    if let Some(handle) = state.take() {
        let _ = handle.shutdown_tx.send(());
        disconnect_from_proxy();
    }

    let runtime = Runtime::new().map_err(|e| format!("Failed to create runtime: {}", e))?;
    let (shutdown_tx, shutdown_rx) = tokio::sync::oneshot::channel();

    let cert_path = get_data_dir()
        .unwrap()
        .join("yuukips")
        .join("ca")
        .to_string_lossy()
        .to_string();

    let proxy_port = *PROXY_PORT.lock().unwrap();
    runtime.spawn(create_proxy_internal(proxy_port, cert_path, shutdown_rx));

    *state = Some(ProxyHandle {
        _runtime: runtime,
        shutdown_tx,
    });

    // Re-establish proxy connection after starting
    connect_to_proxy(proxy_port);

    Ok(format!("Proxy started successfully on port {}", proxy_port))
}

pub fn stop_proxy() -> Result<String, String> {
    let mut state = PROXY_STATE
        .lock()
        .map_err(|e| format!("Failed to lock proxy state: {}", e))?;

    if let Some(handle) = state.take() {
        let _ = handle.shutdown_tx.send(());
        disconnect_from_proxy();
        Ok("Proxy stopped successfully".to_string())
    } else {
        Err("Proxy is not running".to_string())
    }
}

pub fn is_proxy_running() -> bool {
    PROXY_STATE
        .lock()
        .map(|state| state.is_some())
        .unwrap_or(false)
}

pub fn force_stop_proxy() -> Result<String, String> {
    let mut state = PROXY_STATE
        .lock()
        .map_err(|e| format!("Failed to lock proxy state: {}", e))?;

    if let Some(handle) = state.take() {
        let _ = handle.shutdown_tx.send(());
        disconnect_from_proxy();
        Ok("Proxy force stopped successfully".to_string())
    } else {
        Ok("Proxy was not running".to_string())
    }
}

pub fn check_and_disable_windows_proxy() -> Result<String, String> {
    #[cfg(windows)]
    {
        disconnect_from_proxy();
        Ok("Windows proxy settings checked and disabled if necessary".to_string())
    }
    #[cfg(not(windows))]
    {
        Ok("Proxy check not needed on this platform".to_string())
    }
}

pub fn install_ca_certificate() -> Result<String, String> {
    let cert_path = get_data_dir().unwrap().join("yuukips");
    generate_ca_files(&cert_path);
    install_ca_files(&cert_path);
    Ok("CA certificate installed successfully".to_string())
}

pub fn get_certificate_path() -> Result<String, String> {
    let cert_path = get_data_dir().unwrap().join("yuukips").join("cert.crt");
    Ok(cert_path.to_string_lossy().to_string())
}

#[cfg(not(any(target_os = "windows", target_os = "macos", target_os = "linux")))]
pub fn install_ca_files(_cert_path: &Path) {
    println!("CA certificate installation is not supported on this platform.");
}
