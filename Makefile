.PHONY: verify fmt install-cli release

verify:
	cd crates && cargo fmt --check
	cd crates && cargo clippy --workspace --all-targets -- -D warnings
	cd crates && RUST_TEST_THREADS=$$((4 * $$(getconf _NPROCESSORS_ONLN))) cargo test --workspace --no-fail-fast

fmt:
	cd crates && cargo fmt

install-cli:
	cd crates && cargo build --release -p celis
	mkdir -p $(HOME)/.local/bin
	install -m 755 crates/target/release/celis $(HOME)/.local/bin/celis

release:
	sh scripts/release.sh "$(V)"
