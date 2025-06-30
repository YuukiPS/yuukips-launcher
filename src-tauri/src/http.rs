//! HTTP utilities module
//! Handles HTTP client creation and network requests

use reqwest;
use tauri::command;

/// Create an HTTP client with optional proxy bypass
pub fn create_http_client(use_proxy: bool) -> Result<reqwest::Client, String> {
    let mut client_builder = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(30))
        .user_agent("YuukiPS-Launcher/1.0");
    
    if !use_proxy {
        client_builder = client_builder.no_proxy();
    }
    
    client_builder.build()
        .map_err(|e| format!("Failed to create HTTP client: {}", e))
}

/// Test proxy bypass functionality
#[command]
pub fn test_proxy_bypass(url: String) -> Result<String, String> {
    let rt = tokio::runtime::Runtime::new()
        .map_err(|e| format!("Failed to create async runtime: {}", e))?;
    
    rt.block_on(async {
        let client = create_http_client(false)?; // No proxy
        
        println!("🌐 Testing proxy bypass for: {}", url);
        
        let response = client.get(&url)
            .send()
            .await
            .map_err(|e| format!("Request failed: {}", e))?;
        
        let status = response.status();
        let headers = response.headers().clone();
        
        let body = response.text()
            .await
            .map_err(|e| format!("Failed to read response body: {}", e))?;
        
        let result = serde_json::json!({
            "status": status.as_u16(),
            "headers": headers.iter().map(|(k, v)| {
                (k.as_str(), v.to_str().unwrap_or("<invalid>"))
            }).collect::<std::collections::HashMap<_, _>>(),
            "body_length": body.len(),
            "body_preview": if body.len() > 200 {
                format!("{}...", &body[..200])
            } else {
                body
            }
        });
        
        serde_json::to_string(&result)
            .map_err(|e| format!("Failed to serialize response: {}", e))
    })
}

/// Fetch data from an API endpoint
#[command]
pub fn fetch_api_data(url: String) -> Result<String, String> {
    let rt = tokio::runtime::Runtime::new()
        .map_err(|e| format!("Failed to create async runtime: {}", e))?;
    
    rt.block_on(async {
        let client = create_http_client(false)?; // No proxy for API calls
        
        println!("📡 Fetching API data from: {}", url);
        
        let response = client.get(&url)
            .header("Accept", "application/json")
            .send()
            .await
            .map_err(|e| format!("API request failed: {}", e))?;
        
        if !response.status().is_success() {
            return Err(format!("API returned error status: {}", response.status()));
        }
        
        let body = response.text()
            .await
            .map_err(|e| format!("Failed to read API response: {}", e))?;
        
        // Validate that it's valid JSON
        let _: serde_json::Value = serde_json::from_str(&body)
            .map_err(|e| format!("API response is not valid JSON: {}", e))?;
        
        Ok(body)
    })
}

/// Test network connectivity
#[command]
pub fn test_network_connectivity() -> Result<String, String> {
    let rt = tokio::runtime::Runtime::new()
        .map_err(|e| format!("Failed to create async runtime: {}", e))?;
    
    rt.block_on(async {
        let client = create_http_client(false)?;
        
        // Test multiple endpoints to ensure connectivity
        let test_urls = vec![
            "https://httpbin.org/get",
            "https://api.github.com",
            "https://www.google.com",
        ];
        
        let mut results = Vec::new();
        
        for url in test_urls {
            let start_time = std::time::Instant::now();
            
            match client.get(url)
                .timeout(std::time::Duration::from_secs(10))
                .send()
                .await
            {
                Ok(response) => {
                    let duration = start_time.elapsed();
                    results.push(serde_json::json!({
                        "url": url,
                        "status": response.status().as_u16(),
                        "success": true,
                        "duration_ms": duration.as_millis(),
                        "error": null
                    }));
                }
                Err(e) => {
                    let duration = start_time.elapsed();
                    results.push(serde_json::json!({
                        "url": url,
                        "status": null,
                        "success": false,
                        "duration_ms": duration.as_millis(),
                        "error": e.to_string()
                    }));
                }
            }
        }
        
        let summary = serde_json::json!({
            "timestamp": chrono::Utc::now().to_rfc3339(),
            "total_tests": results.len(),
            "successful_tests": results.iter().filter(|r| r["success"].as_bool().unwrap_or(false)).count(),
            "results": results
        });
        
        serde_json::to_string(&summary)
            .map_err(|e| format!("Failed to serialize connectivity test results: {}", e))
    })
}