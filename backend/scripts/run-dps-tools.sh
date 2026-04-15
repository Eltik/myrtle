#!/bin/sh
set -e
DEPS=$(python3 -c "import tomllib; print(' '.join(tomllib.load(open('external/ArknightsDpsCompare/pyproject.toml', 'rb'))['project']['dependencies']))")
PIP_BREAK_SYSTEM_PACKAGES=1 pip3 install $DEPS

cargo run --bin generate-dps
cargo clippy --fix --allow-dirty --all-targets --all-features --allow-no-vcs -- -D warnings
cargo fmt --all
