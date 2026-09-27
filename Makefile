# Makefile for network_scanner

.PHONY: build-all release-all test musl clean

build-all:
	@echo "Building all workspace crates (debug)..."
	cargo build

release-all:
	@echo "Building all workspace crates (release)..."
	cargo build --release

test:
	@echo "Running all tests..."
	cargo test --workspace

musl:
	@echo "Building static musl binaries..."
	cargo build --release --target x86_64-unknown-linux-musl

clean:
	@echo "Cleaning target directories..."
	cargo clean
	@for d in crates/*/target; do \
	  if [ -d "$$d" ]; then rm -rf "$$d"; fi; \
	done
