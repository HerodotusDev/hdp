## Request

30 Blocks, Header blob_gas_used, Min

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
  "aggregateFnId": "min"
}
```

### CLI

```bash
hdp encode -a -o example/min_max_count/min_header/output.json -c example/min_max_count/min_header/input.json "min" -b 5515000 5515029 "header.blob_gas_used" 1
```
