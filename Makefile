
.PHONY: check
check: fmt test build
		cargo clippy --all-targets --locked -- -D warnings

.PHONY: test
test:
		cargo test --locked --all-features --workspace
		cargo llvm-cov report --codecov --output-path cov.json
		cargo llvm-cov report

.PHONY: fmt
fmt:
		cargo fmt --all --check

.PHONY: setup
setup:
		cargo install cargo-llvm-cov@0.8.7
		rustup component add rustfmt clippy llvm-tools-preview

.PHONY: build
build:
		cargo build --locked
