# SELinux Policy Makefile for Autodns

POLICY_MODULE = autodns
POLICY_VERSION = 1.0.0

# Files
TE_FILE = $(POLICY_MODULE).te
MOD_FILE = $(POLICY_MODULE).mod
PP_FILE = $(POLICY_MODULE).pp

# Commands
CHECKMODULE = checkmodule
SEMODULE_PACKAGE = semodule_package
SEMODULE = semodule

.PHONY: all clean install uninstall

all: $(PP_FILE)

$(MOD_FILE): $(TE_FILE)
	@echo "Compiling policy module..."
	$(CHECKMODULE) -M -m -o $@ $<

$(PP_FILE): $(MOD_FILE)
	@echo "Creating policy package..."
	$(SEMODULE_PACKAGE) -o $@ -m $<

install: $(PP_FILE)
	@echo "Installing SELinux policy module..."
	$(SEMODULE) -i $(PP_FILE)
	@echo "Policy installed successfully!"
	@echo "Verify with: semodule -l | grep $(POLICY_MODULE)"

uninstall:
	@echo "Removing SELinux policy module..."
	$(SEMODULE) -r $(POLICY_MODULE) || echo "Policy not installed"

clean:
	@echo "Cleaning up..."
	rm -f $(MOD_FILE) $(PP_FILE)
	@echo "Cleanup complete"

check:
	@echo "Checking for recent SELinux denials..."
	@ausearch -m avc -ts recent | grep $(POLICY_MODULE) || echo "No denials found"

help:
	@echo "Autodns SELinux Policy Makefile"
	@echo ""
	@echo "Targets:"
	@echo "  all       - Compile the policy (default)"
	@echo "  install   - Install the policy module"
	@echo "  uninstall - Remove the policy module"
	@echo "  clean     - Remove compiled files"
	@echo "  check     - Check for recent SELinux denials"
	@echo "  help      - Show this help message"
	@echo ""
	@echo "Usage:"
	@echo "  make          # Compile policy"
	@echo "  make install  # Install policy (requires root)"
	@echo "  make check    # Check for denials (requires root)"
