# Data Processor Core

- `Pre-Processor`: 'dry-run' before actually processing the request

  - Input: serialized bytes of batched tasks
  - Validate the request if it's processable
  - Get Sierra bytes of target contract
  - Construct Input file with contract
  - Output: Identify values that requires for process as return

- `Processor`: actual `cairo-run` responsible to generate PIE
  - Input: Identified values array
  - Provider: Fetch Proofs
  - Get Sierra bytes of target contract
  - Construct Input file with proofs and contract
  - Output: PIE of final run
