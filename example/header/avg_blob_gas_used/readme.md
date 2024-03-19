## Request

30 Blocks, Header blob_gas_used, avg

### Server

```json
{
  "chainId": 11155111,
  "datalakeType": "block_sampled",
  "datalake": {
    "blockRangeStart": 5515000,
    "blockRangeEnd": 5515029,
    "sampledProperty": "header.blob_gas_used"
  },
  "aggregateFnId": "avg"
}
```

### CLI

```bash
hdp encode -a -o example/header/avg_blob_gas_used/output.json -c example/header/avg_blob_gas_used/input.json "avg" -b 5515000 5515029 "header.blob_gas_used" 1
```
