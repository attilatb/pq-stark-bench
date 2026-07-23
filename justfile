# PQ-STARK-BENCH task runner.
# Install just: https://github.com/casey/just

default:
    @just --list

# Run the native benchmark suite and write a results file.
bench-native:
    cargo run --release -p bench-native

# Fast pass for development. Fewer iterations, not for publication.
bench-native-quick:
    cargo run --release -p bench-native -- --quick

# In-circuit benchmarks. Phase 2, not implemented yet.
bench-zkvm:
    @echo "Phase 2 is not implemented yet. See docs/METHODOLOGY.md."
    @exit 1

test:
    cargo test --workspace

lint:
    cargo fmt --all --check
    cargo clippy --workspace --all-targets -- -D warnings

# Fail the build if banned phrasing or em dashes creep into published copy.
# The positioning rules are load-bearing, so they are enforced mechanically.
check-copy:
    bash scripts/check-copy.sh

site-dev:
    npm --prefix site install
    npm --prefix site run dev

site-build:
    npm --prefix site install
    npm --prefix site run build

# Everything CI runs.
ci: lint test check-copy site-build
