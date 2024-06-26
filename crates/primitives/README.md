# HDP Primitives

## TaskEnvelope

The `TaskEnvelope` definition is used as an input for the `pre-processor`. There are two different types of `TaskEnvelope` definitions:

- **DatalakeCompute**: This is a higher-level interface. HDP supports two types of Datalake interfaces: "Block Sampled Data Lake" and "Transactions In Block Data Lake". Users can submit requests using this defined interface and specify a predefined function tag as an aggregate function.
- **Module**: This is a more customized interface. It involves passing a contract hash and input bytes. During the Cairo runtime, the necessary values will be fetched without the need to adhere to a predefined interface.

Both task interfaces can be committed on-chain as a `bytes32` type.
