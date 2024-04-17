## Request

Txs in block, SUM nonce

### Server

```json
{
  "chainId": 11155111,
  "datalakeType": "transactions",
  "datalake": {
    "targetBlock": 5608949,
    "sampledProperty": "tx.nonce",
    "increment": 20
  },
  "aggregateFnId": "sum"
}
```

### CLI

```bash
hdp encode -a -o example/transaction/sum_nonce/output.json -c example/transaction/sum_nonce/input.json "sum" -t 5608949 "tx.nonce" 20
```
