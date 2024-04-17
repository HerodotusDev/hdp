## Request

Tx receipts in block, SUM cumulated_gas_used

### Server

```json
{
  "chainId": 11155111,
  "datalakeType": "transactions",
  "datalake": {
    "targetBlock": 5608949,
    "sampledProperty": "tx_receipt.cumulative_gas_used",
    "increment": 20
  },
  "aggregateFnId": "sum"
}
```

### CLI

```bash
hdp encode -a -o example/transaction/sum_cumulated_gas_used/output.json -c example/transaction/sum_cumulated_gas_used/input.json sum -t 5608949 "tx_receipt.cumulative_gas_used" 20
```
