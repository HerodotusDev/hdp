FROM dataprocessor/hdp-cairo:v0.0.15

# Set shell to bash and define working directory
SHELL ["/bin/bash", "-ci"]
WORKDIR /hdp-runner

# Install Rust using Rustup
RUN curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y && \
    echo 'export PATH="/root/.cargo/bin:$PATH"' >> /root/.bashrc

# Add Cargo executables to PATH
RUN mkdir -p /root/.local/bin && \
    echo 'export PATH="/root/.local/bin:$PATH"' >> /root/.bashrc


# Copy the rest of the application source
COPY . .

# Install Rust dependencies and build the Rust application
RUN cargo install --path cli

# Command to run the Rust binary with user-specified parameters
CMD ["bash", "-c", "source /root/.bashrc && hdp run -r /hdp-runner/request.json -p /hdp-runner/input.json -b /hdp-runner/batch.json -c /hdp-runner/cairo.pie"]
