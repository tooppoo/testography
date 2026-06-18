
.PHONY: check
check:
		cargo fmt --all --check
		cargo clippy --all-targets --locked -- -D warnings
		cargo test --locked
		cargo build --locked

.PHONY: fmt
fmt:
		cargo fmt --all

.PHONY: setup
setup:
		rustup component add rustfmt clippy

.PHONY: build
build:
		cargo build --locked
