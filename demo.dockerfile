FROM dataprocessor/hdp-cairo:v0.0.8

# Set shell to bash and define working directory
SHELL ["/bin/bash", "-ci"]
WORKDIR /hdp-demo

# Install Rust using Rustup
RUN curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y && \
    echo 'export PATH="/root/.cargo/bin:$PATH"' >> /root/.bashrc

# Add Cargo executables to PATH
RUN mkdir -p /root/.local/bin && \
    echo 'export PATH="/root/.local/bin:$PATH"' >> /root/.bashrc

# Create necessary directories
RUN mkdir -p /hdp-demo/build/compiled_cairo

# Copy specific file from the base image
RUN cp /hdp-cairo/build/hdp.json /hdp-demo/build/compiled_cairo/hdp.json
RUN cp /hdp-cairo/build/contract_dry_run.json /hdp-demo/build/compiled_cairo/contract_dry_run.json

# Copy the rest of the application source
COPY . .

# Install Rust dependencies and build the Rust application
RUN cargo install --path cli

# Run the final command ensuring the environment is correctly sourced
CMD source /root/.bashrc && \
    hdp run-module 0x4F21E5,0x4F21E8,0x13cb6ae34a13a0977f4d7101ebc24b87bb23f0d5 -p /hdp-demo/hdp_input.json -o /hdp-demo/output.json -c /hdp-demo/cairo.pie --chain-id 11155111 --class-hash 0x00ababb33ae5911fd14e6b9f2853b6271f553b9ec7835298134f4bb020100971 --rpc-url https://sepolia.ethereum.iosis.tech/ --module-registry-rpc-url https://pathfinder.sepolia.iosis.tech/