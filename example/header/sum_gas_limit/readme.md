## Request

30 Blocks, Header gas_limit, Sum

### Server

```json
{
  "chainId": 11155111,
  "datalakeType": "block_sampled",
  "datalake": {
    "blockRangeStart": 4952200,
    "blockRangeEnd": 4952229,
    "sampledProperty": "header.gas_limit"
  },
  "aggregateFnId": "sum"
}
```

### CLI

```bash
hdp encode -a -o example/header/sum_gas_limit/output.json -c example/header/sum_gas_limit/input.json "sum" -b 4952200 4952229 "header.gas_limit" 1
```
