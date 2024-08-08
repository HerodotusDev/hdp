# private input and module

HDP provides privacy not only for provability.

- private module: In case you don't want to share custom module's computation logic (e.g logic contains sensitive strategy ), you can run hdp locally like this example on [private_module](./private_module/) and point to the module's contract class locally to generate PIE.

- private input: In case you don't want to reveal some inputs of module function (e.g private key), you can change visility into private so that it can be excluded from construction task commitment.
