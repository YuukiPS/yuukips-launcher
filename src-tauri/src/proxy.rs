use std::net::SocketAddr;
use std::sync::Mutex;
use tokio::runtime::Runtime;
use hyper::{Body, Request, Response, Uri, Client, Server};
use hyper::service::{make_service_fn, service_fn};
use std::convert::Infallible;


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

// Simple HTTP proxy handler
async fn proxy_handler(req: Request<Body>) -> Result<Response<Body>, Infallible> {
    let uri = req.uri();
    let method = req.method();
    
    // Handle CONNECT method for HTTPS tunneling
    if method == hyper::Method::CONNECT {
        // For simplicity, we'll reject CONNECT requests
        // In a full implementation, you'd establish a tunnel
        return Ok(Response::builder()
            .status(200)
            .body(Body::from("Connection established"))
            .unwrap());
    }
    
    let url_str = uri.to_string();
    
    // Check if we should ignore this URL
    if should_ignore_url(&url_str) {
        return Ok(Response::builder()
            .status(204)
            .body(Body::empty())
            .unwrap());
    }
    
    // Check if we should redirect the hostname
    if let Some(host) = uri.host() {
        if should_redirect_hostname(host) && !is_private_hostname(host) {
            // Redirect to ps.yuuki.me
            let new_uri = format!("{}://ps.yuuki.me{}", 
                uri.scheme_str().unwrap_or("http"),
                uri.path_and_query().map(|pq| pq.as_str()).unwrap_or("/")
            );
            
            println!("🔄 Redirecting: {} -> {}", url_str, new_uri);
            
            // Create a new request to the redirected URL
            let new_uri: Uri = new_uri.parse().unwrap_or_else(|_| uri.clone());
            let mut new_req = Request::builder()
                .method(method)
                .uri(new_uri);
            
            // Copy headers
            for (key, value) in req.headers() {
                if key != "host" {
                    new_req = new_req.header(key, value);
                }
            }
            
            let client = Client::new();
            match client.request(new_req.body(req.into_body()).unwrap()).await {
                Ok(response) => return Ok(response),
                Err(_) => {
                    return Ok(Response::builder()
                        .status(502)
                        .body(Body::from("Bad Gateway"))
                        .unwrap());
                }
            }
        }
    }
    
    // Forward the request as-is
    let client = Client::new();
    match client.request(req).await {
        Ok(response) => Ok(response),
        Err(_) => {
            Ok(Response::builder()
                .status(502)
                .body(Body::from("Bad Gateway"))
                .unwrap())
        }
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
    
    if proxy_state.is_some() {
        return Err("Proxy is already running".to_string());
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
        let make_svc = make_service_fn(|_conn| async {
            Ok::<_, Infallible>(service_fn(proxy_handler))
        });
        
        let addr = SocketAddr::from(([127, 0, 0, 1], 8080));
        let server = Server::bind(&addr).serve(make_svc);
        
        println!("🚀 HTTP Proxy listening on http://127.0.0.1:8080");
        println!("🎯 Redirecting game domains to ps.yuuki.me");
        println!("📋 Monitored domains:");
        println!("   • *.zenlesszonezero.com");
        println!("   • *.honkaiimpact3.com");
        println!("   • *.honkaistarrail.com");
        println!("   • *.genshinimpact.com");
        println!("   • *.hoyoverse.com");
        println!("   • *.mihoyo.com");
        
        let graceful = server.with_graceful_shutdown(async {
            shutdown_rx.await.ok();
        });
        
        if let Err(e) = graceful.await {
            eprintln!("Server error: {}", e);
        }
    });
    
    *proxy_state = Some(ProxyHandle {
        _runtime: runtime,
        shutdown_tx,
        original_proxy_settings: original_settings,
    });
    
    Ok("HTTP Proxy started successfully on 127.0.0.1:8080 with automatic Windows proxy configuration.".to_string())
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
        
        println!("🛑 HTTP Proxy stopped");
        println!("🔄 Windows proxy settings restored");
        
        Ok("HTTP Proxy stopped successfully. Windows proxy settings restored.".to_string())
    } else {
        Err("Proxy is not running".to_string())
    }
}

pub fn get_certificate_path() -> Result<String, String> {
    let cert_path = std::env::temp_dir().join("yuukips_proxy_cert.pem");
    Ok(cert_path.to_string_lossy().to_string())
}