use serde::{Deserialize, Serialize};
use std::net::IpAddr;
use std::path::Path;
use anyhow::{Context, Result};

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct Config {
    pub dns_servers: Vec<DnsServer>,
    pub mode: OperationMode,
    pub check_interval_seconds: u64,
    pub benchmark_interval_seconds: u64,
    pub resolv_conf_path: Option<String>,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct DnsServer {
    pub name: String,
    pub address: IpAddr,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
#[serde(rename_all = "lowercase")]
pub enum OperationMode {
    Check,      // Only check online/offline
    Benchmark,  // Benchmark and select best DNS servers
}

impl Config {
    pub fn from_file<P: AsRef<Path>>(path: P) -> Result<Self> {
        let content = std::fs::read_to_string(path)
            .context("Failed to read configuration file")?;

        let config: Config = serde_yaml::from_str(&content)
            .context("Failed to parse YAML configuration")?;

        config.validate()?;

        Ok(config)
    }

    fn validate(&self) -> Result<()> {
        if self.dns_servers.is_empty() {
            anyhow::bail!("At least one DNS server must be configured");
        }

        if self.check_interval_seconds == 0 {
            anyhow::bail!("check_interval_seconds must be greater than 0");
        }

        if matches!(self.mode, OperationMode::Benchmark) && self.benchmark_interval_seconds == 0 {
            anyhow::bail!("benchmark_interval_seconds must be greater than 0 in benchmark mode");
        }

        Ok(())
    }

    pub fn resolv_conf_path(&self) -> &str {
        self.resolv_conf_path.as_deref().unwrap_or("/etc/resolv.conf")
    }
}
