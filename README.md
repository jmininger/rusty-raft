# rusty-raft
<img src="ferris-raft.jpeg" alt="Ferris the Rust Crab on a Raft" width="300">


# Overview
A very WIP/toy implemenation of the [Raft Consensus Protocol](https://raft.github.io/) in Rust. See the [Raft
paper](https://raft.github.io/raft.pdf) for more information.

# Running
This project uses the [just command runner](https://github.com/casey/just) to make running various
tasks easier.

<br>
In the raft protocol the set of nodes is fixed and their identities are meant to be known ahead of
time. I use the `orchestrator` program as a way for nodes in the cluster to discover each other.
<br>
<br>

Make sure env vars are set properly in `.env`. Then, in tmux window 1:

```bash
just orchestrator
```
In three separate tmux windows run the following command, making sure to replace N with 1-3

```bash
just run-node $N
```

# Architecture
## Identity protocol
Upon creating a new socket, both nodes expect the following behavior before receiving/sending
protocol messages:
- Over the tcp socket send two lines:
    - `COMMON_NAME`
    - `DIAL_ADDRESS`
- Then read the equivalent two lines from the other side

### NetworkManager and ConnectionActor
- In a prod system I probably would have used a network protocol with a req/resp workflow baked in
  (http, grpc). For `rusty-raft` I thought it would be fun to implement my own layer over raw tcp
  sockets...

- TODO
