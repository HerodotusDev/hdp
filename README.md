![](.github/readme.png)

# HDP

[![CI](https://github.com/HerodotusDev/hdp/actions/workflows/ci.yml/badge.svg)](https://github.com/HerodotusDev/hdp/actions/workflows/ci.yml)

> WARNING: This codebase is experimental and has not been audited.

_Enhance zk-offchain compute for verifiable onchain data using zkVMs_

HDP stands for Herodotus Data Processor, which can process a range of block data and retrieve valid values from proving ZK-STARK proof. For more explanation, check out the [documentation](https://docs.herodotus.dev/herodotus-docs/developers/herodotus-data-processor-hdp).

CLI is mainly used for processing human-readable requests to Cairo-Program acceptable format files. In the broader view, this is called the `pre-processing step`. Additionally provides some useful features that are supported for development.

## Supported Features

### Develop Purpose

- [x] Encode provided datalake and task
- [x] Decode multiple datalakes and tasks in batch (`bytes[]`) abi-encoded
- [x] Decode one datalake and task
- [ ] FactHash computation of HDP Cairo Program

### HDP Core (Pre-Processing)

- [x] Decode provided tasks and datalakes
- [x] Compile datalake 1: Fetch relevant header data and proofs from Herodotus Indexer
- [x] Compile datalake 2: Fetch relevant account and storage data and proofs from RPC provider
- [x] Compute aggregated function (ex. `SUM`, `AVG`) over compiled datalake result
- [x] Return general ( human-readable ) and Cairo formatted ( all chunked with felt size ) file
- [x] Support multi tasks process, with [Standard Merkle Tree](https://github.com/rkdud007/alloy-merkle-tree/blob/main/src/standard_binary_tree.rs) aggregation
- [ ] Support more datalake types: DynamicLayoutDatalake, TransactionsBySenderDatalake ... etc
- [ ] Multichain support
- [ ] Launch server and implement API to generate serialize task bytes
- [ ] Optimize HDP-Provider with cache and persistent DB
- [ ] Support More ZKVM as a backend option ([CAIRO](https://eprint.iacr.org/2021/1063), [RISC0](https://github.com/risc0/risc0), [SP1](https://github.com/succinctlabs/sp1)... etc)

## [Example](https://github.com/HerodotusDev/hdp/tree/main/example)

_Note: Sum and Avg functions support only for numbers as expected input type_

All examples are tested with `script/integration.sh`. The currently compiled HDP Cairo program supports all the features below. If you want to run the script locally, check out the [readme](https://github.com/HerodotusDev/hdp/tree/main/example).

|                                 | SUM | AVG |
| ------------------------------- | --- | --- |
| account.nonce                   | ✅  | ✅  |
| account.balance                 | ✅  | ✅  |
| account.storage_root            | -   | -   |
| account.code_hash               | -   | -   |
| storage.key ( value is num )    | ✅  | ✅  |
| storage.key (value is hash )    | -   | -   |
| header.parent_hash              | -   | -   |
| header.ommers_hash              | -   | -   |
| header.beneficiary              | -   | -   |
| header.state_root               | -   | -   |
| header.transactions_root        | -   | -   |
| header.receipts_root            | -   | -   |
| header.logs_bloom               | -   | -   |
| header.difficulty               | ✅  | ✅  |
| header.number                   | ✅  | ✅  |
| header.gas_limit                | ✅  | ✅  |
| header.gas_used                 | ✅  | ✅  |
| header.timestamp                | ✅  | ✅  |
| header.extra_data               | -   | -   |
| header.mix_hash                 | -   | -   |
| header.nonce                    | ✅  | ✅  |
| header.base_fee_per_gas         | ✅  | ✅  |
| header.withdrawals_root         | -   | -   |
| header.blob_gas_used            | ✅  | ✅  |
| header.excess_blob_gas          | ✅  | ✅  |
| header.parent_beacon_block_root | -   | -   |

## Install HDP

### Install with cargo

```bash
# Install with cargo
❯ cargo install --git https://github.com/HerodotusDev/hdp --locked --force
```

### Build from source

```bash
# clone repo
❯ git clone https://github.com/HerodotusDev/hdp.git

# install hdp
❯ cargo install --path cli -f
```

## Quick Start

For new users not familiar with how to send HDP requests, we provide the `hdp start` command to enter the interactive CLI app.

```bash
# Start the HDP
❯ hdp start
Welcome to Herodotus Data Processor interactive CLI! 🛰️

                _   _   ____    ____
                | | | | |  _ \  |  _ \
                | |_| | | | | | | |_) |
                |  _  | | |_| | |  __/
                |_| |_| |____/  |_|

? Step 1. What's your datalake type?
```

## HDP run

```console
❯ hdp run --help
Run the evaluator

Usage: hdp run [OPTIONS] [TASKS] [DATALAKES] [RPC_URL] [CHAIN_ID]

Arguments:
  [TASKS]      Batched tasks bytes
  [DATALAKES]  Batched datalakes bytes
  [RPC_URL]    The RPC URL to fetch the data
  [CHAIN_ID]   The chain id to fetch the data

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
hdp run 0x000000000000000000000000000000000000000000000000000000000000002000000000000000000000000000000000000000000000000000000000000000010000000000000000000000000000000000000000000000000000000000000020000000000000000000000000000000000000000000000000000000000000006073756d000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000400000000000000000000000000000000000000000000000000000000000000000  0x00000000000000000000000000000000000000000000000000000000000000200000000000000000000000000000000000000000000000000000000000000001000000000000000000000000000000000000000000000000000000000000002000000000000000000000000000000000000000000000000000000000000000e0000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000004b902400000000000000000000000000000000000000000000000000000000004b9027000000000000000000000000000000000000000000000000000000000000000100000000000000000000000000000000000000000000000000000000000000a00000000000000000000000000000000000000000000000000000000000000016027f2c6f930306d3aa736b3a6c6a98f512f74036d40000000000000000000000 ${Input your RPC Provider -- this example is Etherum Sepolia} ${Input Chain ID that you are target on}

```

## Development

```sh
# CI check
cargo clippy --all --all-targets -- -D warnings && cargo fmt -- --check && cargo test --all --all-targets -- --nocapture
```

## How to define own module

_Make sure to create PR if you read this section_

Most likely you would have to define a new module at this [aggregation_fn/mod.rs](https://github.com/HerodotusDev/hdp/tree/main/crates/core/src/aggregate_fn) file. Define the new module as Enum and fill out match arms for the added module.

Depends on the expected input type, if it's an integer use [`U256`](https://docs.rs/alloy-primitives/latest/alloy_primitives/index.html#reexport.U256) and if it's a string then just use string.

Just like [`COUNT`](https://github.com/HerodotusDev/hdp/blob/1d19daceb84d4e8f7ef46774ecc94aebb42b0007/crates/core/src/aggregate_fn/integer.rs#L114) function, if you need additional context to utilize in operation, you could pass it and utilize it.

Finally, add proper tests to see if it works as expected. Especially for integer type, make sure it works well with bytes32 length value.

## More Usage

```console
❯ hdp --help
Interact Herodotus Data Processor via CLI

Usage: hdp <COMMAND>

Commands:
  encode      Encode the task and datalake in batched format test purposes
  decode      Decode batch tasks and datalakes
  decode-one  Decode one task and one datalake (not batched format)
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
Encode the task and datalake in batched format test purposes

Usage: hdp encode [OPTIONS] <AGGREGATE_FN_ID> [AGGREGATE_FN_CTX] [RPC_URL] [CHAIN_ID] <COMMAND>

Commands:
  block-sampled, -b  Encode the block sampled data lake for test purposes
  help               Print this message or the help of the given subcommand(s)

Arguments:
  <AGGREGATE_FN_ID>   The aggregate function id e.g. "sum", "min", "avg"
  [AGGREGATE_FN_CTX]  The aggregate function context. It depends on the aggregate function
  [RPC_URL]           The RPC URL to fetch the data
  [CHAIN_ID]          The chain id to fetch the data

Options:
  -a, --allow-run                  Decide if want to run evaluator as follow step or not (default: false)
  -o, --output-file <OUTPUT_FILE>  Path to the file to save the output result
  -c, --cairo-input <CAIRO_INPUT>  Path to the file to save the input.json in cairo format
  -h, --help                       Print help
```

### Decode

```console
❯ hdp help decode
Decode batch tasks and datalakes

Note: Batch tasks and datalakes should be encoded in bytes[] format

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

## License

`hdp` is licensed under the [GNU General Public License v3.0](./LICENSE).

---

Herodotus Dev Ltd - 2024
