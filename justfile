format-check:
    cargo fmt --all -- --check

format:
    cargo fmt --all

lint:
    cargo clippy --workspace --all-targets --all-features -- -D warnings

test:
    cargo test --workspace --all-features

# Rewrite expected output in crates/bibx/resources/mdtest/*.md from actual runs.
bless:
    MDTEST_UPDATE=1 cargo test -p bibx --all-features --test mdtest

build:
    cargo build --workspace --all-features

check:
    cargo check --workspace --all-targets --all-features

# --lib: bibx-core's lib and bibx's bin would otherwise collide in target/doc.
doc:
    RUSTDOCFLAGS="-D warnings" cargo doc --workspace --no-deps --lib --all-features

# Mirrors the CI `test` job, minus the `deps` and `package` jobs.
ci: format-check lint test build doc

# Quick static checks without tests or builds.
check-all: format-check lint check
