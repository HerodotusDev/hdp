## Request

Txs in block, AVG gas_limit

### Server

```json
{
  "chainId": 11155111,
  "datalakeType": "transactions",
  "datalake": {
    "targetBlock": 5608949,
    "sampledProperty": "tx.gas_limit",
    "increment": 20
  },
  "aggregateFnId": "avg"
}
```

### CLI

```bash
hdp encode -a -o example/transaction/avg_gas_limit/output.json -c example/transaction/avg_gas_limit/input.json "avg" -t 5608949 "tx.gas_limit" 20
```
