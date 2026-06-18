
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
		rustup component add rustfmt clippy

.PHONY: build
build:
		cargo build --locked
