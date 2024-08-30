![](.github/readme.png)

# Herodotus Data Processor (HDP)

[![CI](https://github.com/HerodotusDev/hdp/actions/workflows/ci.yml/badge.svg)](https://github.com/HerodotusDev/hdp/actions/workflows/ci.yml)
[![Crates.io][crates-badge]][crates-url]
[![Documentation][docs-badge]][docs-url]
[![GPLv3 licensed][gpl3-badge]][gpl3-url]

[crates-url]: https://crates.io/crates/hdp
[crates-badge]: https://img.shields.io/crates/v/hdp.svg
[docs-badge]: https://docs.rs/hdp/badge.svg
[docs-url]: https://docs.rs/hdp
[gpl3-badge]: https://img.shields.io/badge/license-GPLv3-blue
[gpl3-url]: LICENSE

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
❯ cargo install --git https://github.com/HerodotusDev/hdp/ --tag v0.5.0 --locked --force hdp-cli
```

### Build from source

```bash
# clone repo
❯ git clone https://github.com/HerodotusDev/hdp.git

# install hdp
❯ cargo install --locked -f --path  cli/
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

First locate `.env` file like the one in [example](./.env.example).

Second, run command like below :

note that this will go though both pre-process -> process step.

```bash
hdp run -r ${Request file path} -p ${Pre-processor output} -c ${PIE file after process} -o ${Output file after process}
```

For a more comprehensive guide on commands available on `hdp run`:

```console
❯ hdp run --help
Run batch of tasks base on request json file

Usage: hdp run [OPTIONS] --request-file <REQUEST_FILE>

Options:
  -r, --request-file <REQUEST_FILE>
          Pass request as json file

      --rpc-url <RPC_URL>
          The RPC URL to fetch the data.

          Can be overwritten by `RPC_URL` environment variable.

      --dry-run-cairo-file <DRY_RUN_CAIRO_FILE>
          dry run contract bootloader program. only used for module task

  -p, --preprocessor-output-file <PREPROCESSOR_OUTPUT_FILE>
          Path to save output file after pre-processing

      --cairo-format
          Set this boolean to true to generate cairo format preprocessor_output_file

      --sound-run-cairo-file <SOUND_RUN_CAIRO_FILE>
          hdp cairo compiled program. main entry point

  -o, --output-file <OUTPUT_FILE>
          Path to save output file after process

          This will trigger processing(=pie generation) step

  -c, --cairo-pie-file <CAIRO_PIE_FILE>
          Path to save pie file

          This will trigger processing(=pie generation) step

  -h, --help
          Print help (see a summary with '-h')
```

## Integration Testing

Integration testing in HDP ensures that the functionality of aggregate functions such as `SUM`, `AVG`, `MIN`, `MAX`, and `COUNT` operates correctly across various numeric fields within the blockchain data structure. These functions are designed specifically for numeric data types, ensuring accurate and reliable computations.

### Integration Test

The core soundness of HDP relies on generating the correct input file and running the Cairo program. To ensure this, a full integration test flow is necessary to link the pre-processor and processor versions. For continuous integration tests, please refer to the [hdp-test](https://github.com/HerodotusDev/hdp-test) repository as it contains all the cases of supported features in table below.

### Supported Aggregate Functions

- **SUM, AVG, MIN, MAX, COUNT**: These functions are supported only for fields with numeric values.

### Context Required Operation

For a practical example of how to implement context-sensitive operations, refer to the implementation of the `COUNT` function. This example shows how to pass and utilize additional context for operations, which can be particularly useful for conditional processing or complex calculations.

### Function Support Matrix

Here is the support matrix indicating which blockchain elements are tested for each aggregate function. The matrix highlights fields where these functions are applicable.

| Field Description                | SUM | AVG | MIN | MAX | COUNT |
| -------------------------------- | --- | --- | --- | --- | ----- |
| `account.nonce`                  | ✅  | ✅  | ✅  | ✅  | ✅    |
| `account.balance`                | ✅  | ✅  | ✅  | ✅  | ✅    |
| `account.storage_root`           | -   | -   | -   | -   | -     |
| `account.code_hash`              | -   | -   | -   | -   | -     |
| `storage.key` (numeric value)    | ✅  | ✅  | ✅  | ✅  | ✅    |
| `storage.key` (hash value)       | -   | -   | -   | -   | -     |
| `header.difficulty`              | ✅  | ✅  | ✅  | ✅  | ✅    |
| `header.gas_limit`               | ✅  | ✅  | ✅  | ✅  | ✅    |
| `header.gas_used`                | ✅  | ✅  | ✅  | ✅  | ✅    |
| `header.timestamp`               | ✅  | ✅  | ✅  | ✅  | ✅    |
| `header.base_fee_per_gas`        | ✅  | ✅  | ✅  | ✅  | ✅    |
| `header.blob_gas_used`           | ✅  | ✅  | ✅  | ✅  | ✅    |
| `header.excess_blob_gas`         | ✅  | ✅  | ✅  | ✅  | ✅    |
| `header.nonce`                   | ✅  | ✅  | ✅  | ✅  | ✅    |
| Other `header` elements          | -   | -   | -   | -   | -     |
| `tx.nonce`                       | ✅  | ✅  | ✅  | ✅  | ✅    |
| `tx.gas_price`                   | ✅  | ✅  | ✅  | ✅  | ✅    |
| `tx.gas_limit`                   | ✅  | ✅  | ✅  | ✅  | ✅    |
| `tx.value`                       | ✅  | ✅  | ✅  | ✅  | ✅    |
| `tx.v`                           | ✅  | ✅  | ✅  | ✅  | ✅    |
| `tx.r`                           | ✅  | ✅  | ✅  | ✅  | ✅    |
| `tx.s`                           | ✅  | ✅  | ✅  | ✅  | ✅    |
| `tx.chain_id`                    | ✅  | ✅  | ✅  | ✅  | ✅    |
| `tx.max_fee_per_gas`             | ✅  | ✅  | ✅  | ✅  | ✅    |
| `tx.max_priority_fee_per_gas`    | ✅  | ✅  | ✅  | ✅  | ✅    |
| `tx.max_fee_per_blob_gas`        | ✅  | ✅  | ✅  | ✅  | ✅    |
| Other `tx` elements              | -   | -   | -   | -   | -     |
| `tx_receipt.success`             | ✅  | ✅  | ✅  | ✅  | ✅    |
| `tx_receipt.cumulative_gas_used` | ✅  | ✅  | ✅  | ✅  | ✅    |
| Other `tx_receipt` elements      | -   | -   | -   | -   | -     |

_Note: Fields marked with "-" are not applicable for the specified aggregate functions because they do not contain numeric data or the data type is not suitable for these calculations._

### Additional Notes

- Please ensure that the data fields you are applying these functions contain numeric values to avoid computational errors.
- For details on how these tests are performed or to contribute to the further development of tests, please refer to the [Integration Test Guide](https://github.com/HerodotusDev/hdp-test).

## Development

```sh
# CI check
cargo make run-ci-flow
```

### Local Run

To run HDP in a stable environment locally, you need to have `cairo-run` installed with the necessary tools in the correct path and locate the compiled Cairo program. If these steps sound tricky to you, just use the Docker image.

To mount in a container environment, you need to create empty `input.json`, `output.json`, and `cairo.pie` files in the root directory of the host machine before running it.

And locate `requeset.json` file on root that contains intended request format.

```sh
docker-compose build

docker-compose up
```

For those looking for an already built Docker image, you can pull it from [here](https://hub.docker.com/r/dataprocessor/hdp-runner).

## License

`hdp` is licensed under the [GNU General Public License v3.0](./LICENSE).

---

Herodotus Dev Ltd - 2024
