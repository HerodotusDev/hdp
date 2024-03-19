## Request

30 Blocks, Header blob_gas_used, Sum

### Server

```json
{
  "chainId": 11155111,
  "datalakeType": "block_sampled",
  "datalake": {
    "blockRangeStart": 4952200,
    "blockRangeEnd": 4952229,
    "sampledProperty": "header.blob_gas_used"
  },
  "aggregateFnId": "sum"
}
```

### CLI

```bash
hdp encode -a -o example/header/sum_blob_gas_used/output.json -c example/header/sum_blob_gas_used/input.json "sum" -b 5515000 5515029 "header.blob_gas_used" 1
```
