use std::fs::{File, create_dir_all};
use std::io::{self, Write};
use std::path::{Path, PathBuf};
use std::time::Instant;
use std::net::{IpAddr, SocketAddr};

use tokio::time::{sleep, timeout, Duration};
use tokio::net::TcpStream;
use reqwest::Client;
use serde_json::Value;
use serde::{Deserialize, Serialize};
use chrono::Local;
use futures::future::join_all;

// Add this to import CloudIpChecker
use crate::cloud_ip_checker::CloudIpChecker;

const API_SERVICES: [&str; 5] = [
    "http://ip-api.com/json/{}",
    "http://ipinfo.io/{}/json",
    "http://geoip.nekudo.com/api/{}",
    "https://ipapi.co/{}/json/",
    "https://freegeoip.app/json/{}"
];

const DEFAULT_OUTPUT_DIR: &str = "ip_geolocation_results";
const PORT_SCAN_TIMEOUT: Duration = Duration::from_secs(1);
const COMMON_PORTS: [u16; 10] = [21, 22, 80, 443, 3306, 5432, 8080, 8443, 27017, 6379];

#[derive(Default)]
pub struct ApiMetrics {
    pub success_count: u32,
    pub failure_count: u32,
    pub total_time: Duration,
}

#[derive(Serialize)]
pub struct IpInfo {
    pub ip: String,
    pub location: Option<String>,
    pub is_active: bool,
    pub open_ports: Vec<u16>,
    pub cloud_provider: Option<String>, // New field for cloud provider information
}

#[derive(Deserialize)]
pub struct IpInput {
    pub ips: String,
    pub use_default_output: bool,
}

#[derive(Serialize)]
pub struct ApiResponse {
    pub message: String,
    pub metrics: ApiMetricsResponse,
    pub results: Vec<IpInfo>,
    pub total_ips: usize,
}

#[derive(Serialize)]
pub struct ApiMetricsResponse {
    pub total_requests: u32,
    pub success_rate: f64,
    pub average_response_time: f64,
}

pub async fn get_geolocation(client: &Client, ip: &str) -> (Result<String, String>, Duration) {
    let start = Instant::now();
    for api in API_SERVICES.iter() {
        let url = api.replace("{}", ip);
        match client.get(&url).send().await {
            Ok(resp) if resp.status().is_success() => {
                match resp.json::<Value>().await {
                    Ok(json) => {
                        let city = json["city"].as_str().unwrap_or("Unknown");
                        let country = json["country"].as_str().unwrap_or("Unknown");
                        return (Ok(format!("{}, {}", city, country)), start.elapsed());
                    },
                    Err(e) => eprintln!("Failed to parse JSON from {}: {}", url, e),
                }
            },
            Ok(_) => eprintln!("Non-success status from {}, trying next API", url),
            Err(e) => eprintln!("Request failed for {}: {}. Trying next API", url, e),
        }
        sleep(Duration::from_secs(1)).await;
    }
    (Err(format!("Failed to get geolocation for IP: {}", ip)), start.elapsed())
}

pub async fn check_ip_activity(ip: &str) -> bool {
    match ip.parse::<IpAddr>() {
        Ok(ip_addr) => {
            TcpStream::connect((ip_addr, 80)).await.is_ok() || 
            TcpStream::connect((ip_addr, 443)).await.is_ok()
        },
        Err(_) => false,
    }
}

async fn scan_port(ip: IpAddr, port: u16) -> bool {
    let socket = SocketAddr::new(ip, port);
    timeout(PORT_SCAN_TIMEOUT, TcpStream::connect(socket)).await.is_ok()
}

pub async fn scan_ip_ports(ip: &str) -> Vec<u16> {
    match ip.parse::<IpAddr>() {
        Ok(ip_addr) => {
            let port_futures = COMMON_PORTS.iter().map(|&port| scan_port(ip_addr, port));
            let results = join_all(port_futures).await;
            COMMON_PORTS.iter()
                .zip(results.iter())
                .filter(|(_, &is_open)| is_open)
                .map(|(&port, _)| port)
                .collect()
        },
        Err(_) => vec![],
    }
}

pub async fn process_ip(client: &Client, ip: String, cloud_checker: &CloudIpChecker) -> IpInfo {
    let (location_result, _) = get_geolocation(client, &ip).await;
    let is_active = check_ip_activity(&ip).await;
    let open_ports = if is_active {
        scan_ip_ports(&ip).await
    } else {
        vec![]
    };

    // Check if the IP is on a cloud server
    let cloud_provider = ip.parse::<IpAddr>()
        .ok()
        .and_then(|ip_addr| cloud_checker.is_cloud_ip(ip_addr));

    IpInfo {
        ip,
        location: location_result.ok(),
        is_active,
        open_ports,
        cloud_provider,
    }
}

pub async fn process_ips(client: &Client, ips: &[String], cloud_checker: &CloudIpChecker) -> (Vec<IpInfo>, ApiMetrics) {
    let start = Instant::now();
    let results = join_all(ips.iter().cloned().map(|ip| process_ip(client, ip, cloud_checker))).await;

    let mut metrics = ApiMetrics::default();
    metrics.total_time = start.elapsed();
    metrics.success_count = results.iter().filter(|info| info.location.is_some()).count() as u32;
    metrics.failure_count = (results.len() as u32) - metrics.success_count;

    println!("\nAPI Performance Metrics:");
    println!("Total requests: {}", results.len());
    println!("Success rate: {:.2}%", (metrics.success_count as f64 / results.len() as f64) * 100.0);
    println!("Average response time: {:.2}ms", metrics.total_time.as_secs_f64() * 1000.0 / results.len() as f64);

    (results, metrics)
}

pub fn write_to_file(results: &[IpInfo], file_path: &Path) -> io::Result<()> {
    if let Some(parent) = file_path.parent() {
        create_dir_all(parent)?;
    }

    let mut file = File::create(file_path)?;
    writeln!(file, "IP,Location,Active,Open Ports,Cloud Provider")?;
    for info in results {
        writeln!(file, "{},{},{},{},{}",
            info.ip,
            info.location.as_deref().unwrap_or("Unknown"),
            info.is_active,
            info.open_ports.iter().map(|p| p.to_string()).collect::<Vec<_>>().join(";"),
            info.cloud_provider.as_deref().unwrap_or("Not on cloud")
        )?;
    }
    Ok(())
}

pub fn get_default_output_path() -> PathBuf {
    let mut path = PathBuf::from(DEFAULT_OUTPUT_DIR);
    path.push(format!("ip_info_{}.csv", Local::now().format("%Y%m%d_%H%M%S")));
    path
}