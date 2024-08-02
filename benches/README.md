# `EvmProvider` Benchmark

## Hardware Specifications

- **Processor**: Apple M2
- **Memory**: 32 GB
- **Operating System**: macOS

## RPC Specifications

- used Alchemy non-paid plan rpc url

## Benchmark Results

| Benchmark                            | Time (ms) | Iterations | Notes          |
| ------------------------------------ | --------- | ---------- | -------------- |
| get_10_header_proofs                 | 200.52 ms | 10         | Block Range 10 |
| get_10_account_proofs                | 243.05 ms | 10         | Block Range 10 |
| get_10_storage_proofs                | 245.14 ms | 10         | Block Range 10 |
| get_tx_with_proof_from_block         | 231.62 ms | 10         | --             |
| get_tx_receipt_with_proof_from_block | 1590.2 ms | 10         | --             |
