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

run-node port:
    LOCAL_ADDR="127.0.0.1:{{port}}" cargo run --bin raft
    echo {{port}}
