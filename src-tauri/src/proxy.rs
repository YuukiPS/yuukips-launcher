use std::net::SocketAddr;
use std::sync::Mutex;
use tokio::runtime::Runtime;
use hudsucker::{
    certificate_authority::RcgenAuthority,
    hyper::{Body, Request, Response, StatusCode},
    *,
};
use tracing::info;

#[cfg(target_os = "windows")]
use winreg::enums::*;
#[cfg(target_os = "windows")]
use winreg::RegKey;

// Global proxy state
static PROXY_STATE: Mutex<Option<ProxyHandle>> = Mutex::new(None);

struct ProxyHandle {
    _runtime: Runtime,
    shutdown_tx: tokio::sync::oneshot::Sender<()>,
    original_proxy_settings: Option<WindowsProxySettings>,
}

#[cfg(target_os = "windows")]
#[derive(Clone, Debug)]
struct WindowsProxySettings {
    proxy_enable: u32,
    proxy_server: String,
    proxy_override: String,
}

#[cfg(not(target_os = "windows"))]
#[derive(Clone, Debug)]
struct WindowsProxySettings;

// Configuration for proxy behavior
struct ProxyConfig {
    show_log: bool,
    send_log: bool,
}

static PROXY_CONFIG: std::sync::OnceLock<ProxyConfig> = std::sync::OnceLock::new();

// Initialize proxy configuration
fn get_proxy_config() -> &'static ProxyConfig {
    PROXY_CONFIG.get_or_init(|| ProxyConfig {
        show_log: true,
        send_log: true,
    })
}

// URL filtering functions
fn is_useless_log_url(url: &str) -> bool {
    let useless_patterns = [
        "hoyoverse.com",
        "mihoyo.com",
        "unity.com",
        "unitychina.cn",
        "googleapis.com",
        "google.com",
        "crashlytics.com",
        "fabric.io",
    ];
    
    useless_patterns.iter().any(|pattern| url.contains(pattern))
}

fn is_good_log_url(url: &str) -> bool {
    let good_patterns = [
        "api-os-takumi",
        "api-takumi",
        "api-account-os",
        "api-account",
        "sdk-os-static",
        "sdk-static",
        "webstatic-sea",
        "webstatic",
        "hk4e-api-os",
        "hk4e-api",
        "bh3-api-os",
        "bh3-api",
        "sg-public-api",
        "public-api",
    ];
    
    good_patterns.iter().any(|pattern| url.contains(pattern))
}

fn should_ignore_url(url: &str) -> bool {
    let config = get_proxy_config();
    
    if !config.show_log && is_useless_log_url(url) {
        return true;
    }
    
    if !config.send_log && !is_good_log_url(url) {
        return true;
    }
    
    false
}

fn is_private_hostname(hostname: &str) -> bool {
    hostname == "localhost" || 
    hostname.starts_with("127.") || 
    hostname.starts_with("192.168.") || 
    hostname.starts_with("10.") || 
    hostname.starts_with("172.")
}

fn should_redirect_hostname(hostname: &str) -> bool {
    let game_domains = [
        "hoyoverse.com",
        "mihoyo.com",
        "yuanshen.com",
        "genshinimpact.com",
        "honkaiimpact3.com",
        "honkaistarrail.com",
        "zenlesszonezero.com",
    ];
    
    game_domains.iter().any(|domain| hostname.ends_with(domain))
}

// Custom handler for hudsucker
#[derive(Clone)]
struct GameProxyHandler;

#[async_trait::async_trait]
impl HttpHandler for GameProxyHandler {
    async fn handle_request(
        &mut self,
        _ctx: &HttpContext,
        mut req: Request<Body>,
    ) -> RequestOrResponse {
        let uri = req.uri();
        let url_str = uri.to_string();
        
        // Check if we should ignore this URL
        if should_ignore_url(&url_str) {
            let response = Response::builder()
                .status(StatusCode::NO_CONTENT)
                .body(Body::empty())
                .unwrap();
            return RequestOrResponse::Response(response);
        }
        
        // Check if we should redirect the hostname
        if let Some(host) = uri.host() {
            if should_redirect_hostname(host) && !is_private_hostname(host) {
                // Redirect to ps.yuuki.me
                let new_uri = format!("{}://ps.yuuki.me{}", 
                    uri.scheme_str().unwrap_or("https"),
                    uri.path_and_query().map(|pq| pq.as_str()).unwrap_or("/")
                );
                
                info!("🔄 Redirecting: {} -> {}", url_str, new_uri);
                
                // Modify the request URI
                let new_uri: hudsucker::hyper::Uri = new_uri.parse().unwrap_or_else(|_| uri.clone());
                *req.uri_mut() = new_uri;
                
                // Update the Host header
                if let Ok(host_header) = "ps.yuuki.me".parse() {
                    req.headers_mut().insert("host", host_header);
                }
            }
        }
        
        RequestOrResponse::Request(req)
    }
    
    async fn handle_response(
        &mut self,
        _ctx: &HttpContext,
        res: Response<Body>,
    ) -> Response<Body> {
        res
    }
}

#[cfg(target_os = "windows")]
fn get_current_proxy_settings() -> Result<WindowsProxySettings, String> {
    let hkcu = RegKey::predef(HKEY_CURRENT_USER);
    let internet_settings = hkcu
        .open_subkey("Software\\Microsoft\\Windows\\CurrentVersion\\Internet Settings")
        .map_err(|e| format!("Failed to open registry key: {}", e))?;
    
    let proxy_enable: u32 = internet_settings
        .get_value("ProxyEnable")
        .unwrap_or(0);
    
    let proxy_server: String = internet_settings
        .get_value("ProxyServer")
        .unwrap_or_default();
    
    let proxy_override: String = internet_settings
        .get_value("ProxyOverride")
        .unwrap_or_default();
    
    Ok(WindowsProxySettings {
        proxy_enable,
        proxy_server,
        proxy_override,
    })
}

#[cfg(target_os = "windows")]
fn set_windows_proxy(proxy_server: &str, proxy_override: &str) -> Result<(), String> {
    let hkcu = RegKey::predef(HKEY_CURRENT_USER);
    let internet_settings = hkcu
        .open_subkey_with_flags("Software\\Microsoft\\Windows\\CurrentVersion\\Internet Settings", KEY_SET_VALUE)
        .map_err(|e| format!("Failed to open registry key for writing: {}", e))?;
    
    internet_settings
        .set_value("ProxyEnable", &1u32)
        .map_err(|e| format!("Failed to enable proxy: {}", e))?;
    
    internet_settings
        .set_value("ProxyServer", &proxy_server)
        .map_err(|e| format!("Failed to set proxy server: {}", e))?;
    
    internet_settings
        .set_value("ProxyOverride", &proxy_override)
        .map_err(|e| format!("Failed to set proxy override: {}", e))?;
    
    Ok(())
}

#[cfg(target_os = "windows")]
fn restore_windows_proxy(settings: &WindowsProxySettings) -> Result<(), String> {
    let hkcu = RegKey::predef(HKEY_CURRENT_USER);
    let internet_settings = hkcu
        .open_subkey_with_flags("Software\\Microsoft\\Windows\\CurrentVersion\\Internet Settings", KEY_SET_VALUE)
        .map_err(|e| format!("Failed to open registry key for writing: {}", e))?;
    
    internet_settings
        .set_value("ProxyEnable", &settings.proxy_enable)
        .map_err(|e| format!("Failed to restore proxy enable: {}", e))?;
    
    internet_settings
        .set_value("ProxyServer", &settings.proxy_server)
        .map_err(|e| format!("Failed to restore proxy server: {}", e))?;
    
    internet_settings
        .set_value("ProxyOverride", &settings.proxy_override)
        .map_err(|e| format!("Failed to restore proxy override: {}", e))?;
    
    Ok(())
}

#[cfg(not(target_os = "windows"))]
fn get_current_proxy_settings() -> Result<WindowsProxySettings, String> {
    Ok(WindowsProxySettings)
}

#[cfg(not(target_os = "windows"))]
fn set_windows_proxy(_proxy_server: &str, _proxy_override: &str) -> Result<(), String> {
    Ok(())
}

#[cfg(not(target_os = "windows"))]
fn restore_windows_proxy(_settings: &WindowsProxySettings) -> Result<(), String> {
    Ok(())
}

pub fn start_proxy() -> Result<String, String> {
    let mut proxy_state = PROXY_STATE.lock().unwrap();
    
    // If proxy is already running, stop it first
    if proxy_state.is_some() {
        info!("🔄 Proxy already running, stopping existing proxy...");
        
        // Take the existing handle to stop it
        if let Some(handle) = proxy_state.take() {
            // Send shutdown signal
            let _ = handle.shutdown_tx.send(());
            
            // Restore original proxy settings
            if let Some(original_settings) = &handle.original_proxy_settings {
                let _ = restore_windows_proxy(original_settings);
            }
            
            info!("🛑 Previous proxy stopped, starting new proxy...");
        }
    }
    
    // Get current proxy settings before modifying them
    let original_settings = get_current_proxy_settings().ok();
    
    // Set Windows proxy settings
    set_windows_proxy("127.0.0.1:8080", "localhost;127.*;10.*;172.16.*;172.17.*;172.18.*;172.19.*;172.20.*;172.21.*;172.22.*;172.23.*;172.24.*;172.25.*;172.26.*;172.27.*;172.28.*;172.29.*;172.30.*;172.31.*;192.168.*")?;
    
    // Create runtime for the proxy server
    let runtime = Runtime::new().map_err(|e| format!("Failed to create runtime: {}", e))?;
    
    let (shutdown_tx, shutdown_rx) = tokio::sync::oneshot::channel();
    
    // Start the proxy server
    runtime.spawn(async move {
        // Initialize tracing
        tracing_subscriber::fmt::init();
        
        // Create certificate authority with generated key and cert
        use rcgen::{generate_simple_self_signed, CertifiedKey};
        use rustls::{Certificate, PrivateKey};
        
        let subject_alt_names = vec!["localhost".to_string(), "127.0.0.1".to_string()];
        let CertifiedKey { cert, key_pair } = generate_simple_self_signed(subject_alt_names)
            .expect("Failed to generate certificate");
        
        let cert_der = cert.der().to_vec();
        let key_der = key_pair.serialize_der();
        
        let ca = RcgenAuthority::new(
            PrivateKey(key_der),
            Certificate(cert_der),
            1024
        ).expect("Failed to create certificate authority");
        
        // Create HTTP client
        let client = hyper::Client::builder()
            .build(hyper_rustls::HttpsConnectorBuilder::new()
                .with_native_roots()
                .https_or_http()
                .enable_http1()
                .build());
        
        let proxy = Proxy::builder()
            .with_addr(SocketAddr::from(([127, 0, 0, 1], 8080)))
            .with_client(client)
            .with_ca(ca)
            .with_http_handler(GameProxyHandler)
            .build();
        
        info!("🚀 MITM Proxy listening on https://127.0.0.1:8080");
        info!("🎯 Redirecting game domains to ps.yuuki.me");
        info!("📋 Monitored domains:");
        info!("   • *.zenlesszonezero.com");
        info!("   • *.honkaiimpact3.com");
        info!("   • *.honkaistarrail.com");
        info!("   • *.genshinimpact.com");
        info!("   • *.hoyoverse.com");
        info!("   • *.mihoyo.com");
        
        let shutdown_future = async {
            let _ = shutdown_rx.await;
        };
        
        if let Err(e) = proxy.start(shutdown_future).await {
            tracing::error!("Proxy error: {}", e);
        }
    });
    
    *proxy_state = Some(ProxyHandle {
        _runtime: runtime,
        shutdown_tx,
        original_proxy_settings: original_settings,
    });
    
    Ok("MITM Proxy started successfully on 127.0.0.1:8080 with automatic Windows proxy configuration.".to_string())
}

pub fn stop_proxy() -> Result<String, String> {
    let mut proxy_state = PROXY_STATE.lock().unwrap();
    
    if let Some(handle) = proxy_state.take() {
        // Send shutdown signal
        let _ = handle.shutdown_tx.send(());
        
        // Restore original proxy settings
        if let Some(original_settings) = &handle.original_proxy_settings {
            restore_windows_proxy(original_settings)?;
        }
        
        info!("🛑 MITM Proxy stopped");
        info!("🔄 Windows proxy settings restored");
        
        Ok("MITM Proxy stopped successfully. Windows proxy settings restored.".to_string())
    } else {
        Err("Proxy is not running".to_string())
    }
}

pub fn get_certificate_path() -> Result<String, String> {
    let cert_path = std::env::temp_dir().join("yuukips_proxy_cert.pem");
    Ok(cert_path.to_string_lossy().to_string())
}

pub fn is_proxy_running() -> bool {
    let proxy_state = PROXY_STATE.lock().unwrap();
    proxy_state.is_some()
}

pub fn force_stop_proxy() -> Result<String, String> {
    let mut proxy_state = PROXY_STATE.lock().unwrap();
    
    if let Some(handle) = proxy_state.take() {
        // Send shutdown signal
        let _ = handle.shutdown_tx.send(());
        
        // Restore original proxy settings
        if let Some(original_settings) = &handle.original_proxy_settings {
            restore_windows_proxy(original_settings)?;
        }
        
        info!("🛑 MITM Proxy force stopped");
        info!("🔄 Windows proxy settings restored");
        
        Ok("MITM Proxy force stopped successfully. Windows proxy settings restored.".to_string())
    } else {
        Ok("No proxy was running.".to_string())
    }
}

#[cfg(target_os = "windows")]
pub fn check_and_disable_windows_proxy() -> Result<String, String> {
    let current_settings = get_current_proxy_settings()?;
    
    // Check if Windows proxy is currently enabled
    if current_settings.proxy_enable == 1 {
        info!("🔍 Detected enabled Windows proxy settings");
        info!("   Proxy Server: {}", current_settings.proxy_server);
        info!("   Proxy Override: {}", current_settings.proxy_override);
        
        // Disable Windows proxy by setting ProxyEnable to 0
        let disabled_settings = WindowsProxySettings {
            proxy_enable: 0,
            proxy_server: String::new(),
            proxy_override: String::new(),
        };
        
        restore_windows_proxy(&disabled_settings)?;
        
        info!("🛑 Windows proxy settings disabled");
        Ok(format!("Windows proxy was enabled and has been disabled. Previous settings: Server={}, Override={}", 
                  current_settings.proxy_server, current_settings.proxy_override))
    } else {
        info!("✅ No Windows proxy settings detected or already disabled");
        Ok("No Windows proxy settings were enabled.".to_string())
    }
}

#[cfg(not(target_os = "windows"))]
pub fn check_and_disable_windows_proxy() -> Result<String, String> {
    Ok("Windows proxy check not available on this platform.".to_string())
}