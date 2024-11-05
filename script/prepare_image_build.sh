#!/bin/bash

# Source the environment variables from the JSON config file
source ./script/config_to_env.sh config/config.json

# Define the default paths for the programs
DEFAULT_DRY_CAIRO_RUN_CAIRO_FILE="build/contract_dry_run.json"
DEFAULT_SOUND_CAIRO_RUN_CAIRO_FILE="build/hdp.json"

clone_registry() {
    echo "Cloning program registry..."
    temp_dir=$(mktemp -d)
    git clone https://github.com/petscheit/cairo-program-registry-new.git "$temp_dir/cairo-program-registry"

    if [ $? -eq 0 ]; then
        rm -rf cairo-programs
        mv "$temp_dir/cairo-program-registry" cairo-programs
        rm -rf "$temp_dir"
        echo "Programs directory created/updated successfully."
    else
        echo "Failed to clone registry."
        rm -rf "$temp_dir"
        exit 1
    fi
}

check_program_dir() {
    local hash=$1
    if [ ! -d "cairo-programs/$hash" ]; then
        echo "Required program directory for hash $hash not found."
        return 1
    fi
    return 0
}

# Check if cairo-programs directory exists and contains required directories
if [ ! -d "cairo-programs" ] || \
   ! check_program_dir "$HDP_PROGRAM_HASH" || \
   ! check_program_dir "$DRY_RUN_PROGRAM_HASH"; then
    echo "Required program directories not found. Cloning/updating registry..."
    clone_registry
fi

# Final check for both program directories
if ! check_program_dir "$HDP_PROGRAM_HASH" || \
   ! check_program_dir "$DRY_RUN_PROGRAM_HASH"; then
    echo "Error: Required program directories still not available after clone/refetch."
    exit 1
else
    echo "Required program directories found."
fi

# Create the build directory if it doesn't exist
mkdir -p build

# Move the HDP program to the default path
if [ -f "cairo-programs/$HDP_PROGRAM_HASH/program.json" ]; then
    cp "cairo-programs/$HDP_PROGRAM_HASH/program.json" "$DEFAULT_SOUND_CAIRO_RUN_CAIRO_FILE"
    echo "HDP program moved to $DEFAULT_SOUND_CAIRO_RUN_CAIRO_FILE."
else
    echo "Error: HDP program file not found in cairo-programs/$HDP_PROGRAM_HASH."
    exit 1
fi

# Move the Dry Run program to the default path
if [ -f "cairo-programs/$DRY_RUN_PROGRAM_HASH/program.json" ]; then
    cp "cairo-programs/$DRY_RUN_PROGRAM_HASH/program.json" "$DEFAULT_DRY_CAIRO_RUN_CAIRO_FILE"
    echo "Dry Run program moved to $DEFAULT_DRY_CAIRO_RUN_CAIRO_FILE."
else
    echo "Error: Dry Run program file not found in cairo-programs/$DRY_RUN_PROGRAM_HASH."
    exit 1
fi
