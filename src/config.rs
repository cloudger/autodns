use serde::{Deserialize, Serialize};
use std::net::IpAddr;
use std::path::Path;
use anyhow::{Context, Result, bail};
use std::collections::HashSet;

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct Config {
    pub dns_servers: Vec<DnsServer>,
    pub mode: OperationMode,
    pub execution_interval_seconds: u64,
    #[serde(default = "default_timeout_seconds")]
    pub timeout_seconds: u64,
    pub resolv_conf_path: Option<String>,
}

fn default_timeout_seconds() -> u64 {
    2
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct DnsServer {
    pub name: String,
    pub address: IpAddr,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
#[serde(rename_all = "lowercase")]
pub enum OperationMode {
    FirstOnline,  // Select first N online DNS servers from the list
    Benchmark,    // Benchmark and select best DNS servers by latency
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
        // Validate at least 2 DNS servers are configured
        if self.dns_servers.is_empty() {
            bail!("At least one DNS server must be configured");
        }

        if self.dns_servers.len() < 2 {
            bail!(
                "At least 2 DNS servers recommended for redundancy (currently {})",
                self.dns_servers.len()
            );
        }

        // Validate execution interval
        if self.execution_interval_seconds == 0 {
            bail!("execution_interval_seconds must be greater than 0");
        }

        // Warn if interval is too short
        if self.execution_interval_seconds < 30 {
            eprintln!(
                "⚠ WARNING: execution_interval_seconds is very short ({} seconds). \
                This may cause excessive DNS queries. Recommended: 120+ seconds",
                self.execution_interval_seconds
            );
        }

        // Validate timeout
        if self.timeout_seconds == 0 {
            bail!("timeout_seconds must be greater than 0");
        }

        // Warn if timeout is too long
        if self.timeout_seconds > 10 {
            eprintln!(
                "⚠ WARNING: timeout_seconds is very long ({} seconds). \
                DNS queries may take too long. Recommended: 2-5 seconds",
                self.timeout_seconds
            );
        }

        // Check for duplicate DNS addresses
        let mut seen_addresses = HashSet::new();
        let mut duplicates = Vec::new();

        for server in &self.dns_servers {
            if !seen_addresses.insert(server.address) {
                duplicates.push(format!("{} ({})", server.name, server.address));
            }
        }

        if !duplicates.is_empty() {
            bail!(
                "Duplicate DNS server addresses found:\n  - {}",
                duplicates.join("\n  - ")
            );
        }

        // Check for duplicate DNS names
        let mut seen_names = HashSet::new();
        let mut duplicate_names = Vec::new();

        for server in &self.dns_servers {
            if !seen_names.insert(&server.name) {
                duplicate_names.push(server.name.clone());
            }
        }

        if !duplicate_names.is_empty() {
            eprintln!(
                "⚠ WARNING: Duplicate DNS server names found: {}",
                duplicate_names.join(", ")
            );
        }

        // Validate resolv.conf path if specified
        if let Some(path) = &self.resolv_conf_path {
            if path.is_empty() {
                bail!("resolv_conf_path cannot be empty");
            }

            // Check if parent directory exists
            if let Some(parent) = Path::new(path).parent() {
                if !parent.exists() {
                    bail!(
                        "Parent directory does not exist for resolv_conf_path: {}",
                        parent.display()
                    );
                }
            }
        }

        // Validate mode-specific settings
        match self.mode {
            OperationMode::FirstOnline => {
                if self.execution_interval_seconds > 300 {
                    eprintln!(
                        "⚠ WARNING: In firstonline mode, long intervals ({} seconds) \
                        may delay detection of DNS failures. Recommended: 120 seconds",
                        self.execution_interval_seconds
                    );
                }
            }
            OperationMode::Benchmark => {
                if self.execution_interval_seconds < 300 {
                    eprintln!(
                        "⚠ WARNING: In benchmark mode, short intervals ({} seconds) \
                        may cause excessive DNS load. Recommended: 1800 seconds",
                        self.execution_interval_seconds
                    );
                }
            }
        }

        Ok(())
    }

    pub fn resolv_conf_path(&self) -> &str {
        self.resolv_conf_path.as_deref().unwrap_or("/etc/resolv.conf")
    }
}
