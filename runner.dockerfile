# Use the base image dataprocessor/hdp-cairo:v0.0.2
FROM dataprocessor/hdp-cairo:v0.0.2

# Set shell to bash and define working directory
SHELL ["/bin/bash", "-ci"]
WORKDIR /hdp-runner

# Install necessary dependencies
RUN apt-get update && apt-get install -y curl jq \
    && apt-get clean \
    && rm -rf /var/lib/apt/lists/*

# Fetch the latest release tag from GitHub and install HDP
ENV PATH="/root/.cargo/bin:/root/.local/bin:$PATH"
RUN LATEST_TAG=$(curl -s https://api.github.com/repos/HerodotusDev/hdp/releases/latest | jq -r .tag_name) \
    && cargo install --git https://github.com/HerodotusDev/hdp --tag $LATEST_TAG --locked --force \
    && echo 'export PATH="/root/.cargo/bin:$PATH"' >> /root/.bashrc

# Copy the rest of the application source
RUN mkdir -p /hdp-runner/build/compiled_cairo
RUN cp /hdp-cairo/build/hdp.json /hdp-runner/build/compiled_cairo/hdp.json

# Copy the rest of the application source
COPY . .

# Set environment variables for parameters
ENV TASKS=""
ENV DATALAKES=""
ENV RPC_URL=""
ENV CHAIN_ID=""

# Command to run the Rust binary with user-specified parameters
CMD ["bash", "-c", "hdp run $TASKS $DATALAKES $RPC_URL $CHAIN_ID -c /hdp-runner/input.json -o /hdp-runner/output.json -p /hdp-runner/cairo.pie"]