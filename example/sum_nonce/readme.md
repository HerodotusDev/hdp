## Request

30 Blocks, Account nonce, Sum

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
  "aggregateFnId": "sum"
}
```

### CLI

```bash
hdp encode -a -o example/sum_nonce/output.json -c example/sum_nonce/input.json "sum" -b 4952200 4952229 "account.0x7f2c6f930306d3aa736b3a6c6a98f512f74036d4.nonce" 1
```
