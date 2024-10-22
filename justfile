set dotenv-load := true
set dotenv-required := true

default:
    just --list

# Rust + TOML formatter
fmt:
    cargo +nightly fmt
    taplo fmt

# Open docs
doc:
    cargo doc --open

# Run unit tests
test:
  cargo test

# Orchestrator command
orchestrator:
    cargo run --bin orchestrator -- --host $ORCHESTRATOR_ADDR

# This command starts N tmux panes and runs the orchestrator and raft nodes
# start-simulator n:
#     for i in `seq 1 {{n}}`; do \
#         port=$(expr $BASE_PORT + $i); \
#         echo $port; \
#         cmd="just run-node port=$port"; \
#         alacritty -e zsh -c cmd; \
#     done

run-node id:
    LOCAL_ADDR="127.0.0.1:"$(( $BASE_PORT + {{id}} ))"" NODE_ID="{{id}}" cargo run --bin raft
