# Autodns

A DNS server manager for Linux with automatic monitoring and benchmarking, written in Rust.

## Problem

You use DNS providers that may stop responding without warning (as happened with Hetzner's IPv6 servers). This program solves this:

- Constantly monitors the health of DNS servers
- Performs automatic benchmarking and selects the fastest ones (Optional)
- Automatically updates `/etc/resolv.conf` with the best servers

## Features

1. **YAML Configuration**: Configurable list of DNS servers
2. **FirstOnline Mode**: Checks if DNS servers are online and selects first 2 available
3. **Benchmark Mode**: Measures latency and selects the 2 fastest DNS servers
4. **Configurable Interval**: Runs the selected mode at configurable intervals
5. **Automatic Update**: Updates `/etc/resolv.conf` with the selected DNS servers

## Installation

### Requirements

- Rust 1.70 or higher
- Any modern Linux distribution
- Root/sudo permissions to edit `/etc/resolv.conf`

### Quick Installation (Recommended)

Use the automated installation script:

```bash
sudo ./install.sh
```

This script will:
- Build the binary with musl target
- Install to `/usr/local/bin/autodns`
- Create configuration directory `/etc/autodns/`
- Copy configuration file
- Install systemd service
- Validate configuration and permissions

### Manual Compilation

```bash
cargo build --release --target x86_64-unknown-linux-musl
```

The binary will be located in `target/x86_64-unknown-linux-musl/release/autodns`

### Manual Install

```bash
sudo cp target/x86_64-unknown-linux-musl/release/autodns /usr/local/bin/
sudo chmod +x /usr/local/bin/autodns
sudo mkdir -p /etc/autodns
sudo cp config.yaml /etc/autodns/
```

## Settings

Create a `config.yaml` file:

```yaml
# List of DNS servers to monitor
dns_servers:
  - name: "Cloudflare-1"
    address: "1.1.1.1"
  - name: "Cloudflare-2"
    address: "1.0.0.1"
  - name: "Cloudflare-IPv6-1"
    address: "2606:4700:4700::1111"
  - name: "Cloudflare-IPv6-2"
    address: "2606:4700:4700::1001"
  - name: "Google-1"
    address: "8.8.8.8"
  - name: "Google-2"
    address: "8.8.4.4"
  - name: "Quad9"
    address: "9.9.9.9"
  - name: "Hetzner-IPv6-1"
    address: "2a01:4ff:ff00::add:1"
  - name: "Hetzner-IPv6-2"
    address: "2a01:4ff:ff00::add:2"

# Operating mode: "firstonline" or "benchmark"
mode: benchmark

# Execution interval (in seconds)
# The program executes the configured mode at this interval
# Recommended: 120 for firstonline, 1800 for benchmark
execution_interval_seconds: 1800

# DNS query timeout (in seconds)
# Default: 2
# How long to wait for a DNS response before considering it offline
timeout_seconds: 2

# Path to resolv.conf (optional, default: /etc/resolv.conf)
resolv_conf_path: "/etc/resolv.conf"
```

## Usage

### Run once based on config mode

The `check` command respects the `mode` setting in your config file:

```bash
# With mode: firstonline in config.yaml - uses first 2 online DNS from the list
# With mode: benchmark in config.yaml - measures latency and uses fastest 2 DNS
sudo autodns --config config.yaml check
```

### Run as daemon

```bash
sudo autodns --config config.yaml run
# or simply
sudo autodns --config config.yaml
```

## Systemd Installation as a Service

1. Copy the service file:

```bash
sudo cp autodns.service /etc/systemd/system/
```

2. Edit the service file if necessary:

```bash
sudo nano /etc/systemd/system/autodns.service
```

3. Create the configuration directory:

```bash
sudo mkdir -p /etc/autodns
sudo cp config.yaml /etc/autodns/
```

4. Enable and start the service:

```bash
sudo systemctl daemon-reload
sudo systemctl enable autodns
sudo systemctl start autodns
```

5. Check the status:

```bash
sudo systemctl status autodns
```

6. View logs:

```bash
sudo journalctl -u autodns -f
```

## Operating Modes

### FirstOnline Mode

- Periodically checks if DNS servers are responding
- **Updates `/etc/resolv.conf` with the first 2 online DNS servers from the list**
- Follows the order of DNS servers as configured in `config.yaml`
- Useful when you want to maintain a specific priority order
- Recommended interval: 120 seconds (2 minutes)

### Benchmark Mode

- Measures the latency of each DNS server
- **Updates `/etc/resolv.conf` with the 2 fastest servers based on latency**
- Useful for automatically optimizing DNS performance
- Recommended interval: 1800 seconds (30 minutes)

## Operation

### FirstOnline Mode Operation

1. Tests each DNS server by querying `google.com`
2. Marks as ONLINE or OFFLINE
3. Selects the first 2 online DNS servers from the configured list
4. Creates a backup of `/etc/resolv.conf`
5. Updates `/etc/resolv.conf` with the selected servers
6. Repeats every `execution_interval_seconds`

### Benchmark Mode Operation

1. Tests each DNS server by measuring response time
2. Measures latency for all servers
3. Sorts by latency (fastest first)
4. Selects the 2 fastest servers
5. Creates a backup of `/etc/resolv.conf`
6. Updates `/etc/resolv.conf` with the fastest servers
7. Repeats every `execution_interval_seconds`

## Security

- Creates an automatic backup of `/etc/resolv.conf` before modifying
- Checks permissions before starting
- Uses a temporary file for atomic writing
- Logs all operations

## Project Structure

```
autodns/
├── src/
│   ├── main.rs           # Main application and CLI
│   ├── config.rs         # YAML configuration parser
│   ├── dns_checker.rs    # Verification and benchmarking logic
│   └── resolv_conf.rs    # /etc/resolv.conf manager
├── Cargo.toml            # Rust Dependencies
├── config.yaml           # Example configuration
├── autodns.service   # Systemd service
└── README.md             # This file
```

### Permission error

```
Error: No write permission for /etc/resolv.conf
```

**Solution**: Run with `sudo`

### DNS are not being updated

1. Check if the service is running:
   ```bash
   sudo systemctl status autodns
   ```

2. Check the logs:
   ```bash
   sudo journalctl -u autodns -n 100
   ```

3. Verify at least 2 DNS servers are online:
   ```bash
   sudo autodns --config config.yaml check
   ```

### All DNS servers are offline.

- Check network connectivity
- Check firewall (UDP port 53)
- Test manually: `dig @1.1.1.1 google.com`

## Usage Examples

### Test Configuration

```bash
# Run once using the mode from config.yaml (firstonline or benchmark)
sudo autodns --config config.yaml check

# Force benchmark mode to see latency results
sudo autodns --config config.yaml benchmark
```

### Continuous monitoring

```bash
# FirstOnline mode: Uses first 2 online DNS from the configured list
mode: firstonline
execution_interval_seconds: 120  # Check and update every 2 minutes

# Benchmark mode: Uses 2 fastest DNS based on latency
mode: benchmark
execution_interval_seconds: 1800  # Benchmark and update every 30 minutes
```

## License

MIT
