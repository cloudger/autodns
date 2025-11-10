mod config;
mod dns_checker;
mod resolv_conf;

use anyhow::Result;
use clap::{Parser, Subcommand};
use config::{Config, OperationMode};
use dns_checker::{select_best_dns, DnsChecker};
use log::{error, info, warn};
use resolv_conf::ResolvConfManager;
use std::net::IpAddr;
use std::time::Duration;
use tokio::time;

#[derive(Parser)]
#[command(name = "autodns")]
#[command(about = "DNS server manager with monitoring and benchmarking", long_about = None)]
struct Cli {
    #[arg(short, long, default_value = "config.yaml")]
    config: String,

    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand)]
enum Commands {
    /// Run the Autodns daemon
    Run,
    /// Run once based on config mode (check or benchmark) and exit
    Check,
    /// Force benchmark mode once and exit (ignores config mode)
    Benchmark,
}

#[tokio::main]
async fn main() -> Result<()> {
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info")).init();

    let cli = Cli::parse();

    info!("Starting Autodns");

    // Load configuration
    let config = Config::from_file(&cli.config)?;
    info!("Loaded configuration from {}", cli.config);

    match cli.command {
        Some(Commands::Run) | None => {
            run_daemon(config).await?;
        }
        Some(Commands::Check) => {
            // Run one-time check/benchmark based on config.mode
            run_once(config).await?;
        }
        Some(Commands::Benchmark) => {
            // Force benchmark mode for this command
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

    let checker = DnsChecker::new()
        .with_timeout(Duration::from_secs(config.timeout_seconds));

    let servers: Vec<_> = config
        .dns_servers
        .iter()
        .map(|s| (s.address, s.name.clone()))
        .collect();

    let mut execution_interval = time::interval(Duration::from_secs(config.execution_interval_seconds));

    // Run initial check/benchmark based on mode
    match config.mode {
        OperationMode::FirstOnline => {
            info!("Running initial health check (FirstOnline mode)");
            let results = checker.check_multiple(&servers).await;
            display_check_results(&results);

            // Update resolv.conf with first 2 online servers
            let selected_dns = select_first_online_dns(&results, 2);
            if !selected_dns.is_empty() {
                if let Err(e) = resolv_manager.update_dns_servers(&selected_dns) {
                    error!("Failed to update resolv.conf: {}", e);
                } else {
                    info!("Updated resolv.conf with first {} online DNS servers", selected_dns.len());
                }
            } else {
                warn!("No online DNS servers found!");
            }
        }
        OperationMode::Benchmark => {
            info!("Running initial benchmark");
            let results = checker.benchmark_multiple(&servers).await;

            // Update resolv.conf with best servers by latency
            let best_dns = select_best_dns(&results, 2);
            display_benchmark_results_with_selection(&results, &best_dns);

            if !best_dns.is_empty() {
                if let Err(e) = resolv_manager.update_dns_servers(&best_dns) {
                    error!("Failed to update resolv.conf: {}", e);
                } else {
                    info!("Updated resolv.conf with {} fastest DNS servers", best_dns.len());
                }
            } else {
                warn!("No online DNS servers found!");
            }
        }
    }

    loop {
        execution_interval.tick().await;

        match config.mode {
            OperationMode::FirstOnline => {
                info!("Running scheduled health check (FirstOnline mode)");
                let results = checker.check_multiple(&servers).await;
                display_check_results(&results);

                let selected_dns = select_first_online_dns(&results, 2);
                if !selected_dns.is_empty() {
                    if let Err(e) = resolv_manager.update_dns_servers(&selected_dns) {
                        error!("Failed to update resolv.conf: {}", e);
                    } else {
                        info!("Updated resolv.conf with first {} online DNS servers", selected_dns.len());
                    }
                } else {
                    error!("ALERT: All DNS servers are offline!");
                }
            }
            OperationMode::Benchmark => {
                info!("Running scheduled benchmark (Benchmark mode)");
                let results = checker.benchmark_multiple(&servers).await;

                let best_dns = select_best_dns(&results, 2);
                display_benchmark_results_with_selection(&results, &best_dns);

                if !best_dns.is_empty() {
                    if let Err(e) = resolv_manager.update_dns_servers(&best_dns) {
                        error!("Failed to update resolv.conf: {}", e);
                    } else {
                        info!("Updated resolv.conf with {} fastest DNS servers", best_dns.len());
                    }
                } else {
                    warn!("No online DNS servers found in benchmark!");
                }
            }
        }
    }
}

async fn run_once(config: Config) -> Result<()> {
    info!("Running one-time operation in {:?} mode", config.mode);

    match config.mode {
        OperationMode::FirstOnline => check_once(config).await,
        OperationMode::Benchmark => benchmark_once(config).await,
    }
}

async fn check_once(config: Config) -> Result<()> {
    info!("Running one-time health check");

    let checker = DnsChecker::new()
        .with_timeout(Duration::from_secs(config.timeout_seconds));
    let servers: Vec<_> = config
        .dns_servers
        .iter()
        .map(|s| (s.address, s.name.clone()))
        .collect();

    let results = checker.check_multiple(&servers).await;
    display_check_results(&results);

    let online_count = results.iter().filter(|r| r.is_online).count();
    println!(
        "\nSummary: {}/{} DNS servers are online",
        online_count,
        results.len()
    );

    // Update resolv.conf with first 2 online servers
    let selected_dns = select_first_online_dns(&results, 2);
    if !selected_dns.is_empty() {
        println!("\nSelected DNS servers (first {} online):", selected_dns.len());
        for dns in &selected_dns {
            if let Some(result) = results.iter().find(|r| r.address == *dns) {
                println!("  {} ({})", result.name, result.address);
            }
        }

        let resolv_manager = ResolvConfManager::new(config.resolv_conf_path().to_string());
        resolv_manager.check_permissions()?;
        resolv_manager.update_dns_servers(&selected_dns)?;
        println!("\nUpdated {} with selected DNS servers", config.resolv_conf_path());
    } else {
        warn!("No online DNS servers found!");
    }

    Ok(())
}

async fn benchmark_once(config: Config) -> Result<()> {
    info!("Running one-time benchmark");

    let checker = DnsChecker::new()
        .with_timeout(Duration::from_secs(config.timeout_seconds));
    let servers: Vec<_> = config
        .dns_servers
        .iter()
        .map(|s| (s.address, s.name.clone()))
        .collect();

    let results = checker.benchmark_multiple(&servers).await;

    let best_dns = select_best_dns(&results, 2);
    display_benchmark_results_with_selection(&results, &best_dns);

    if !best_dns.is_empty() {
        println!("\n✓ Selected 2 fastest DNS servers:");
        for dns in &best_dns {
            if let Some(result) = results.iter().find(|r| r.address == *dns) {
                if let Some(latency) = result.latency_ms {
                    println!("  • {} ({}) - {:.2}ms", result.name, result.address, latency);
                }
            }
        }

        let resolv_manager = ResolvConfManager::new(config.resolv_conf_path().to_string());
        resolv_manager.check_permissions()?;
        resolv_manager.update_dns_servers(&best_dns)?;
        println!("\n✓ Updated {} with fastest DNS servers", config.resolv_conf_path());
    } else {
        warn!("No online DNS servers found!");
    }

    Ok(())
}

fn display_check_results(results: &[dns_checker::DnsCheckResult]) {
    println!("\n=== DNS Health Check Results ===");
    for result in results {
        let status = if result.is_online {
            "ONLINE"
        } else {
            "OFFLINE"
        };
        println!(
            "{:15} ({:40}) - {}",
            result.name,
            result.address.to_string(),
            status
        );
    }
}


fn display_benchmark_results_with_selection(
    results: &[dns_checker::DnsCheckResult],
    selected: &[IpAddr],
) {
    println!("\n=== DNS Benchmark Results ===");

    // Sort results by latency for display
    let mut sorted_results: Vec<_> = results.iter().collect();
    sorted_results.sort_by(|a, b| {
        match (a.latency_ms, b.latency_ms) {
            (Some(lat_a), Some(lat_b)) => lat_a.partial_cmp(&lat_b).unwrap_or(std::cmp::Ordering::Equal),
            (Some(_), None) => std::cmp::Ordering::Less,
            (None, Some(_)) => std::cmp::Ordering::Greater,
            (None, None) => std::cmp::Ordering::Equal,
        }
    });

    for result in sorted_results {
        let is_selected = selected.contains(&result.address);
        let marker = if is_selected { "→" } else { " " };

        if let Some(latency) = result.latency_ms {
            println!(
                "{} {:15} ({:40}) - {:.2}ms",
                marker,
                result.name,
                result.address.to_string(),
                latency
            );
        } else {
            println!(
                "{} {:15} ({:40}) - FAILED",
                marker,
                result.name,
                result.address.to_string()
            );
        }
    }
}

/// Select first N online DNS servers from the list (in order)
fn select_first_online_dns(results: &[dns_checker::DnsCheckResult], count: usize) -> Vec<IpAddr> {
    results
        .iter()
        .filter(|r| r.is_online)
        .take(count)
        .map(|r| r.address)
        .collect()
}
