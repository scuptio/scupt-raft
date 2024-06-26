# scupt-raft

TLA+ verified Raft consensus

![build](https://github.com/scuptio/scupt-raft/actions/workflows/build.yaml/badge.svg)


# Specification-Driven Development of Raft Consensus

## Prerequisite

1. Download and install TLA+ [toolbox](https://github.com/tlaplus/tlaplus/releases)

2. Download and install [sedeve-kit](https://github.com/scuptio/sedeve-kit.git)

   ```
   git clone https://github.com/scuptio/sedeve-kit.git
   cd sedeve-kit
   cargo install --path .
   ```
3. [Configure the TLA+ toolbox](https://github.com/scuptio/sedeve-kit/blob/main/doc/configuring_toolbox.md)
   
## Run model check
   
   Model setup, filling constant
   
   Constant example [MC](data/MC_1n.tla)
   
   After checking finish, output state database.

## Generate trace of the model
   Use the state database the toolbox output, generate trace file.

   Example,
   ```
   cd scupt-raft
   sedeve_trace_gen  \
      --state-db-path /tmp/raft_action.db  \
      --out-trace-db-path /tmp/raft_trace.db \
      --map-const-path  ./data/raft_map_const.json
   ```
   
   After finish executing, the output trace would be store in database file '/tmp/raft_trace.db'

## Validate by running deterministic testing
   
   Write test code
   
   Example [test_raft.rs](src/test_raft_1n.rs)

## Test Coverage

[Test Coverage in Docker](doc/coverage.md)

# Build docker environment

[Dockerfile](docker/Dockerfile)

```shell
   cd docker
   docker build .
```



