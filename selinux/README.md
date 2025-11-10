# SELinux Policy for Autodns

This directory contains the SELinux policy module for Autodns.

## Files

- `autodns.te` - Policy source file (Type Enforcement)
- `Makefile` - Makefile to compile the policy
- `autodns.fc` - File contexts (optional)

## Installation

### Method 1: Using Makefile (Recommended)

```bash
cd selinux/
make
sudo make install
```

### Method 2: Manual compilation

```bash
cd selinux/

# Compile the policy
checkmodule -M -m -o autodns.mod autodns.te
semodule_package -o autodns.pp -m autodns.mod

# Install the policy
sudo semodule -i autodns.pp

# Verify installation
sudo semodule -l | grep autodns
```

## Verification

After installation, verify the policy is loaded:

```bash
# Check if module is installed
sudo semodule -l | grep autodns

# Should output:
# autodns
```

## Uninstallation

To remove the policy module:

```bash
sudo semodule -r autodns
```

## Troubleshooting

If Autodns is still being blocked after installing the policy:

1. Check for denials:
   ```bash
   sudo ausearch -m avc -ts recent | grep autodns
   ```

2. Generate additional policy rules:
   ```bash
   sudo ausearch -m avc -ts recent | grep autodns | audit2allow -M autodns_additional
   sudo semodule -i autodns_additional.pp
   ```

3. View what permissions are being denied:
   ```bash
   sudo ausearch -m avc -ts recent | grep autodns | audit2why
   ```

## Testing

Test the policy in permissive mode first:

```bash
# Set autodns domain to permissive (logs denials but doesn't block)
sudo semanage permissive -a unconfined_service_t

# Test autodns
sudo systemctl restart autodns
sudo systemctl status autodns

# Check for denials
sudo ausearch -m avc -ts recent | grep autodns

# If no issues, set back to enforcing
sudo semanage permissive -d unconfined_service_t
```

## Notes

- This policy is designed for RHEL/Rocky Linux/Fedora with default SELinux configuration
- The policy allows the service to manage `/etc/resolv.conf` and query DNS servers
- For production use, you may want to create a custom SELinux domain specifically for autodns instead of using `unconfined_service_t`

## Creating a Custom Domain (Advanced)

For production environments, consider creating a dedicated SELinux domain:

```te
# Define a new domain for autodns
type autodns_t;
type autodns_exec_t;

init_daemon_domain(autodns_t, autodns_exec_t)

# Then define specific permissions for autodns_t
# instead of modifying unconfined_service_t
```

This provides better security isolation but requires more complex policy rules.
