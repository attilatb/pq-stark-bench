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
    #!/usr/bin/env bash
    set -euo pipefail
    files=$(git ls-files 'docs/*.md' 'site/src/**' 'README.md' | grep -v 'KICKOFF-v1-superseded' | grep -v '^docs/research/' || true)
    [ -z "$files" ] && exit 0
    fail=0
    if grep -lP '\x{2014}' $files 2>/dev/null; then
        echo "FAIL: em dash found. House style is hyphens only."
        fail=1
    fi
    for phrase in "world's first" "first ever" "quantum-proof" "quantum proof" "unbreakable"; do
        if grep -ril "$phrase" $files 2>/dev/null; then
            echo "FAIL: banned phrase '$phrase'."
            fail=1
        fi
    done
    if grep -rin "first post-quantum signature verification in a STARK" $files 2>/dev/null; then
        echo "FAIL: that claim is false. See docs/LITERATURE.md."
        fail=1
    fi
    exit $fail

site-dev:
    npm --prefix site install
    npm --prefix site run dev

site-build:
    npm --prefix site install
    npm --prefix site run build

# Everything CI runs.
ci: lint test check-copy site-build
