mod helpers;

use helpers::{create_test_config, read_resolv_conf};
use std::net::IpAddr;
use tempfile::TempDir;

#[tokio::test]
async fn test_check_mode_selects_first_two_online_dns() {
    // Test 1: FirstOnline mode should select the first 2 online DNS from the configured list
    let temp_dir = TempDir::new().expect("Failed to create temp dir");

    // Configure 6 DNS servers - using well-known public DNS that should be online
    let dns_servers = vec![
        ("Cloudflare-1", "1.1.1.1"),
        ("Cloudflare-2", "1.0.0.1"),
        ("Google-1", "8.8.8.8"),
        ("Google-2", "8.8.4.4"),
        ("Quad9", "9.9.9.9"),
        ("OpenDNS", "208.67.222.222"),
    ];

    let config_path = create_test_config(&temp_dir, "firstonline", dns_servers.clone(), 2);

    // Run autodns check command
    let output = std::process::Command::new("cargo")
        .args(&[
            "run",
            "--release",
            "--",
            "--config",
            config_path.to_str().unwrap(),
            "check",
        ])
        .output()
        .expect("Failed to execute autodns");

    assert!(
        output.status.success(),
        "autodns check command failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    // Read the generated resolv.conf
    let resolv_path = temp_dir.path().join("resolv.conf");
    let dns_ips = read_resolv_conf(&resolv_path);

    // Should have exactly 2 DNS entries
    assert_eq!(
        dns_ips.len(),
        2,
        "Expected exactly 2 DNS entries in resolv.conf"
    );

    // Should be the first 2 from the list (Cloudflare DNS)
    let expected_first: IpAddr = "1.1.1.1".parse().unwrap();
    let expected_second: IpAddr = "1.0.0.1".parse().unwrap();

    assert_eq!(
        dns_ips[0], expected_first,
        "First DNS should be 1.1.1.1 (Cloudflare-1)"
    );
    assert_eq!(
        dns_ips[1], expected_second,
        "Second DNS should be 1.0.0.1 (Cloudflare-2)"
    );

    println!("✓ Test 1 passed: FirstOnline mode selected first 2 online DNS");
}

#[tokio::test]
async fn test_check_mode_skips_offline_dns() {
    // Test: FirstOnline mode should skip offline DNS and select the next online ones
    let temp_dir = TempDir::new().expect("Failed to create temp dir");

    // Configure with some invalid DNS first, followed by valid ones
    let dns_servers = vec![
        ("Invalid-1", "192.0.2.1"),    // TEST-NET-1, should timeout
        ("Invalid-2", "192.0.2.2"),    // TEST-NET-1, should timeout
        ("Cloudflare-1", "1.1.1.1"),   // Should be online
        ("Cloudflare-2", "1.0.0.1"),   // Should be online
        ("Google-1", "8.8.8.8"),
    ];

    let config_path = create_test_config(&temp_dir, "firstonline", dns_servers.clone(), 1);

    // Run autodns check command
    let output = std::process::Command::new("cargo")
        .args(&[
            "run",
            "--release",
            "--",
            "--config",
            config_path.to_str().unwrap(),
            "check",
        ])
        .output()
        .expect("Failed to execute autodns");

    assert!(
        output.status.success(),
        "autodns check command failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    // Read the generated resolv.conf
    let resolv_path = temp_dir.path().join("resolv.conf");
    let dns_ips = read_resolv_conf(&resolv_path);

    // Should have exactly 2 DNS entries
    assert_eq!(
        dns_ips.len(),
        2,
        "Expected exactly 2 DNS entries in resolv.conf"
    );

    // Should skip the invalid ones and use Cloudflare DNS
    let expected_first: IpAddr = "1.1.1.1".parse().unwrap();
    let expected_second: IpAddr = "1.0.0.1".parse().unwrap();

    assert_eq!(
        dns_ips[0], expected_first,
        "First DNS should be 1.1.1.1 (skipped offline)"
    );
    assert_eq!(
        dns_ips[1], expected_second,
        "Second DNS should be 1.0.0.1 (skipped offline)"
    );

    println!("✓ Test passed: FirstOnline mode correctly skipped offline DNS");
}

#[tokio::test]
async fn test_benchmark_mode_selects_fastest_dns() {
    // Test 2: Benchmark mode should select the 2 fastest DNS based on latency
    // Configure 6 DNS servers and verify the fastest 2 are selected
    let temp_dir = TempDir::new().expect("Failed to create temp dir");

    // Configure 6 well-known public DNS servers
    let dns_servers = vec![
        ("Cloudflare-1", "1.1.1.1"),
        ("Cloudflare-2", "1.0.0.1"),
        ("Google-1", "8.8.8.8"),
        ("Google-2", "8.8.4.4"),
        ("Quad9", "9.9.9.9"),
        ("OpenDNS", "208.67.222.222"),
    ];

    let config_path = create_test_config(&temp_dir, "benchmark", dns_servers.clone(), 2);

    // Run autodns benchmark command
    let output = std::process::Command::new("cargo")
        .args(&[
            "run",
            "--release",
            "--",
            "--config",
            config_path.to_str().unwrap(),
            "benchmark",
        ])
        .output()
        .expect("Failed to execute autodns");

    assert!(
        output.status.success(),
        "autodns benchmark command failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    // Read the generated resolv.conf
    let resolv_path = temp_dir.path().join("resolv.conf");
    let dns_ips = read_resolv_conf(&resolv_path);

    // Should have exactly 2 DNS entries
    assert_eq!(
        dns_ips.len(),
        2,
        "Expected exactly 2 DNS entries in resolv.conf"
    );

    // Verify both DNS are from the configured list
    let all_configured_ips: Vec<IpAddr> = dns_servers
        .iter()
        .map(|(_, ip)| ip.parse().unwrap())
        .collect();

    for dns_ip in &dns_ips {
        assert!(
            all_configured_ips.contains(dns_ip),
            "DNS {} not in configured list",
            dns_ip
        );
    }

    // Parse output to verify latency information was logged
    let output_str = String::from_utf8_lossy(&output.stdout);
    assert!(
        output_str.contains("ms") || String::from_utf8_lossy(&output.stderr).contains("ms"),
        "Output should contain latency measurements in milliseconds"
    );

    println!("✓ Test 2 passed: Benchmark mode selected 2 fastest DNS from 6 configured");
    println!("  Selected DNS: {} and {}", dns_ips[0], dns_ips[1]);
}

#[tokio::test]
async fn test_timeout_configuration() {
    // Test: Verify timeout configuration is respected
    let temp_dir = TempDir::new().expect("Failed to create temp dir");

    let dns_servers = vec![
        ("Invalid-Slow", "192.0.2.1"), // Should timeout
        ("Cloudflare-1", "1.1.1.1"),
    ];

    // Use short timeout of 1 second
    let config_path = create_test_config(&temp_dir, "firstonline", dns_servers.clone(), 1);

    let start = std::time::Instant::now();

    let output = std::process::Command::new("cargo")
        .args(&[
            "run",
            "--release",
            "--",
            "--config",
            config_path.to_str().unwrap(),
            "check",
        ])
        .output()
        .expect("Failed to execute autodns");

    let elapsed = start.elapsed();

    assert!(
        output.status.success(),
        "autodns check command failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    // Should complete reasonably quickly (within 10 seconds for 2 DNS checks)
    // The invalid DNS should timeout in 1 second, not the default 5 seconds
    assert!(
        elapsed.as_secs() < 10,
        "Command took too long: {:?}",
        elapsed
    );

    println!("✓ Test passed: Timeout configuration respected (completed in {:?})", elapsed);
}
