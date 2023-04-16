#!/usr/bin/env sh

CARGO_INCREMENTAL=0 RUSTFLAGS='-Cinstrument-coverage' LLVM_PROFILE_FILE='cargo-test-%p-%m.profraw' cargo test --profile coverage

# Generate `lcov` file which IDEs can read and inline.
grcov . --binary-path ./target/coverage/deps/ -s . -t lcov --branch --ignore-not-existing --ignore '../*' --ignore "/*" -o target/coverage/tests.lcov

# Generate HTML page to show coverage
grcov . --binary-path ./target/coverage/deps/ -s . -t html --branch --ignore-not-existing --ignore '../*' --ignore "/*" -o target/coverage/html

find . -name "*.profraw" -type f -delete
