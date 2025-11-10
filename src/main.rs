mod config;
mod dns_checker;
mod resolv_conf;

use anyhow::Result;
use clap::{Parser, Subcommand};
use config::{Config, OperationMode};
use dns_checker::{DnsChecker, select_best_dns};
use log::{error, info, warn};
use resolv_conf::ResolvConfManager;
use std::time::Duration;
use tokio::time;

#[derive(Parser)]
#[command(name = "dns-manager")]
#[command(about = "DNS server manager with monitoring and benchmarking", long_about = None)]
struct Cli {
    #[arg(short, long, default_value = "config.yaml")]
    config: String,

    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand)]
enum Commands {
    /// Run the DNS manager daemon
    Run,
    /// Check DNS servers once and exit
    Check,
    /// Benchmark DNS servers once and exit
    Benchmark,
}

#[tokio::main]
async fn main() -> Result<()> {
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info")).init();

    let cli = Cli::parse();

    info!("Starting DNS Manager");

    // Load configuration
    let config = Config::from_file(&cli.config)?;
    info!("Loaded configuration from {}", cli.config);

    match cli.command {
        Some(Commands::Run) | None => {
            run_daemon(config).await?;
        }
        Some(Commands::Check) => {
            check_once(config).await?;
        }
        Some(Commands::Benchmark) => {
            benchmark_once(config).await?;
        }
    }

    Ok(())
}

async fn run_daemon(config: Config) -> Result<()> {
    info!("Running in daemon mode");

    let resolv_manager = ResolvConfManager::new(config.resolv_conf_path().to_string());

    // Check permissions before starting
    resolv_manager.check_permissions()?;

    let checker = DnsChecker::new();

    let servers: Vec<_> = config
        .dns_servers
        .iter()
        .map(|s| (s.address, s.name.clone()))
        .collect();

    let mut check_interval = time::interval(Duration::from_secs(config.check_interval_seconds));
    let mut benchmark_interval = time::interval(Duration::from_secs(config.benchmark_interval_seconds));

    // Run initial check/benchmark
    match config.mode {
        OperationMode::Check => {
            info!("Running initial health check");
            let results = checker.check_multiple(&servers).await;
            display_check_results(&results);
        }
        OperationMode::Benchmark => {
            info!("Running initial benchmark");
            let results = checker.benchmark_multiple(&servers).await;
            display_benchmark_results(&results);

            // Update resolv.conf with best servers
            let best_dns = select_best_dns(&results, 2);
            if !best_dns.is_empty() {
                if let Err(e) = resolv_manager.update_dns_servers(&best_dns) {
                    error!("Failed to update resolv.conf: {}", e);
                }
            } else {
                warn!("No online DNS servers found!");
            }
        }
    }

    loop {
        tokio::select! {
            _ = check_interval.tick() => {
                info!("Running scheduled health check");
                let results = checker.check_multiple(&servers).await;
                display_check_results(&results);

                // In check mode, if all DNS are down, warn
                if matches!(config.mode, OperationMode::Check) {
                    let online_count = results.iter().filter(|r| r.is_online).count();
                    if online_count == 0 {
                        error!("ALERT: All DNS servers are offline!");
                    } else {
                        info!("{}/{} DNS servers are online", online_count, results.len());
                    }
                }
            }
            _ = benchmark_interval.tick() => {
                if matches!(config.mode, OperationMode::Benchmark) {
                    info!("Running scheduled benchmark");
                    let results = checker.benchmark_multiple(&servers).await;
                    display_benchmark_results(&results);

                    // Update resolv.conf with best servers
                    let best_dns = select_best_dns(&results, 2);
                    if !best_dns.is_empty() {
                        if let Err(e) = resolv_manager.update_dns_servers(&best_dns) {
                            error!("Failed to update resolv.conf: {}", e);
                        }
                    } else {
                        warn!("No online DNS servers found in benchmark!");
                    }
                }
            }
        }
    }
}

async fn check_once(config: Config) -> Result<()> {
    info!("Running one-time health check");

    let checker = DnsChecker::new();
    let servers: Vec<_> = config
        .dns_servers
        .iter()
        .map(|s| (s.address, s.name.clone()))
        .collect();

    let results = checker.check_multiple(&servers).await;
    display_check_results(&results);

    let online_count = results.iter().filter(|r| r.is_online).count();
    println!("\nSummary: {}/{} DNS servers are online", online_count, results.len());

    Ok(())
}

async fn benchmark_once(config: Config) -> Result<()> {
    info!("Running one-time benchmark");

    let checker = DnsChecker::new();
    let servers: Vec<_> = config
        .dns_servers
        .iter()
        .map(|s| (s.address, s.name.clone()))
        .collect();

    let results = checker.benchmark_multiple(&servers).await;
    display_benchmark_results(&results);

    let best_dns = select_best_dns(&results, 2);
    if !best_dns.is_empty() {
        println!("\nBest DNS servers:");
        for dns in &best_dns {
            if let Some(result) = results.iter().find(|r| r.address == *dns) {
                if let Some(latency) = result.latency_ms {
                    println!("  {} ({}) - {:.2}ms", result.name, result.address, latency);
                }
            }
        }

        if matches!(config.mode, OperationMode::Benchmark) {
            let resolv_manager = ResolvConfManager::new(config.resolv_conf_path().to_string());
            resolv_manager.check_permissions()?;
            resolv_manager.update_dns_servers(&best_dns)?;
            println!("\nUpdated {} with best DNS servers", config.resolv_conf_path());
        }
    } else {
        warn!("No online DNS servers found!");
    }

    Ok(())
}

fn display_check_results(results: &[dns_checker::DnsCheckResult]) {
    println!("\n=== DNS Health Check Results ===");
    for result in results {
        let status = if result.is_online { "ONLINE" } else { "OFFLINE" };
        println!("{:15} ({:40}) - {}", result.name, result.address.to_string(), status);
    }
}

fn display_benchmark_results(results: &[dns_checker::DnsCheckResult]) {
    println!("\n=== DNS Benchmark Results ===");
    for result in results {
        if let Some(latency) = result.latency_ms {
            println!(
                "{:15} ({:40}) - {:.2}ms",
                result.name,
                result.address.to_string(),
                latency
            );
        } else {
            println!(
                "{:15} ({:40}) - FAILED",
                result.name,
                result.address.to_string()
            );
        }
    }
}
