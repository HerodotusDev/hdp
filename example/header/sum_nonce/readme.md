## Request

30 Blocks, Header Nonce, Sum

### Server

```json
{
  "chainId": 11155111,
  "datalakeType": "block_sampled",
  "datalake": {
    "blockRangeStart": 4952200,
    "blockRangeEnd": 4952229,
    "sampledProperty": "header.nonce"
  },
  "aggregateFnId": "sum"
}
```

### CLI

```bash
hdp encode -a -o example/header/sum_nonce/output.json -c example/header/sum_nonce/input.json "sum" -b 4952200 4952229 "header.nonce" 1
```
