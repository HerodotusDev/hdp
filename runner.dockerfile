# Use the base image dataprocessor/hdp-cairo:v0.0.2
FROM dataprocessor/hdp-cairo:v0.0.2

# Set shell to bash and define working directory
SHELL ["/bin/bash", "-ci"]
WORKDIR /hdp-runner

# Create necessary directories
RUN mkdir -p /hdp-runner/build/compiled_cairo

# Copy specific file from the base image
RUN cp /hdp-cairo/build/hdp.json /hdp-runner/build/compiled_cairo/hdp.json

# Copy the rest of the application source
COPY . .

# Install Rust dependencies and build the Rust application
RUN cargo install --path cli

# Command to run the Rust binary with user-specified parameters
CMD ["bash", "-c", "source /root/.bashrc && hdp run $TASKS $DATALAKES $RPC_URL $CHAIN_ID -c /hdp-runner/input.json -o /hdp-runner/output.json -p /hdp-runner/cairo.pie"]
