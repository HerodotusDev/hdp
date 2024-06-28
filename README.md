![](.github/readme.png)

# Herodotus Data Processor (HDP)

[![CI](https://github.com/HerodotusDev/hdp/actions/workflows/ci.yml/badge.svg)](https://github.com/HerodotusDev/hdp/actions/workflows/ci.yml)

> **Warning:** This codebase is experimental and not audited. Use at your own risk.

HDP enhances off-chain compute capabilities with zkVMs for verifiable on-chain data integration. For more, visit our [documentation](https://docs.herodotus.dev/herodotus-docs/developers/herodotus-data-processor-hdp).

## Introduction

The Data Processor CLI serves as an essential tool for developers working with Cairo programs and zkVM environments. Its primary function is to translate human-readable requests into a format compatible with Cairo programs, enabling commands to be executed over the Cairo VM and generating executable outputs. This transformation is a crucial preprocessing step that prepares data for off-chain computations in zkVM environments.

## Features

- **Development Tools**: Encode and decode data lakes and computational tasks.
- **Core Processing**: Compile data from various sources and compute aggregate functions.
- **Extensibility**: Support for multiple blockchain integrations and various ZKVM backends is planned.
- **Ease of Use**: Provides a CLI for easy interaction with the system.

## Install HDP

### Install with cargo

```bash
# Install with cargo
❯ cargo install --git https://github.com/HerodotusDev/hdp --tag v0.2.6 --locked --force
```

### Build from source

```bash
# clone repo
❯ git clone https://github.com/HerodotusDev/hdp.git

# install hdp
❯ cargo install --path cli -f
```

## Getting Started

To launch the interactive CLI:

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

## Usage Examples

The following examples demonstrate how to use the HDP CLI to encode various blockchain data elements into a format suitable for processing. Each command highlights a different function of the HDP system, from averaging values to counting values based on specific criteria.

Header value with `AVG`:

```
hdp encode -a -c {input.file} "avg" -b 4952100 4952110 "header.base_fee_per_gas" 1
```

Account value with `SUM`:

```
hdp encode -a -c {input.file} "sum" -b 4952100 4952110 "account.0x7f2c6f930306d3aa736b3a6c6a98f512f74036d4.nonce" 2
```

Storage value with `AVG`:

```
hdp encode -a -c {input.file} "avg" -b 5382810 5382820 "storage.0x75CeC1db9dCeb703200EAa6595f66885C962B920.0x0000000000000000000000000000000000000000000000000000000000000002" 1
```

Account value with `COUNT`:

```
hdp encode -a -c {input.file} "count" "gt.1000" -b 4952100 4952110 "account.0x7f2c6f930306d3aa736b3a6c6a98f512f74036d4.nonce" 2
```

After encoding, you can directly run processing tasks using environmental configurations for RPC and Chain ID, as shown below:

```bash
# pro tip: run herodotus data processing with `.env`
hdp run

# run herodotus data processing
hdp run ${Encoded Task} ${Encoded Datalake} ${Input your RPC Provider -- this example is Etherum Sepolia} ${Input Chain ID that you are target on}
```

For a more comprehensive guide on commands available within HDP CLI:

```console
❯ hdp --help
Interact Herodotus Data Processor via CLI

Usage: hdp <COMMAND>

Commands:
  start       New to the HDP CLI? Start here!
  encode      Encode the task and datalake in batched format test purposes
  decode      Decode batch tasks and datalakes
  decode-one  Decode one task and one datalake (not batched format)
  run         Run the evaluator
  help        Print this message or the help of the given subcommand(s)

Options:
  -h, --help     Print help
  -V, --version  Print version
```

## Integration Testing

Integration testing in HDP ensures that the functionality of aggregate functions such as `SUM`, `AVG`, `MIN`, `MAX`, and `COUNT` operates correctly across various numeric fields within the blockchain data structure. These functions are designed specifically for numeric data types, ensuring accurate and reliable computations.

### Integration Test

The core soundness of HDP relies on generating the correct input file and running the Cairo program. To ensure this, a full integration test flow is necessary to link the pre-processor and processor versions. For continuous integration tests, please refer to the [hdp-test](https://github.com/HerodotusDev/hdp-test) repository as it contains all the cases of supported features in table below.

### Supported Aggregate Functions

- **SUM, AVG, MIN, MAX, COUNT**: These functions are supported only for fields with numeric values.
- **SLR**: Simple linear regression written in Cairo 1. The input array should contain more than 2 elements.

### Function Support Matrix

Here is the support matrix indicating which blockchain elements are tested for each aggregate function. The matrix highlights fields where these functions are applicable.

| Field Description                | SUM | AVG | MIN | MAX | COUNT | SLR |
| -------------------------------- | --- | --- | --- | --- | ----- | --- |
| `account.nonce`                  | ✅  | ✅  | ✅  | ✅  | ✅    | ✅  |
| `account.balance`                | ✅  | ✅  | ✅  | ✅  | ✅    | ✅  |
| `account.storage_root`           | -   | -   | -   | -   | -     | -   |
| `account.code_hash`              | -   | -   | -   | -   | -     | -   |
| `storage.key` (numeric value)    | ✅  | ✅  | ✅  | ✅  | ✅    | ✅  |
| `storage.key` (hash value)       | -   | -   | -   | -   | -     | -   |
| `header.difficulty`              | ✅  | ✅  | ✅  | ✅  | ✅    | ✅  |
| `header.gas_limit`               | ✅  | ✅  | ✅  | ✅  | ✅    | ✅  |
| `header.gas_used`                | ✅  | ✅  | ✅  | ✅  | ✅    | ✅  |
| `header.timestamp`               | ✅  | ✅  | ✅  | ✅  | ✅    | ✅  |
| `header.base_fee_per_gas`        | ✅  | ✅  | ✅  | ✅  | ✅    | ✅  |
| `header.blob_gas_used`           | ✅  | ✅  | ✅  | ✅  | ✅    | ✅  |
| `header.excess_blob_gas`         | ✅  | ✅  | ✅  | ✅  | ✅    | ✅  |
| `header.nonce`                   | ✅  | ✅  | ✅  | ✅  | ✅    | ✅  |
| Other `header` elements          | -   | -   | -   | -   | -     | -   |
| `tx.nonce`                       | ✅  | ✅  | ✅  | ✅  | ✅    | ✅  |
| `tx.gas_price`                   | ✅  | ✅  | ✅  | ✅  | ✅    | ✅  |
| `tx.gas_limit`                   | ✅  | ✅  | ✅  | ✅  | ✅    | ✅  |
| `tx.value`                       | ✅  | ✅  | ✅  | ✅  | ✅    | ✅  |
| `tx.v`                           | ✅  | ✅  | ✅  | ✅  | ✅    | ✅  |
| `tx.r`                           | ✅  | ✅  | ✅  | ✅  | ✅    | ✅  |
| `tx.s`                           | ✅  | ✅  | ✅  | ✅  | ✅    | ✅  |
| `tx.chain_id`                    | ✅  | ✅  | ✅  | ✅  | ✅    | ✅  |
| `tx.max_fee_per_gas`             | ✅  | ✅  | ✅  | ✅  | ✅    | ✅  |
| `tx.max_priority_fee_per_gas`    | ✅  | ✅  | ✅  | ✅  | ✅    | ✅  |
| `tx.max_fee_per_blob_gas`        | ✅  | ✅  | ✅  | ✅  | ✅    | ✅  |
| Other `tx` elements              | -   | -   | -   | -   | -     | -   |
| `tx_receipt.success`             | ✅  | ✅  | ✅  | ✅  | ✅    | ✅  |
| `tx_receipt.cumulative_gas_used` | ✅  | ✅  | ✅  | ✅  | ✅    | ✅  |
| Other `tx_receipt` elements      | -   | -   | -   | -   | -     | -   |

_Note: Fields marked with "-" are not applicable for the specified aggregate functions because they do not contain numeric data or the data type is not suitable for these calculations._

### Additional Notes

- Please ensure that the data fields you are applying these functions contain numeric values to avoid computational errors.
- For details on how these tests are performed or to contribute to the further development of tests, please refer to the [Integration Test Guide](https://github.com/HerodotusDev/hdp-test).

## Development

```sh
# CI check
cargo make run-ci-flow
```

## Defining Your Own Module

For developers interested in extending the functionality of HDP by adding new modules, follow the steps outlined below. This guide assumes a basic understanding of Rust and its module system.

### Getting Started

1. **Module Location**: Start by creating a new module within the `aggregate_fn` directory. You can find this at [aggregation_fn/mod.rs](./crates/primitives/src/aggregate_fn).

2. **Define Enum**: Define your new function as an enum in the [file](./crates/primitives/src/aggregate_fn). Make sure to add match arms for the new enum variants in the implementation.

3. **Handle Data Types**: Depending on the expected input type for your function:
   - **Integer Inputs**: Use [`U256`](https://docs.rs/alloy-primitives/latest/alloy_primitives/index.html#reexport.U256) for handling large integers compatible with Ethereum's numeric constraints.
   - **String Inputs**: Use Rust's standard `String` type for text data.

### Context Required Operation

For a practical example of how to implement context-sensitive operations, refer to the implementation of the `COUNT` function. This example shows how to pass and utilize additional context for operations, which can be particularly useful for conditional processing or complex calculations.

During `SLR` computation, we also need a context to use as the target index for computation. Since `SLR` is not supported during the preprocessing step, we simply pass the encoded task that contains the function context, and the Cairo program will handle this computation based on the provided index.

### Testing Your Module

After implementing your new function, it's crucial to verify its functionality:

- **Create Unit Tests**: Add tests in the corresponding test file in the `tests` directory. Ensure your tests cover all new logic to maintain stability and reliability.
- **Test for Integer Types**: Pay special attention to functions that handle integer types, ensuring they correctly process and output values fitting within a `bytes32` length, reflecting Ethereum's data type constraints.

### Local Run

To run HDP in a stable environment locally, you need to have `cairo-run` installed with the necessary tools in the correct path and locate the compiled Cairo program. If these steps sound tricky to you, just use the Docker image.

To mount in a container environment, you need to create empty `input.json`, `output.json`, and `cairo.pie` files in the root directory of the host machine before running it.

```sh
docker-compose build

docker-compose up
```

For those looking for an already built Docker image, you can pull it from [here](https://hub.docker.com/r/dataprocessor/hdp-runner).

## License

`hdp` is licensed under the [GNU General Public License v3.0](./LICENSE).

---

Herodotus Dev Ltd - 2024
