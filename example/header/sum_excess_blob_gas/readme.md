## Request

30 Blocks, Header excess_blob_gas, Sum

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
  "aggregateFnId": "sum"
}
```

### CLI

```bash
hdp encode -a -o example/header/sum_excess_blob_gas/output.json -c example/header/sum_excess_blob_gas/input.json "sum" -b 5515000 5515029 "header.excess_blob_gas" 1
```
