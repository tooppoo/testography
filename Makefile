
.PHONY: check
check: lint fmt test build

.PHONY: test
test:
		cargo llvm-cov --locked --all-features --workspace --no-report
		cargo llvm-cov report --codecov --output-path cov.json \
				--ignore-filename-regex 'component/builtin/(evaluator|reporter)'
		cargo llvm-cov report \
				--fail-under-functions 80 \
				--fail-under-lines 80 \
				--fail-under-file-lines 80 \
				--fail-under-regions 80 \
				--ignore-filename-regex 'component/builtin/(evaluator|reporter)'
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
