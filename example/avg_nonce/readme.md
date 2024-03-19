## Request

30 Blocks, Account Nonce, AVG

### Server

```json
{
  "chainId": 11155111,
  "datalakeType": "block_sampled",
  "datalake": {
    "blockRangeStart": 4952200,
    "blockRangeEnd": 4952229,
    "sampledProperty": "account.0x7f2c6f930306d3aa736b3a6c6a98f512f74036d4.nonce"
  },
  "aggregateFnId": "avg"
}
```

### CLI

```bash
hdp encode -a -o example/avg_nonce/output.json -c example/avg_nonce/input.json "avg" -b 4952200 4952229 "account.0x7f2c6f930306d3aa736b3a6c6a98f512f74036d4.nonce" 1
```
