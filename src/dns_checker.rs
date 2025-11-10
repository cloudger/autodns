use std::net::IpAddr;
use std::time::{Duration, Instant};
use trust_dns_resolver::config::{NameServerConfig, Protocol, ResolverConfig, ResolverOpts};
use trust_dns_resolver::TokioAsyncResolver;
use anyhow::Result;
use log::{debug, info, warn};

#[derive(Debug, Clone)]
pub struct DnsCheckResult {
    pub address: IpAddr,
    pub name: String,
    pub is_online: bool,
    pub latency_ms: Option<f64>,
}

pub struct DnsChecker {
    test_domain: String,
    timeout: Duration,
}

impl DnsChecker {
    pub fn new() -> Self {
        Self {
            test_domain: "google.com".to_string(),
            timeout: Duration::from_secs(5),
        }
    }

    pub fn with_timeout(mut self, timeout: Duration) -> Self {
        self.timeout = timeout;
        self
    }

    /// Check if a DNS server is online
    pub async fn check_dns_online(&self, address: IpAddr, name: &str) -> DnsCheckResult {
        debug!("Checking DNS server: {} ({})", name, address);

        match self.perform_dns_query(address).await {
            Ok(_) => {
                info!("DNS server {} ({}) is ONLINE", name, address);
                DnsCheckResult {
                    address,
                    name: name.to_string(),
                    is_online: true,
                    latency_ms: None,
                }
            }
            Err(e) => {
                warn!("DNS server {} ({}) is OFFLINE: {}", name, address, e);
                DnsCheckResult {
                    address,
                    name: name.to_string(),
                    is_online: false,
                    latency_ms: None,
                }
            }
        }
    }

    /// Benchmark a DNS server by measuring latency
    pub async fn benchmark_dns(&self, address: IpAddr, name: &str) -> DnsCheckResult {
        debug!("Benchmarking DNS server: {} ({})", name, address);

        let start = Instant::now();
        match self.perform_dns_query(address).await {
            Ok(_) => {
                let latency = start.elapsed();
                let latency_ms = latency.as_secs_f64() * 1000.0;
                info!(
                    "DNS server {} ({}) responded in {:.2}ms",
                    name, address, latency_ms
                );
                DnsCheckResult {
                    address,
                    name: name.to_string(),
                    is_online: true,
                    latency_ms: Some(latency_ms),
                }
            }
            Err(e) => {
                warn!("DNS server {} ({}) failed benchmark: {}", name, address, e);
                DnsCheckResult {
                    address,
                    name: name.to_string(),
                    is_online: false,
                    latency_ms: None,
                }
            }
        }
    }

    async fn perform_dns_query(&self, dns_server: IpAddr) -> Result<()> {
        // Create a custom resolver that only uses the specified DNS server
        let nameserver = NameServerConfig {
            socket_addr: std::net::SocketAddr::new(dns_server, 53),
            protocol: Protocol::Udp,
            tls_dns_name: None,
            trust_negative_responses: true,
            bind_addr: None,
        };

        let mut resolver_config = ResolverConfig::new();
        resolver_config.add_name_server(nameserver);

        let mut resolver_opts = ResolverOpts::default();
        resolver_opts.timeout = self.timeout;
        resolver_opts.attempts = 1;

        let resolver = TokioAsyncResolver::tokio(resolver_config, resolver_opts);

        // Perform a DNS lookup with explicit timeout wrapper
        tokio::time::timeout(
            self.timeout,
            resolver.lookup_ip(&self.test_domain)
        ).await??;

        Ok(())
    }

    /// Check multiple DNS servers in parallel
    pub async fn check_multiple(&self, servers: &[(IpAddr, String)]) -> Vec<DnsCheckResult> {
        use futures::future::join_all;

        let tasks = servers
            .iter()
            .map(|(address, name)| self.check_dns_online(*address, name));

        join_all(tasks).await
    }

    /// Benchmark multiple DNS servers in parallel
    pub async fn benchmark_multiple(&self, servers: &[(IpAddr, String)]) -> Vec<DnsCheckResult> {
        use futures::future::join_all;

        let tasks = servers
            .iter()
            .map(|(address, name)| self.benchmark_dns(*address, name));

        join_all(tasks).await
    }
}

impl Default for DnsChecker {
    fn default() -> Self {
        Self::new()
    }
}

/// Select the best DNS servers based on latency
pub fn select_best_dns(results: &[DnsCheckResult], count: usize) -> Vec<IpAddr> {
    let mut online_servers: Vec<_> = results
        .iter()
        .filter(|r| r.is_online && r.latency_ms.is_some())
        .collect();

    // Sort by latency (lowest first)
    online_servers.sort_by(|a, b| {
        let latency_a = a.latency_ms.unwrap_or(f64::MAX);
        let latency_b = b.latency_ms.unwrap_or(f64::MAX);
        latency_a.partial_cmp(&latency_b).unwrap_or(std::cmp::Ordering::Equal)
    });

    // Take the best ones
    online_servers
        .iter()
        .take(count)
        .map(|r| r.address)
        .collect()
}
