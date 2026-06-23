
.PHONY: check
check: lint fmt test build

.PHONY: test
test:
		cargo llvm-cov --locked --all-features --workspace --no-report
		cargo llvm-cov report --codecov --output-path cov.json \
				--ignore-filename-regex 'component/builtin/(evaluator|reporter)|cli/src/main|plugins/parser/rust/src/main|validation/schema'
		# validation/schema is excluded: its builder functions contain map_err closures on
		# schema compilation that are unreachable at runtime because all schemas are hardcoded
		# via include_str! and are always syntactically valid at build time.
		# cli/src/main and plugins/parser/rust/src/main are excluded: both are thin binary entry
		# points (arg parsing / stdin→stdout dispatch) with no behavioral logic to test.
		cargo llvm-cov report \
				--fail-under-functions 80 \
				--fail-under-lines 80 \
				--fail-under-file-lines 80 \
				--fail-under-regions 80 \
				--ignore-filename-regex 'component/builtin/(evaluator|reporter)|cli/src/main|plugins/parser/rust/src/main|validation/schema'
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
