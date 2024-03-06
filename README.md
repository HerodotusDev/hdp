![](.github/readme.png)

# HDP

[![CI](https://github.com/HerodotusDev/hdp/actions/workflows/ci.yml/badge.svg)](https://github.com/HerodotusDev/hdp/actions/workflows/ci.yml)

HDP stands for Herodotus Data Processor, which is able to process a range of block data and retrieve valid values from proving ZK-STARK proof. CLI is mainly used for processing human-readable requests to Cairo-Program acceptable format files. Additionally, some useful features are supported for development.

## Supported Features

### Develop Purpose

- [x] Encode provided datalake and task
- [x] Decode multiple datalakes and tasks in batch (`bytes[]`) abi-encoded
- [x] Decode one datalake and task

### HDP Core (Evaluation)

- [x] Decode provided tasks and datalakes
- [x] Compile datalake 1: Fetch relevant header data and proofs from Herodotus Indexer
- [x] Compile datalake 2: Fetch relevant account and storage data and proofs from RPC provider
- [x] Compute aggregated function (ex. `SUM`, `AVG`) over compiled datalake result
- [x] Return general ( human-readable ) and Cairo formatted ( all chunked with felt size ) file
- [x] Support multi tasks process, with [Standard Merkle Tree](https://github.com/rkdud007/alloy-merkle-tree/blob/main/src/standard_binary_tree.rs) aggregation
- [ ] Support more datalake types: DynamicLayoutDatalake, TransactionsBySenderDatalake ... etc
- [ ] Multichain support
- [ ] Support More Provers as a backend option ([SHARP](https://starkware.co/resource/joining-forces-sharp/), [RISC0](https://github.com/risc0/risc0), [SP1](https://github.com/succinctlabs/sp1)... etc)

## Install HDP

### Install with cargo

```bash
# Install with cargo
❯ cargo install --git https://github.com/HerodotusDev/hdp --locked --force

# Run the HDP
❯ hdp run --help
```

### Build from source

```bash
# clone repo
❯ git clone https://github.com/HerodotusDev/hdp.git

# install hdp
❯ cargo install --path cli

# Run the HDP
❯ hdp run --help
```

## HDP run

```console
❯ hdp run --help

Run the evaluator

Usage: hdp run [OPTIONS] [TASKS] [DATALAKES] [RPC_URL]

Arguments:
  [TASKS]
  [DATALAKES]
  [RPC_URL]

Options:
  -o, --output-file <OUTPUT_FILE>  Path to the file to save the output result
  -c, --cairo-input <CAIRO_INPUT>  Path to the file to save the input.json in cairo format
  -h, --help                       Print help
```

Support passing argument as env variable or as arguments.

```bash

# pro tip: run herodotus data processing with `.env`
hdp run

# run herodotus data processing
hdp run 0x000000000000000000000000000000000000000000000000000000000000002000000000000000000000000000000000000000000000000000000000000000010000000000000000000000000000000000000000000000000000000000000020000000000000000000000000000000000000000000000000000000000000006073756d000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000400000000000000000000000000000000000000000000000000000000000000000  0x00000000000000000000000000000000000000000000000000000000000000200000000000000000000000000000000000000000000000000000000000000001000000000000000000000000000000000000000000000000000000000000002000000000000000000000000000000000000000000000000000000000000000e0000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000004b902400000000000000000000000000000000000000000000000000000000004b9027000000000000000000000000000000000000000000000000000000000000000100000000000000000000000000000000000000000000000000000000000000a00000000000000000000000000000000000000000000000000000000000000016027f2c6f930306d3aa736b3a6c6a98f512f74036d40000000000000000000000 ${Input your RPC Provider -- this example is Etherum Sepolia}

```

## Development

```sh
# CI check
cargo clippy --all --all-targets -- -D warnings && cargo fmt -- --check && cargo test --all --all-targets -- --nocapture
```

## More Usage

```console
❯ hdp --help
Simple Herodotus Data Processor CLI to handle tasks and datalakes

Usage: hdp <COMMAND>

Commands:
  encode      Encode the task and data lake in batched format test purposes
  decode      Decode batch tasks and data lakes
  decode-one  Decode one task and one data lake (not batched format)
  run         Run the evaluator
  help        Print this message or the help of the given subcommand(s)

Options:
  -h, --help     Print help
  -V, --version  Print version
```

Generate encoded tasks and datalakes for testing purposes. The format is the same as what smart contract emits (considered as batched tasks and datalakes).

### Encode

some examples:

Header value with `AVG`

```
hdp encode "avg" -b 4952100 4952110 "header.base_fee_per_gas" 1
```

Account value with `SUM`

```
hdp encode "sum" -b 4952100 4952110 "account.0x7f2c6f930306d3aa736b3a6c6a98f512f74036d4.nonce" 2
```

Storage value with `AVG`

```
hdp encode "avg" -b 5382810 5382820 "storage.0x75CeC1db9dCeb703200EAa6595f66885C962B920.0x0000000000000000000000000000000000000000000000000000000000000002" 1
```

Check out the encode command for how to generate the encoded value of the targeted task and its corresponding datalake:

```console
❯ hdp help encode
Encode the task and data lake in batched format test purposes

Usage: hdp encode <AGGREGATE_FN_ID> [AGGREGATE_FN_CTX] <COMMAND>

Commands:
  block-sampled, -b  Encode the block sampled data lake for test purposes
  help               Print this message or the help of the given subcommand(s)

Arguments:
  <AGGREGATE_FN_ID>   The aggregate function id e.g. "sum", "min", "avg"
  [AGGREGATE_FN_CTX]  The aggregate function context. It depends on the aggregate function

Options:
  -h, --help  Print help
```

### Decode

```console
❯ hdp help decode
Decode batch tasks and data lakes

Note: Batch tasks and data lakes should be encoded in bytes[] format

Usage: hdp decode <TASKS> <DATALAKES>

Arguments:
  <TASKS>
          Batched tasks bytes

  <DATALAKES>
          Batched datalakes bytes

Options:
  -h, --help
          Print help (see a summary with '-h')
```

### Decode non-batched format

```console
❯ hdp help decode-one
Decode one task and one data lake (not batched format)

Usage: hdp decode-one <TASK> <DATALAKE>

Arguments:
  <TASK>
  <DATALAKE>

Options:
  -h, --help  Print help
```
