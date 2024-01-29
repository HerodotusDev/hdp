# Herodotus Data Processor (HDP)

[![CI](https://github.com/HerodotusDev/hdp/actions/workflows/ci.yml/badge.svg)](https://github.com/HerodotusDev/hdp/actions/workflows/ci.yml)

## Main Workflow in `main.rs`

- **`EventWatcher`**: Scrapes events from `HreExecutionStore.sol`.
- **`Decoder`**: Decodes events into `Datalake` and `Task` .
- **`Task Generator`**: Inputs a datalake and task, outputs multiple tasks and proof requests.
- **`Fetcher`**: Retrieves proofs for the generated tasks.
- **`Evaluator`**: Processes tasks with their proofs and evaluates results.
- **Post-Processing**: Sends the evaluation root to Cairo HDP.
