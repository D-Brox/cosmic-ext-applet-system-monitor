name := 'cosmic-ext-applet-system-monitor'
export APPID := 'dev.DBrox.CosmicSystemMonitor'
import "res/packaging.just"

# Default recipe which runs `just build-release`
[private]
default:
    @just --list

# Runs `cargo clean`
clean:
    cargo clean

# Removes vendored dependencies
clean-vendor:
    rm -rf .cargo vendor vendor.tar

# Runs `cargo clean` and removes vendored dependencies
clean-dist: clean clean-vendor

# Runs `cargo fmt`
fmt: 
    cargo fmt

# Runs a clippy check
check *args: fmt
    cargo clippy --all-features {{args}} -- -W clippy::pedantic

# Runs a clippy check with JSON message format
check-json: (check '--message-format=json')

# Run with args
run *args:
    cargo run {{args}}

# Run with debug logs
run-logs *args:
    env RUST_LOG=cosmic_tasks=info RUST_BACKTRACE=full cargo run --release {{args}}

spellcheck *args:
	@codespell --skip="./i18n" --skip="./.git" --skip="./target" --builtin clear,rare,informal,code --ignore-words-list mut,crate {{args}}
	@echo Spellings look good!
