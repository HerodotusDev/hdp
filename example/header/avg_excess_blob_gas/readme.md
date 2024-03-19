## Request

30 Blocks, Header excess_blob_gas, avg

### Server

```json
{
  "chainId": 11155111,
  "datalakeType": "block_sampled",
  "datalake": {
    "blockRangeStart": 4952200,
    "blockRangeEnd": 4952229,
    "sampledProperty": "header.excess_blob_gas"
  },
  "aggregateFnId": "avg"
}
```

### CLI

```bash
hdp encode -a -o example/header/avg_excess_blob_gas/output.json -c example/header/avg_excess_blob_gas/input.json "avg" -b 5515000 5515029 "header.excess_blob_gas" 1
```
