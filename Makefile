
.PHONY: check
check: lint fmt test build

.PHONY: test
test:
		cargo llvm-cov clean --workspace
		cargo llvm-cov --locked --all-features --workspace --no-report
		cargo llvm-cov report --codecov --output-path cov.json
		cargo llvm-cov report

.PHONY: fmt
fmt:
		cargo fmt --all --check

.PHONY: lint
lint:
		cargo clippy --all-targets --locked -- -D warnings


.PHONY: setup
setup:
		cargo install cargo-llvm-cov@0.8.7
		rustup component add rustfmt clippy llvm-tools-preview

.PHONY: build
build:
		cargo build --locked
