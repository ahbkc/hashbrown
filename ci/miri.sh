#!/usr/bin/env sh
# -e表示`set -e`后面的命令一旦运行失败就终止后续命令的执行, -x用于显示出命令与执行结果
set -ex

export CARGO_NET_RETRY=5
export CARGO_NET_TIMEOUT=10

MIRI_NIGHTLY=nightly-$(curl -s https://rust-lang.github.io/rustup-components-history/x86_64-unknown-linux-gnu/miri)
echo "Installing latest nightly with Miri: $MIRI_NIGHTLY"
rustup default "$MIRI_NIGHTLY"

rustup component add miri
cargo miri setup

cargo miri test
