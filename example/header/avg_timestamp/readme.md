## Request

30 Blocks, Header Timestamp, Avg

### Server

```json
{
  "chainId": 11155111,
  "datalakeType": "block_sampled",
  "datalake": {
    "blockRangeStart": 4952200,
    "blockRangeEnd": 4952229,
    "sampledProperty": "header.timestamp"
  },
  "aggregateFnId": "avg"
}
```

### CLI

```bash
hdp encode -a -o example/header/avg_timestamp/output.json -c example/header/avg_timestamp/input.json "avg" -b 4952200 4952229 "header.timestamp" 1
```
