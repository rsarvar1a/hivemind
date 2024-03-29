build:
    cargo build --release

check:
    cargo check \
    && cargo clippy \
    && just fmt

dev:
    cargo run

fmt:
    cargo fmt

gdb:
    cargo build \
    && gdb --ex=run --args env -i RUST_BACKTRACE=1 target/debug/hivemind

perf:
    touch perf/graph.svg && mv perf/graph.svg perf/graph.old.svg
    -cargo flamegraph --profile flame -o perf/graph.svg
    just perf-save

perf-save:
    mkdir -p perf && rm perf.data.old && mv perf.data perf/perf.data

run:
    just build \
    && touch .env \
    && target/release/hivemind

test:
    RUST_LOG=debug cargo pretty-test -- --show-output \
    3>&1 &>logs/test.log 1> >(tee >(cat >&3))

test-heavy:
    RUST_LOG=trace cargo test -- --nocapture --test-threads 1 \
    3>&1 &>logs/full.log 1> >(tee >(cat >&3))

test-all:
    just test
    just test-heavy
