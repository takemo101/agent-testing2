.PHONY: help install build test lint clean release

help: ## ヘルプを表示
	@grep -E '^[a-zA-Z_-]+:.*?## .*$$' $(MAKEFILE_LIST) | awk 'BEGIN {FS = ":.*?## "}; {printf "\033[36m%-20s\033[0m %s\n", $$1, $$2}'

install: ## 依存関係をインストール
	rustup target add x86_64-apple-darwin aarch64-apple-darwin
	cargo install cargo-audit cargo-deny cargo-tarpaulin

build: ## デバッグビルド
	cargo build

build-release: ## リリースビルド（ユニバーサルバイナリ）
	cargo build --release --target x86_64-apple-darwin
	cargo build --release --target aarch64-apple-darwin
	mkdir -p target/release
	lipo -create \
		target/x86_64-apple-darwin/release/pomodoro \
		target/aarch64-apple-darwin/release/pomodoro \
		-output target/release/pomodoro-universal

test: ## テスト実行
	cargo test --all-features

lint: ## Lint実行
	cargo fmt --all -- --check
	cargo clippy --all-targets --all-features -- -D warnings

coverage: ## カバレッジ測定
	cargo tarpaulin --out Html --all-features
	open tarpaulin-report.html

audit: ## セキュリティ監査
	cargo audit
	cargo deny check

clean: ## ビルド成果物削除
	cargo clean

run: ## デーモン起動
	cargo run -- daemon

install-local: build-release ## ローカルにインストール
	sudo cp target/release/pomodoro-universal /usr/local/bin/pomodoro
	sudo chmod +x /usr/local/bin/pomodoro

uninstall-local: ## ローカルからアンインストール
	sudo rm -f /usr/local/bin/pomodoro
