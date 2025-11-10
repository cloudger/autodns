# SELinux Configuration for Autodns

## Overview

SELinux (Security-Enhanced Linux) provides mandatory access control (MAC) that can block Autodns from modifying `/etc/resolv.conf` even when running as root.

## Symptoms of SELinux Blocking Autodns

If you see these errors in the logs:

```
Error: ✗ No write permission for /etc/resolv.conf
Error: Cannot create temporary files in /etc
```

And SELinux is in enforcing mode:

```bash
$ getenforce
Enforcing
```

Then SELinux is likely blocking the operation.

## Solution Options

### Option 1: Create a Custom SELinux Policy (Recommended)

This is the most secure approach. It creates a specific policy allowing Autodns to modify `/etc/resolv.conf`.

#### Step 1: Enable audit logging and reproduce the issue

```bash
# Start the service and let it fail
sudo systemctl start autodns

# Check for SELinux denials
sudo ausearch -m avc -ts recent | grep autodns
```

#### Step 2: Generate a policy module

```bash
# Generate policy from audit logs
sudo ausearch -m avc -ts recent | grep autodns | audit2allow -M autodns_resolv

# This creates two files:
# - autodns_resolv.te (policy source)
# - autodns_resolv.pp (compiled policy)
```

#### Step 3: Review and install the policy

```bash
# Review the policy (optional but recommended)
cat autodns_resolv.te

# Install the policy module
sudo semodule -i autodns_resolv.pp

# Verify installation
sudo semodule -l | grep autodns
```

#### Step 4: Restart the service

```bash
sudo systemctl restart autodns
sudo systemctl status autodns
```

### Option 2: Use a Pre-configured Policy

We provide a basic SELinux policy for common scenarios:

```bash
# Install the policy
sudo semodule -i selinux/autodns.pp

# Restart the service
sudo systemctl restart autodns
```

### Option 3: Add File Context for the Binary

Label the Autodns binary to allow network operations:

```bash
# Add file context
sudo semanage fcontext -a -t bin_t "/usr/local/bin/autodns"

# Apply the context
sudo restorecon -v /usr/local/bin/autodns

# Verify
ls -Z /usr/local/bin/autodns
```

### Option 4: Use permissive mode for Autodns only (Testing)

This allows Autodns to run while still logging denials (useful for debugging):

```bash
# Find the domain autodns is running in
ps -eZ | grep autodns

# Make that domain permissive (example: if it's 'unconfined_service_t')
sudo semanage permissive -a unconfined_service_t
```

**Note:** This is only for testing. Don't use in production.

### Option 5: Temporary Disable SELinux (NOT Recommended)

**WARNING:** Only use this for testing. Never disable SELinux in production.

```bash
# Set SELinux to permissive mode temporarily
sudo setenforce 0

# Verify
getenforce

# To re-enable
sudo setenforce 1
```

## Creating a Custom Policy from Scratch

If the auto-generated policy doesn't work, create a custom one:

### 1. Create policy file: `autodns.te`

```te
policy_module(autodns, 1.0.0)

require {
    type unconfined_service_t;
    type etc_t;
    type net_conf_t;
    class file { create write unlink rename open read getattr setattr };
    class dir { add_name remove_name write };
}

# Allow autodns to manage /etc/resolv.conf
allow unconfined_service_t net_conf_t:file { create write unlink rename open read getattr setattr };
allow unconfined_service_t etc_t:dir { add_name remove_name write };
allow unconfined_service_t etc_t:file { create write unlink rename open read getattr setattr };
```

### 2. Compile and install

```bash
# Compile the policy
checkmodule -M -m -o autodns.mod autodns.te
semodule_package -o autodns.pp -m autodns.mod

# Install
sudo semodule -i autodns.pp

# Verify
sudo semodule -l | grep autodns
```

## Troubleshooting

### Check if SELinux is blocking

```bash
# Check SELinux status
sestatus

# Check for denials
sudo ausearch -m avc -ts today | grep autodns

# Or using audit2why for human-readable output
sudo ausearch -m avc -ts today | grep autodns | audit2why
```

### View current SELinux context

```bash
# Binary context
ls -Z /usr/local/bin/autodns

# Service process context
ps -eZ | grep autodns

# Config file context
ls -Z /etc/autodns/config.yaml

# Target file context
ls -Z /etc/resolv.conf
```

### Check what permissions are needed

```bash
# Generate human-readable report
sudo ausearch -m avc -ts recent | audit2why

# Generate full policy with explanations
sudo ausearch -m avc -ts recent | audit2allow -w
```

### Remove the custom policy (if needed)

```bash
# List installed modules
sudo semodule -l | grep autodns

# Remove module
sudo semodule -r autodns_resolv
```

## Systemd Service with SELinux

The systemd service file should have minimal hardening when SELinux is active:

```ini
[Service]
Type=simple
User=root
Group=root

# With SELinux, we can be more permissive in systemd
# because SELinux provides the actual security layer
NoNewPrivileges=true
PrivateTmp=true
ProtectHome=true

# Don't use ProtectSystem with SELinux
# Let SELinux handle the access control
```

## Rocky Linux / RHEL / Fedora Specific

These distributions ship with SELinux enabled by default:

```bash
# Install SELinux utilities if not present
sudo dnf install policycoreutils-python-utils audit

# Enable audit daemon
sudo systemctl enable --now auditd
```

## Testing Your SELinux Configuration

After configuring SELinux:

```bash
# 1. Ensure SELinux is enforcing
sudo setenforce 1

# 2. Restart autodns
sudo systemctl restart autodns

# 3. Check status
sudo systemctl status autodns

# 4. Verify resolv.conf is being updated
sudo journalctl -u autodns -f

# 5. Check for any remaining denials
sudo ausearch -m avc -ts recent | grep autodns
```

## Best Practices

1. ✅ **Always use enforcing mode in production**
2. ✅ **Create specific policies rather than disabling SELinux**
3. ✅ **Test policies in permissive mode first**
4. ✅ **Monitor audit logs regularly**
5. ✅ **Document custom policies for team members**
6. ❌ **Never run `setenforce 0` in production**
7. ❌ **Never disable SELinux permanently**

## Additional Resources

- [SELinux Project Wiki](https://selinuxproject.org/page/Main_Page)
- [Red Hat SELinux Guide](https://access.redhat.com/documentation/en-us/red_hat_enterprise_linux/8/html/using_selinux/)
- [Fedora SELinux FAQ](https://docs.fedoraproject.org/en-US/quick-docs/selinux-getting-started/)

## Support

If you encounter SELinux issues not covered here:

1. Collect audit logs: `sudo ausearch -m avc -ts today | grep autodns > selinux-denials.log`
2. Generate suggested policy: `cat selinux-denials.log | audit2allow -M autodns_custom`
3. Open an issue with the generated policy and denial logs
