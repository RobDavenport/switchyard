.PHONY: bootstrap fmt lint test test-no-default docs contracts ci runpack showcase-wasm

bootstrap:
	python3 --version
	rustc --version
	cargo --version

fmt:
	cargo fmt --all

lint:
	cargo clippy --workspace --all-targets --all-features -- -D warnings

test:
	cargo test --workspace --all-features
	python3 -m unittest scripts/test_run_prompt_pack.py
	python3 scripts/validate_contract_fixtures.py

test-no-default:
	cargo check --workspace --lib --no-default-features

docs:
	cargo doc --workspace --no-deps

contracts:
	python3 scripts/validate_contract_fixtures.py

runpack:
	python3 scripts/run_prompt_pack.py

showcase-wasm:
	wasm-pack build demo-wasm --target web --release --out-dir www/pkg

ci: fmt lint test test-no-default docs
