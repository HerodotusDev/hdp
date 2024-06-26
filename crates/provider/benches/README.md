# `EvmProvider` Benchmark

## Hardware Specifications

- **Processor**: Apple M2
- **Memory**: 32 GB
- **Operating System**: macOS

## RPC Specifications

- used Alchemy non-paid plan rpc url

## Benchmark Results

| Benchmark                            | Time (ms)       | Iterations | Notes          |
| ------------------------------------ | --------------- | ---------- | -------------- |
| get_10_header_proofs                 | 1667.7 - 1720.4 | 10         | Block Range 10 |
| get_10_account_proofs                | 343.19 - 403.63 | 10         | Block Range 10 |
| get_10_storage_proofs                | 331.28 - 385.67 | 10         | Block Range 10 |
| get_tx_with_proof_from_block         | 458.63 - 552.80 | 10         | --             |
| get_tx_receipt_with_proof_from_block | 2090.4 - 2692.8 | 10         | --             |
