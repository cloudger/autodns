use std::fs;
use std::net::IpAddr;
use std::path::PathBuf;
use tempfile::TempDir;

/// Helper to create a temporary config file for testing
///
/// # Arguments
/// * `temp_dir` - Temporary directory to store config and resolv.conf
/// * `mode` - Operating mode: "firstonline" or "benchmark"
/// * `dns_servers` - List of DNS servers as (name, address) tuples
/// * `timeout_seconds` - DNS query timeout in seconds
///
/// # Returns
/// Path to the created config file
pub fn create_test_config(
    temp_dir: &TempDir,
    mode: &str,
    dns_servers: Vec<(&str, &str)>,
    timeout_seconds: u64,
) -> PathBuf {
    let config_path = temp_dir.path().join("config.yaml");
    let resolv_path = temp_dir.path().join("resolv.conf");

    let mut config_content = String::from("dns_servers:\n");
    for (name, address) in dns_servers {
        config_content.push_str(&format!("  - name: \"{}\"\n", name));
        config_content.push_str(&format!("    address: \"{}\"\n", address));
    }

    config_content.push_str(&format!("\nmode: {}\n", mode));
    config_content.push_str("execution_interval_seconds: 120\n");
    config_content.push_str(&format!("timeout_seconds: {}\n", timeout_seconds));
    config_content.push_str(&format!("resolv_conf_path: \"{}\"\n", resolv_path.display()));

    fs::write(&config_path, config_content).expect("Failed to write test config");

    config_path
}

/// Helper to read resolv.conf and extract DNS IP addresses
///
/// # Arguments
/// * `path` - Path to the resolv.conf file
///
/// # Returns
/// Vector of IP addresses found in the nameserver entries
pub fn read_resolv_conf(path: &PathBuf) -> Vec<IpAddr> {
    let content = fs::read_to_string(path).expect("Failed to read resolv.conf");
    content
        .lines()
        .filter(|line| line.starts_with("nameserver "))
        .filter_map(|line| {
            line.split_whitespace()
                .nth(1)
                .and_then(|ip| ip.parse::<IpAddr>().ok())
        })
        .collect()
}
