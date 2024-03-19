## Request

30 Blocks, Header Number, Avg

### Server

```json
{
  "chainId": 11155111,
  "datalakeType": "block_sampled",
  "datalake": {
    "blockRangeStart": 4952200,
    "blockRangeEnd": 4952229,
    "sampledProperty": "header.number"
  },
  "aggregateFnId": "avg"
}
```

### CLI

```bash
hdp encode -a -o example/header/avg_number/output.json -c example/header/avg_number/input.json "avg" -b 4952200 4952229 "header.number" 1
```
