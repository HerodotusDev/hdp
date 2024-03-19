## Request

30 Blocks, Header gas_used, Sum

### Server

```json
{
  "chainId": 11155111,
  "datalakeType": "block_sampled",
  "datalake": {
    "blockRangeStart": 4952200,
    "blockRangeEnd": 4952229,
    "sampledProperty": "header.gas_used"
  },
  "aggregateFnId": "sum"
}
```

### CLI

```bash
hdp encode -a -o example/header/sum_gas_used/output.json -c example/header/sum_gas_used/input.json "sum" -b 4952200 4952229 "header.gas_used" 1
```
