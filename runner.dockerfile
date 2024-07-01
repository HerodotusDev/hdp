FROM dataprocessor/hdp-cairo:v0.0.4

# Set shell to bash and define working directory
SHELL ["/bin/bash", "-ci"]
WORKDIR /hdp-runner

# Install Rust using Rustup
RUN curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y && \
    echo 'export PATH="/root/.cargo/bin:$PATH"' >> /root/.bashrc

# Add Cargo executables to PATH
RUN mkdir -p /root/.local/bin && \
    echo 'export PATH="/root/.local/bin:$PATH"' >> /root/.bashrc

# Create necessary directories
RUN mkdir -p /hdp-runner/build/compiled_cairo

# Copy specific file from the base image
RUN cp /hdp-cairo/build/hdp.json /hdp-runner/build/compiled_cairo/hdp.json
RUN cp /hdp-cairo/build/contract_dry_run.json /hdp-runner/build/compiled_cairo/contract_dry_run.json

# Copy the rest of the application source
COPY . .

# Install Rust dependencies and build the Rust application
RUN cargo install --path cli

# Command to run the Rust binary with user-specified parameters
CMD ["bash", "-c", "source /root/.bashrc && hdp run $TASKS $DATALAKES $RPC_URL $CHAIN_ID -c /hdp-runner/input.json -o /hdp-runner/output.json -p /hdp-runner/cairo.pie"]
