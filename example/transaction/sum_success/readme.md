## Request

Tx receipts in block, SUM success

### Server

```json
{
  "chainId": 11155111,
  "datalakeType": "transactions",
  "datalake": {
    "targetBlock": 5608949,
    "sampledProperty": "tx_receipt.success",
    "increment": 20
  },
  "aggregateFnId": "sum"
}
```

### CLI

```bash
hdp encode -a -o example/transaction/sum_success/output.json -c example/transaction/sum_success/input.json sum -t 5608949 "tx_receipt.success" 20
```
