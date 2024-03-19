## Request

30 Blocks, Header base_fee_per_gas, Sum

### Server

```json
{
  "chainId": 11155111,
  "datalakeType": "block_sampled",
  "datalake": {
    "blockRangeStart": 4952200,
    "blockRangeEnd": 4952229,
    "sampledProperty": "header.base_fee_per_gas"
  },
  "aggregateFnId": "sum"
}
```

### CLI

```bash
hdp encode -a -o example/header/sum_base_fee_per_gas/output.json -c example/header/sum_base_fee_per_gas/input.json "sum" -b 4952200 4952229 "header.base_fee_per_gas" 1
```
