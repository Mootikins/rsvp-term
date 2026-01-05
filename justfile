# rsvp-term justfile

default:
    @just --list

build:
    @cargo build 2>&1 | awk '/warning|error|Finished/'

build-release:
    @cargo build --release 2>&1 | awk '/warning|error|Finished/'

test:
    @cargo test 2>&1 | awk '/^test result:|FAILED|panicked/'

test-verbose:
    cargo test -- --nocapture

test-review:
    cargo insta test --review

lint:
    @cargo clippy -- -D warnings 2>&1 | awk '/warning\[|error\[|generated|could not compile/'

fmt:
    cargo fmt

fmt-check:
    @cargo fmt -- --check

ci: fmt-check lint test

run *ARGS:
    @cargo run --quiet -- {{ARGS}}

run-release *ARGS:
    @cargo run --release --quiet -- {{ARGS}}

clean:
    cargo clean

update:
    cargo update

outdated:
    @cargo outdated --depth 1 2>/dev/null | awk 'NR<=2 || (!/Removed/ && !/---/)'

release-check: ci build-release
    @echo "✓ All checks passed!"
