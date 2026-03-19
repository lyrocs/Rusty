//! Network subsystem: WiFi connection and HTTP client.
//!
//! Extracted from stdgotchi's wifi.rs.

use crate::error::{Result, RustyError};
use esp_idf_svc::eventloop::EspSystemEventLoop;
use esp_idf_svc::hal::peripheral;
use esp_idf_svc::http::client::{Configuration as HttpConfig, EspHttpConnection};
use esp_idf_svc::io::Read;
use esp_idf_svc::nvs::EspDefaultNvsPartition;
use esp_idf_svc::wifi::{AuthMethod, BlockingWifi, ClientConfiguration, Configuration, EspWifi};
use serde::{Deserialize, Serialize};
use std::time::Duration;

/// Maximum HTTP response size (64KB, fits in PSRAM).
const MAX_HTTP_RESPONSE_SIZE: usize = 65536;

/// WiFi credentials.
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

/// Network handle providing WiFi and HTTP functionality.
pub struct Network {
    wifi: BlockingWifi<EspWifi<'static>>,
}

impl Network {
    /// Connect to WiFi with the given config.
    pub fn connect(
        config: &WifiConfig,
        modem: impl peripheral::Peripheral<P = esp_idf_svc::hal::modem::Modem> + 'static,
        sysloop: EspSystemEventLoop,
        nvs: EspDefaultNvsPartition,
    ) -> Result<Self> {
        log::info!("Initializing WiFi...");

        let mut wifi = BlockingWifi::wrap(
            EspWifi::new(modem, sysloop.clone(), Some(nvs))
                .map_err(|e| RustyError::Network(format!("WiFi init: {:?}", e)))?,
            sysloop,
        )
        .map_err(|e| RustyError::Network(format!("WiFi wrap: {:?}", e)))?;

        let wifi_config = Configuration::Client(ClientConfiguration {
            ssid: config
                .ssid
                .as_str()
                .try_into()
                .map_err(|_| RustyError::Network("SSID too long".into()))?,
            bssid: None,
            auth_method: AuthMethod::WPA2Personal,
            password: config
                .password
                .as_str()
                .try_into()
                .map_err(|_| RustyError::Network("Password too long".into()))?,
            channel: None,
            ..Default::default()
        });

        wifi.set_configuration(&wifi_config)
            .map_err(|e| RustyError::Network(format!("WiFi config: {:?}", e)))?;

        wifi.start()
            .map_err(|e| RustyError::Network(format!("WiFi start: {:?}", e)))?;

        wifi.connect()
            .map_err(|e| RustyError::Network(format!("WiFi connect: {:?}", e)))?;

        wifi.wait_netif_up()
            .map_err(|e| RustyError::Network(format!("WiFi netif: {:?}", e)))?;

        let ip_info = wifi
            .wifi()
            .sta_netif()
            .get_ip_info()
            .map_err(|e| RustyError::Network(format!("IP info: {:?}", e)))?;
        log::info!("WiFi connected: IP={}", ip_info.ip);

        Ok(Self { wifi })
    }

    /// Check if WiFi is connected.
    pub fn is_connected(&self) -> bool {
        self.wifi.is_connected().unwrap_or(false)
    }

    /// Perform an HTTP GET request, returning the body as a String.
    pub fn http_get(&self, url: &str) -> Result<String> {
        log::info!("HTTP GET: {}", url);

        let mut connection = EspHttpConnection::new(&HttpConfig {
            timeout: Some(Duration::from_secs(10)),
            buffer_size: Some(MAX_HTTP_RESPONSE_SIZE),
            crt_bundle_attach: Some(esp_idf_svc::sys::esp_crt_bundle_attach),
            ..Default::default()
        })
        .map_err(|e| RustyError::Network(format!("HTTP connection: {:?}", e)))?;

        connection
            .initiate_request(
                esp_idf_svc::http::Method::Get,
                url,
                &[("User-Agent", "RustyKit-ESP32")],
            )
            .map_err(|e| RustyError::Network(format!("HTTP request: {:?}", e)))?;

        connection
            .initiate_response()
            .map_err(|e| RustyError::Network(format!("HTTP response: {:?}", e)))?;

        let status = connection.status();
        if status != 200 {
            return Err(RustyError::Network(format!("HTTP status: {}", status)));
        }

        let mut buffer = vec![0u8; MAX_HTTP_RESPONSE_SIZE];
        let bytes_read = Read::read(&mut connection, &mut buffer)
            .map_err(|e| RustyError::Network(format!("HTTP read: {:?}", e)))?;

        Ok(String::from_utf8_lossy(&buffer[..bytes_read]).to_string())
    }

    /// Perform an HTTP GET and parse the response as JSON.
    pub fn http_get_json<T: serde::de::DeserializeOwned>(&self, url: &str) -> Result<T> {
        let body = self.http_get(url)?;
        serde_json::from_str(&body).map_err(|e| RustyError::Network(format!("JSON parse: {:?}", e)))
    }
}
