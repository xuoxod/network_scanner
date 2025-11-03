# Makefile for common tasks

.PHONY: build-all release-all clean

build-all:
	@echo "Building all crates (debug)..."
	@for m in crates/*/Cargo.toml; do \
	  echo "Building $$m"; \
	  cargo build --manifest-path "$$m" || exit 1; \
	done

release-all:
	@echo "Building all crates (release)..."
	@for m in crates/*/Cargo.toml; do \
	  echo "Building $$m (release)"; \
	  cargo build --manifest-path "$$m" --release || exit 1; \
	done

clean:
	@echo "Cleaning target directories..."
	@for d in crates/*/target; do \
	  if [ -d "$$d" ]; then rm -rf "$$d"; fi; \
	done
