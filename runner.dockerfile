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

# Create necessary directories
RUN mkdir -p /hdp-runner/build/compiled_cairo

# Clone the program registry and set up required program directories
RUN git clone https://github.com/petscheit/cairo-program-registry-new.git /hdp-runner/cairo-programs

# Define environment variables for program hashes, which you can override at build time
ARG HDP_PROGRAM_HASH
ARG DRY_RUN_PROGRAM_HASH

# Clone the program registry and copy the required program.json files
RUN rm -rf /hdp-runner/cairo-programs && \
    git clone https://github.com/petscheit/cairo-program-registry-new.git /hdp-runner/cairo-programs && \
    if [ -f "/hdp-runner/cairo-programs/$HDP_PROGRAM_HASH/program.json" ]; then \
        cp "/hdp-runner/cairo-programs/$HDP_PROGRAM_HASH/program.json" "/hdp-runner/build/hdp.json"; \
    else \
        echo "Error: program.json for HDP_PROGRAM_HASH not found." && exit 1; \
    fi && \
    if [ -f "/hdp-runner/cairo-programs/$DRY_RUN_PROGRAM_HASH/program.json" ]; then \
        cp "/hdp-runner/cairo-programs/$DRY_RUN_PROGRAM_HASH/program.json" "/hdp-runner/build/dry_run_program.json"; \
    else \
        echo "Error: program.json for DRY_RUN_PROGRAM_HASH not found." && exit 1; \
    fi

# Copy the rest of the application source
COPY . .

# Install Rust dependencies and build the Rust application
RUN cargo install --path cli

# Command to run the Rust binary with user-specified parameters
CMD ["bash", "-c", "source /root/.bashrc && hdp run -r /hdp-runner/request.json -p /hdp-runner/input.json -b /hdp-runner/batch.json -c /hdp-runner/cairo.pie"]
