#!/usr/bin/env bash

set -eux

PROJ_NAME=$(cat Cargo.toml | grep -E "^name" | sed -E 's/name[[:space:]]=[[:space:]]"(.*)"/\1/g' | sed -E 's/-/_/g')
rm -rf target/debug/deps/${PROJ_NAME}-*

export CARGO_INCREMENTAL=0
export RUSTFLAGS="-Zprofile -Ccodegen-units=1 -Copt-level=0 -Clink-dead-code -Coverflow-checks=off -Zpanic_abort_tests -C panic=abort"

cargo build
cargo test

zip -0 ccov.zip `find . \( -name "${PROJ_NAME}*.gc*" -o -name "test-*.gc*" \) -print`
grcov ccov.zip -s . -t lcov --llvm --branch --ignore-not-existing --ignore "/*" --ignore "tests/*" -o lcov.info
rust-covfix -o lcov.info lcov.info

if [ $# = 0 ] || [ "$1" != "ontravis" ]; then
    genhtml -o report/ --show-details --highlight --ignore-errors source --legend lcov.info --branch-coverage
fi

if [ $# -gt 1 ] && [ "$2" = "sendcov" ]; then
    bash <(curl -s https://codecov.io/bash) -f lcov.info -t "${CODECOV_TOKEN}" -cF api
fi
