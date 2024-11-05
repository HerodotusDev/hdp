#!/bin/bash

# This script loads the config params from the passed JSON file, writing them to the environment variables.
# These can then be used in the docker compose files, making then available to the different services

# Path to your JSON file
JSON_FILE="$1"

# Check if the file exists
if [ ! -f "$JSON_FILE" ]; then
  echo "Config file '$JSON_FILE' not found!"
  exit 1
fi

# Read JSON and export variables
export_vars() {
    jq -r 'to_entries | .[] | "export " + .key + "=" + (.value | @sh)' "$JSON_FILE"
}

# Export variables
eval "$(export_vars)"

# Create a string with all the environment variables
ENV_VARS=$(jq -r 'to_entries | map("\(.key)=\(.value | @sh)") | join("\n")' "$JSON_FILE")

# Export the variables and run Docker Compose
export $(echo "$ENV_VARS" | xargs)