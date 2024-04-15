# Integration Examples

This section provides step-by-step instructions to run integration examples for HDP using the `script/integration.sh`. These examples demonstrate how to process requests and integrate them with the Cairo Program. Ensure you have the `RPC_URL` set in your `.env` file, as the script utilizes configuration from the environment variables.

## Preparing the Environment

### Install cairo-lang v0.13.1

1. **Clone the Cairo Program repository:**
   ```bash
   git clone https://github.com/HerodotusDev/offchain-evm-headers-processor-mirror
   ```
2. **Navigate to the project directory and set up the environment**:
   Follow the instructions provided in the installation [guideline](https://github.com/HerodotusDev/offchain-evm-headers-processor-mirror/tree/main/src/hdp).
3. **Activate the virtual environment**: `source venv/bin/activate`
4. **Verify the installation of Cairo**: `cairo-compile --version`

### Running the Script

Ensure you are within the virtual environment when executing the script to ensure all dependencies are correctly resolved.

```bash
./script/integration.sh
```

### Understanding the Logs

The integration script logs several key steps, providing insights into the process:

1. **Request Handling**: Captures and logs the command-line request for data processing.
2. **Preprocessing and Fetching**: Utilizes the HDP CLI to preprocess data and fetch necessary information through configured RPC endpoints.
3. **Cairo Program Execution**: Inputs the preprocessed .json files to the Cairo Program (HDP Cairo), executing the computations and logging the results.

```console
Running script in example/storage/sum_storage
2024-03-20T04:25:23.720761Z  INFO hdp: Encoded datalakes: 0x00000000000000000000000000000000000000000000000000000000000000200000000000000000000000000000000000000000000000000000000000000001000000000000000000000000000000000000000000000000000000000000002000000000000000000000000000000000000000000000000000000000000001000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000052229a00000000000000000000000000000000000000000000000000000000005222b7000000000000000000000000000000000000000000000000000000000000000100000000000000000000000000000000000000000000000000000000000000a000000000000000000000000000000000000000000000000000000000000000350375cec1db9dceb703200eaa6595f66885c962b92000000000000000000000000000000000000000000000000000000000000000020000000000000000000000
2024-03-20T04:25:23.720795Z  INFO hdp: Encoded tasks: 0x0000000000000000000000000000000000000000000000000000000000000020000000000000000000000000000000000000000000000000000000000000000100000000000000000000000000000000000000000000000000000000000000200000000000000000000000000000000000000000000000000000000000000080000000000000000000000000000000000000000000000000000000000000002073756d000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000400000000000000000000000000000000000000000000000000000000000000000
2024-03-20T04:25:23.725932Z  INFO hdp: datalakes: [
    BlockSampled(
        BlockSampledDatalake {
            block_range_start: 5382810,
            block_range_end: 5382839,
            sampled_property: "storage.0x75cec1db9dceb703200eaa6595f66885c962b920.0x0000000000000000000000000000000000000000000000000000000000000002",
            increment: 1,
        },
    ),
]
2024-03-20T04:25:23.725970Z  INFO hdp: tasks: [
    ComputationalTask {
        datalake: None,
        aggregate_fn_id: "sum",
        aggregate_fn_ctx: None,
    },
]
2024-03-20T04:25:29.885799Z  INFO hdp_provider::evm: Successfully fetched MMR data from indexer
2024-03-20T04:25:29.885815Z  INFO hdp_provider::evm: Time taken (fetch from Indexer): 6.159720875s
2024-03-20T04:25:30.939781Z  INFO hdp_provider::evm: Time taken (Storage Fetch): 1.05383175s
2024-03-20T04:25:30.940630Z  INFO hdp: Output file saved to: example/storage/sum_storage/output.json
2024-03-20T04:25:30.944392Z  INFO hdp: Cairo input file saved to: example/storage/sum_storage/input.json
2024-03-20T04:25:30.944425Z  INFO hdp: HDP Cli Finished in: 7.226253209s
property_type:  3
Computing Sum
result: 300000000000000 0
Tasks Root: 0xd6041eb499882324485755df92b06f87 0x8019aba1300f2818d8ea89ec06790eec
Results Root: 0xa4b134fca12b7feec62626804f3723f1 0x99861db50a9510894eed61ed3c63d55a
Program output:
  -40042192817701121787654147447478065151574308074341030539244937213005709226
  660813
  218913502553362885664575915674559783921
  204068253667586774572883969655548663130
  284476183066279284595878119826969882503
  170274471944582595366331826059465002732
Successfully processed file: example/storage/sum_storage/input.json
```
