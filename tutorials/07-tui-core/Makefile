# Makefile for this tutorial
#
# Targets:
#   make check        - Run all checks (format, lint, test)
#   make test         - Run Rust tests
#   make lint         - Run clippy with warnings as errors
#   make format-check - Check formatting
#   make format       - Auto-format code
#   make build        - Build with warnings as errors
#   make clean        - Remove build artifacts

# Clippy allows for intentional Aeneas-compatible patterns
CLIPPY_ALLOWS := -A clippy::result_unit_err -A clippy::new_without_default

.PHONY: check test lint format-check format build clean help

help: ## Show this help
	@grep -E '^[a-zA-Z_-]+:.*?## .*$$' $(MAKEFILE_LIST) | sort | awk 'BEGIN {FS = ":.*?## "}; {printf "\033[36m%-20s\033[0m %s\n", $$1, $$2}'

check: format-check lint test ## Run all checks

test: ## Run Rust tests
	cd rust && cargo test

lint: ## Run clippy (warnings as errors)
	cd rust && cargo clippy -- -D warnings $(CLIPPY_ALLOWS)

format-check: ## Check formatting without changes
	cd rust && cargo fmt --check

format: ## Auto-format Rust code
	cd rust && cargo fmt

build: ## Build with warnings as errors
	cd rust && RUSTFLAGS="-D warnings" cargo build

clean: ## Remove build artifacts
	cd rust && cargo clean
	rm -rf lean/.lake
	find . -name "*.llbc" -delete 2>/dev/null
