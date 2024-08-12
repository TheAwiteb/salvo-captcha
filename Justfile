# This justfile is for the contrbutors of this project, not for the end user.
#
# Requirements for this justfile:
# - Linux distribution, the real programer does not program on garbage OS like Windows or MacOS
# - just (Of course) <https://github.com/casey/just>
# - cargo (For the build and tests) <https://doc.rust-lang.org/cargo/getting-started/installation.html>

set shell := ["/usr/bin/bash", "-c"]

JUST_EXECUTABLE := "just -u -f " + justfile()
header := "Available tasks:\n"
# Get the MSRV from the Cargo.toml
msrv := `cat Cargo.toml | grep "rust-version" | sed 's/.*"\(.*\)".*/\1/'`


_default:
    @{{JUST_EXECUTABLE}} --list-heading "{{header}}" --list

# Run the CI (Local use only)
@ci:
    cargo fmt --all --check
    cargo build -F 'simple_generator' --example simple_login
    cargo clippy --workspace --all-targets --examples --tests --all-features -- -D warnings
    cargo nextest run --workspace --all-targets --all-features
    @{{JUST_EXECUTABLE}} msrv

# Install workspace tools
@install:
    cargo install cargo-nextest

# Check that the current MSRV is correct
msrv:
    @echo "Current MSRV: {{msrv}}"
    @echo "Checking that the current MSRV is correct..."
    @cargo +{{msrv}} check --workspace --all-targets --all-features
    @echo "The current MSRV is correct"
