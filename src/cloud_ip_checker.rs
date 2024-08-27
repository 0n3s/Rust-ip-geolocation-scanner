use std::fs;
use std::net::IpAddr;
use std::sync::Arc;
use serde_json::Value;
use ipnetwork::IpNetwork;

#[derive(Clone)]
pub struct CloudIpChecker {
    aws_ranges: Arc<Vec<IpNetwork>>,
    azure_ranges: Arc<Vec<IpNetwork>>,
    google_ranges: Arc<Vec<IpNetwork>>,
    alibaba_ranges: Arc<Vec<IpNetwork>>,
    tencent_ranges: Arc<Vec<IpNetwork>>,
    huawei_ranges: Arc<Vec<IpNetwork>>,
    tianyi_ranges: Arc<Vec<IpNetwork>>,
}

impl CloudIpChecker {
    pub fn new() -> Self {
        CloudIpChecker {
            aws_ranges: Arc::new(Self::load_aws_ranges()),
            azure_ranges: Arc::new(Self::load_azure_ranges()),
            google_ranges: Arc::new(Self::load_google_ranges()),
            alibaba_ranges: Arc::new(Self::load_alibaba_ranges()),
            tencent_ranges: Arc::new(Self::load_tencent_ranges()),
            huawei_ranges: Arc::new(Self::load_huawei_ranges()),
            tianyi_ranges: Arc::new(Self::load_tianyi_ranges()),
        }
    }

    pub fn is_cloud_ip(&self, ip: IpAddr) -> Option<String> {
        if self.aws_ranges.iter().any(|range| range.contains(ip)) {
            Some("AWS".to_string())
        } else if self.azure_ranges.iter().any(|range| range.contains(ip)) {
            Some("Azure".to_string())
        } else if self.google_ranges.iter().any(|range| range.contains(ip)) {
            Some("Google Cloud".to_string())
        } else if self.alibaba_ranges.iter().any(|range| range.contains(ip)) {
            Some("Alibaba Cloud".to_string())
        } else if self.tencent_ranges.iter().any(|range| range.contains(ip)) {
            Some("Tencent Cloud".to_string())
        } else if self.huawei_ranges.iter().any(|range| range.contains(ip)) {
            Some("Huawei Cloud".to_string())
        } else if self.tianyi_ranges.iter().any(|range| range.contains(ip)) {
            Some("Tianyi Cloud".to_string())
        } else {
            None
        }
    }

    fn load_aws_ranges() -> Vec<IpNetwork> {
        let data = fs::read_to_string("aws_ip_ranges.json").unwrap_or_else(|_| "{}".to_string());
        let json: Value = serde_json::from_str(&data).unwrap_or_else(|_| serde_json::json!({}));
        json["prefixes"]
            .as_array()
            .unwrap_or(&Vec::new())
            .iter()
            .filter_map(|prefix| {
                prefix["ip_prefix"].as_str().and_then(|ip| ip.parse().ok())
            })
            .collect()
    }
    
    fn load_azure_ranges() -> Vec<IpNetwork> {
        // Implement Azure IP range parsing logic here
        Vec::new()
    }

    fn load_google_ranges() -> Vec<IpNetwork> {
        let data = fs::read_to_string("google_cloud_ip_ranges.json").unwrap_or_else(|_| "{}".to_string());
        let json: Value = serde_json::from_str(&data).unwrap_or_else(|_| serde_json::json!({}));
        json["prefixes"]
            .as_array()
            .unwrap_or(&Vec::new())
            .iter()
            .filter_map(|prefix| {
                prefix["ipv4Prefix"].as_str().and_then(|ip| ip.parse().ok())
            })
            .collect()
    }

    fn load_alibaba_ranges() -> Vec<IpNetwork> {
        // Implement Alibaba Cloud IP range parsing logic here
        Vec::new()
    }

    fn load_tencent_ranges() -> Vec<IpNetwork> {
        // Implement Tencent Cloud IP range parsing logic here
        Vec::new()
    }

    fn load_huawei_ranges() -> Vec<IpNetwork> {
        // Implement Huawei Cloud IP range parsing logic here
        Vec::new()
    }

    fn load_tianyi_ranges() -> Vec<IpNetwork> {
        let data = fs::read_to_string("tianyi_cloud_ip_ranges.json").unwrap_or_else(|_| "{}".to_string());
        let json: Value = serde_json::from_str(&data).unwrap_or_else(|_| serde_json::json!({}));
        json["ip_ranges"]
            .as_array()
            .unwrap_or(&Vec::new())
            .iter()
            .filter_map(|range| range.as_str().and_then(|ip| ip.parse().ok()))
            .collect()
    }
}