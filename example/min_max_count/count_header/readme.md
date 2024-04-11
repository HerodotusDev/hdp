## Request

30 Blocks, Header blob_gas_used, Count GreaterThan

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
  "aggregateFnId": "count",
  "aggregateFnCtx": {
    "operatorId": "gt",
    "valueToCompare": 100000
  }
}
```

### CLI

```bash
hdp encode -a -o example/min_max_count/count_header/output.json -c example/min_max_count/count_header/input.json "count" "gt.100000" -b 5515000 5515029 "header.blob_gas_used" 1
```
