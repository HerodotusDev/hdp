## Request

30 Blocks, Storage, Sum

### Server

```json
{
  "chainId": 11155111,
  "datalakeType": "block_sampled",
  "datalake": {
    "blockRangeStart": 5382810,
    "blockRangeEnd": 5382839,
    "sampledProperty": "storage.0x75CeC1db9dCeb703200EAa6595f66885C962B920.0x0000000000000000000000000000000000000000000000000000000000000002"
  },
  "aggregateFnId": "sum"
}
```

### CLI

```bash
hdp encode -a -o example/sum_storage/output.json -c example/sum_storage/input.json "sum" -b 5382810 5382839 "storage.0x75CeC1db9dCeb703200EAa6595f66885C962B920.0x0000000000000000000000000000000000000000000000000000000000000002" 1
```
