//! WiFi connection and HTTP client module for ESP32-S3
//!
//! This module provides WiFi connectivity and HTTP GET functionality.

use anyhow::{bail, Result};
use esp_idf_svc::eventloop::EspSystemEventLoop;
use esp_idf_svc::hal::peripheral;
use esp_idf_svc::http::client::{Configuration as HttpConfig, EspHttpConnection};
use esp_idf_svc::io::Read;
use esp_idf_svc::nvs::EspDefaultNvsPartition;
use esp_idf_svc::wifi::{AuthMethod, BlockingWifi, ClientConfiguration, Configuration, EspWifi};
use log::info;
use serde::{Deserialize, Serialize};
use std::time::Duration;

/// WiFi credentials configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WifiConfig {
    pub ssid: String,
    pub password: String,
}

impl Default for WifiConfig {
    fn default() -> Self {
        Self {
            ssid: "YOUR_WIFI_SSID".to_string(),
            password: "YOUR_WIFI_PASSWORD".to_string(),
        }
    }
}

/// WiFi connection timeout in milliseconds
const WIFI_CONNECT_TIMEOUT_MS: u64 = 10000;

/// Maximum HTTP response size in bytes
/// Pokemon API responses can be quite large (30-50KB with all moves/stats)
/// ESP32-S3 has plenty of PSRAM, so we use 64KB to be safe
const MAX_HTTP_RESPONSE_SIZE: usize = 65536;

/// Load WiFi configuration from SD card
///
/// # Arguments
/// * `sd_card` - Mutable reference to SD card wrapper
///
/// # Returns
/// * `Ok(WifiConfig)` - WiFi configuration loaded from file
/// * `Err` - File not found or invalid JSON
///
/// # File Format
/// The file `/sdcard/wifi.json` should contain:
/// ```json
/// {
///   "ssid": "YOUR_NETWORK_NAME",
///   "password": "YOUR_PASSWORD"
/// }
/// ```
pub fn load_wifi_config(sd_card: &mut crate::ecs::resources::SdCardWrapper) -> Result<WifiConfig> {
    let filename = "/sdcard/wifi.json";

    info!("Loading WiFi config from: {}", filename);

    // Read file from SD card
    let json_data = sd_card.load_from_file(filename)
        .map_err(|e| anyhow::anyhow!("Failed to read WiFi config file: {:?}", e))?;

    // Parse JSON
    let config: WifiConfig = serde_json::from_str(&json_data)
        .map_err(|e| anyhow::anyhow!("Failed to parse WiFi config JSON: {:?}", e))?;

    info!("WiFi config loaded: SSID={}", config.ssid);
    Ok(config)
}

/// Create a default WiFi configuration file on SD card
///
/// This creates a template file that users can edit with their credentials
pub fn create_default_wifi_config(sd_card: &mut crate::ecs::resources::SdCardWrapper) -> Result<()> {
    let filename = "/sdcard/wifi.json";
    let default_config = WifiConfig::default();

    let json_data = serde_json::to_string_pretty(&default_config)
        .map_err(|e| anyhow::anyhow!("Failed to serialize WiFi config: {:?}", e))?;

    sd_card.save_to_file(filename, &json_data)
        .map_err(|e| anyhow::anyhow!("Failed to save WiFi config file: {:?}", e))?;

    info!("Created default WiFi config at: {}", filename);
    Ok(())
}

/// Initialize WiFi connection
///
/// # Arguments
/// * `config` - WiFi configuration with SSID and password
/// * `modem` - WiFi modem peripheral
/// * `sysloop` - System event loop
/// * `nvs` - NVS partition for WiFi credentials storage
///
/// # Returns
/// * `Ok(BlockingWifi)` - Successfully connected WiFi instance
/// * `Err` - Connection failed
pub fn wifi_create<'d>(
    config: &WifiConfig,
    modem: impl peripheral::Peripheral<P = esp_idf_svc::hal::modem::Modem> + 'd,
    sysloop: EspSystemEventLoop,
    nvs: EspDefaultNvsPartition,
) -> Result<BlockingWifi<EspWifi<'d>>> {
    info!("Initializing WiFi...");

    let mut wifi = BlockingWifi::wrap(EspWifi::new(modem, sysloop.clone(), Some(nvs))?, sysloop)?;

    let wifi_configuration = Configuration::Client(ClientConfiguration {
        ssid: config.ssid.as_str().try_into()
            .map_err(|_| anyhow::anyhow!("SSID too long or invalid"))?,
        bssid: None,
        auth_method: AuthMethod::WPA2Personal,
        password: config.password.as_str().try_into()
            .map_err(|_| anyhow::anyhow!("Password too long or invalid"))?,
        channel: None,
        ..Default::default()
    });

    wifi.set_configuration(&wifi_configuration)?;

    info!("Starting WiFi...");
    wifi.start()?;

    info!("Connecting to WiFi SSID: {}", config.ssid);
    wifi.connect()?;

    info!("Waiting for IP address assignment...");
    wifi.wait_netif_up()?;

    let ip_info = wifi.wifi().sta_netif().get_ip_info()?;
    info!("WiFi connected successfully!");
    info!("IP address: {}", ip_info.ip);
    info!("Subnet mask: {}", ip_info.subnet.mask);
    info!("Gateway: {}", ip_info.subnet.gateway);

    Ok(wifi)
}

/// Perform HTTP GET request
///
/// # Arguments
/// * `url` - The URL to fetch (e.g., "http://example.com/api/data")
///
/// # Returns
/// * `Ok(String)` - Response body as string
/// * `Err` - Request failed
///
/// # Example
/// ```no_run
/// let response = http_get("http://api.example.com/data")?;
/// println!("Response: {}", response);
/// ```
pub fn http_get(url: &str) -> Result<String> {
    info!("Performing HTTP GET request to: {}", url);

    // Create HTTP connection with TLS configuration
    // For development/testing, we skip certificate verification
    let mut connection = EspHttpConnection::new(&HttpConfig {
        timeout: Some(Duration::from_secs(10)),
        buffer_size: Some(MAX_HTTP_RESPONSE_SIZE),
        crt_bundle_attach: Some(esp_idf_svc::sys::esp_crt_bundle_attach),
        ..Default::default()
    })?;

    // Perform GET request
    connection.initiate_request(
        esp_idf_svc::http::Method::Get,
        url,
        &[("User-Agent", "ESP32-Rust-Client")],
    )?;

    info!("Submitting HTTP request...");
    connection.initiate_response()?;

    let status = connection.status();
    info!("HTTP Response Status: {}", status);

    if status != 200 {
        bail!("HTTP request failed with status: {}", status);
    }

    // Read response body
    let mut buffer = vec![0u8; MAX_HTTP_RESPONSE_SIZE];
    let bytes_read = Read::read(&mut connection, &mut buffer)?;

    info!("Read {} bytes from response", bytes_read);

    // Convert to string
    let response_text = String::from_utf8_lossy(&buffer[..bytes_read]).to_string();

    info!("HTTP GET successful");
    Ok(response_text)
}

/// Perform HTTP GET request and parse JSON
///
/// # Arguments
/// * `url` - The URL to fetch
///
/// # Returns
/// * `Ok(serde_json::Value)` - Parsed JSON response
/// * `Err` - Request or parsing failed
///
/// # Example
/// ```no_run
/// let data = http_get_json("http://api.example.com/data.json")?;
/// println!("Data: {:?}", data);
/// ```
pub fn http_get_json(url: &str) -> Result<serde_json::Value> {
    let response_text = http_get(url)?;

    info!("Parsing JSON response...");
    let json: serde_json::Value = serde_json::from_str(&response_text)?;

    info!("JSON parsed successfully");
    Ok(json)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    #[ignore] // Requires actual WiFi connection
    fn test_http_get() {
        // This test requires a real ESP32 device with WiFi
        // Run with: cargo test --target xtensa-esp32s3-espidf -- --ignored
        let response = http_get("https://pokeapi.co/api/v2/pokemon/ditto").unwrap();
        assert!(!response.is_empty());
    }
}
