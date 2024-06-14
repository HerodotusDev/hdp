# Compiler

This compiler is supporting both version of interfaces:

- v1 : task + datalake as processing unit
- v1.5: task + datalake as processing unit, have hash to pass in aggregate function + get from registry
- v2 : module + module inputs as processing unit

The compiler is responsible for accepting the task interface and transforming it into the `FetchKeyEnvelope` structure. This structure is necessary for initiating multiple providers through the `AbstractProvider`.

The compiler prioritizes merging fetch keys with the same chain_id. After merging, it organizes the sub-fetch points in the following distinct sequences:

- Header -> Account -> Storage
- Header -> Transaction / Transaction Receipt

After structuring the fetch keys accordingly, the final `FetchKeyEnvelope` is passed to the `AbstractProvider`. This provider coordinates with multiple sub-providers, utilizing multi-threading and caching mechanisms.
