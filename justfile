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

# In-circuit benchmarks. Requires the RISC Zero toolchain (rzup) on PATH.
# The batch size N defaults to 1.
bench-zkvm-risc0 n="1":
    cd crates/bench-zkvm/risc0 && cargo run --release -p pqb-risc0-host -- {{n}}

# RISC Zero, one scheme, execute (cycles) or prove (wall-clock). N defaults 1.
bench-zkvm-risc0 scheme="ed25519" mode="execute" n="1":
    cd crates/bench-zkvm/risc0 && cargo run --release -p pqb-risc0-host -- {{scheme}} {{mode}} {{n}}

# SP1, one scheme, execute only (cycle counts). Needs protoc on PATH.
# Proving is deferred to the Linux x86 fairness run.
bench-zkvm-sp1 scheme="ed25519" n="1":
    cd crates/bench-zkvm/sp1 && PROTOC="${PROTOC:-$(which protoc)}" cargo run --release -p pqb-sp1-host -- {{scheme}} execute {{n}}

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
