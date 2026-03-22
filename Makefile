# Root Makefile for Rust + Lean 4 Formal Verification Tutorials
#
# Common targets:
#   make check        - Run all checks (format, lint, test) across all tutorials
#   make test         - Run all tests
#   make lint         - Run clippy on all tutorials
#   make format-check - Check formatting without modifying files
#   make format       - Auto-format all Rust code
#   make clean        - Remove all build artifacts

TUTORIALS := 01-setup-hello-proof \
             02-rpn-calculator \
             03-infix-calculator \
             04-state-machines \
             05-message-protocol \
             06-buffer-management \
             07-tui-core \
             08-llm-client-core \
             09-agent-reasoning \
             10-multi-agent-orchestrator \
             11-full-integration

TUTORIAL_DIRS := $(addprefix tutorials/,$(TUTORIALS))

# Clippy allows for intentional Aeneas-compatible patterns
CLIPPY_ALLOWS := -A clippy::result_unit_err -A clippy::new_without_default

.PHONY: check test lint format-check format clean help

help: ## Show this help
	@grep -E '^[a-zA-Z_-]+:.*?## .*$$' $(MAKEFILE_LIST) | sort | awk 'BEGIN {FS = ":.*?## "}; {printf "\033[36m%-20s\033[0m %s\n", $$1, $$2}'

check: format-check lint test ## Run all checks (format, lint, test)
	@echo ""
	@echo "All checks passed."

test: ## Run all Rust tests
	@echo "=== Running tests ==="
	@failed=0; total=0; \
	for dir in $(TUTORIAL_DIRS); do \
		if [ -f "$$dir/rust/Cargo.toml" ]; then \
			name=$$(basename "$$dir"); \
			cd "$$dir/rust" && \
			passed=$$(cargo test 2>&1 | grep "test result: ok" | grep -oE '[0-9]+ passed' | awk '{sum+=$$1}END{print sum+0}') && \
			echo "  $$name: $$passed tests pass" && \
			total=$$((total + passed)) && \
			cd "$(CURDIR)" || { echo "  $$name: FAILED"; failed=$$((failed + 1)); cd "$(CURDIR)"; }; \
		fi; \
	done; \
	echo ""; \
	echo "Total: $$total tests passed"; \
	if [ $$failed -gt 0 ]; then echo "$$failed tutorial(s) failed"; exit 1; fi

lint: ## Run cargo clippy with warnings as errors
	@echo "=== Running clippy ==="
	@failed=0; \
	for dir in $(TUTORIAL_DIRS); do \
		if [ -f "$$dir/rust/Cargo.toml" ]; then \
			name=$$(basename "$$dir"); \
			cd "$$dir/rust" && \
			cargo clippy -- -D warnings $(CLIPPY_ALLOWS) 2>&1 | tail -1 && \
			echo "  $$name: clippy clean" && \
			cd "$(CURDIR)" || { echo "  $$name: CLIPPY FAILED"; failed=$$((failed + 1)); cd "$(CURDIR)"; }; \
		fi; \
	done; \
	if [ $$failed -gt 0 ]; then echo "$$failed tutorial(s) failed clippy"; exit 1; fi

format-check: ## Check Rust formatting without modifying files
	@echo "=== Checking format ==="
	@failed=0; \
	for dir in $(TUTORIAL_DIRS); do \
		if [ -f "$$dir/rust/Cargo.toml" ]; then \
			name=$$(basename "$$dir"); \
			cd "$$dir/rust" && \
			cargo fmt --check 2>/dev/null && \
			echo "  $$name: formatted" && \
			cd "$(CURDIR)" || { echo "  $$name: NEEDS FORMATTING (run 'make format')"; failed=$$((failed + 1)); cd "$(CURDIR)"; }; \
		fi; \
	done; \
	if [ $$failed -gt 0 ]; then echo "$$failed tutorial(s) need formatting"; exit 1; fi

format: ## Auto-format all Rust code
	@echo "=== Formatting ==="
	@for dir in $(TUTORIAL_DIRS); do \
		if [ -f "$$dir/rust/Cargo.toml" ]; then \
			name=$$(basename "$$dir"); \
			cd "$$dir/rust" && cargo fmt && echo "  $$name: formatted" && cd "$(CURDIR)"; \
		fi; \
	done

build: ## Build all tutorials (warnings as errors)
	@echo "=== Building ==="
	@failed=0; \
	for dir in $(TUTORIAL_DIRS); do \
		if [ -f "$$dir/rust/Cargo.toml" ]; then \
			name=$$(basename "$$dir"); \
			cd "$$dir/rust" && \
			RUSTFLAGS="-D warnings" cargo build 2>&1 | tail -1 && \
			echo "  $$name: build ok" && \
			cd "$(CURDIR)" || { echo "  $$name: BUILD FAILED"; failed=$$((failed + 1)); cd "$(CURDIR)"; }; \
		fi; \
	done; \
	if [ $$failed -gt 0 ]; then echo "$$failed tutorial(s) failed to build"; exit 1; fi

clean: ## Remove all build artifacts
	@echo "=== Cleaning ==="
	@for dir in $(TUTORIAL_DIRS); do \
		if [ -f "$$dir/rust/Cargo.toml" ]; then \
			cd "$$dir/rust" && cargo clean 2>/dev/null && cd "$(CURDIR)"; \
		fi; \
		rm -rf "$$dir/lean/.lake" 2>/dev/null; \
	done
	@find . -name "*.llbc" -delete 2>/dev/null
	@echo "  Done."

# Per-tutorial targets
define TUTORIAL_TARGET
.PHONY: $(1)-test $(1)-lint $(1)-format $(1)-check
$(1)-test:
	@cd tutorials/$(1)/rust && cargo test
$(1)-lint:
	@cd tutorials/$(1)/rust && cargo clippy -- -D warnings $(CLIPPY_ALLOWS)
$(1)-format:
	@cd tutorials/$(1)/rust && cargo fmt
$(1)-check: $(1)-format $(1)-lint $(1)-test
endef

$(foreach t,$(TUTORIALS),$(eval $(call TUTORIAL_TARGET,$(t))))
