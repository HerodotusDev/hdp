#!/bin/bash

# Base directory where the folders 'storage' and 'account' are located
BASE_DIR="example"

# Function to process each input.json file
process_file() {
    inputFilePath=$1
    # Extract the base directory and subfolder name from the input file path
    baseDir=$(dirname "${inputFilePath}")
    subFolder=$(basename "${baseDir}")
    
    # Define the output .pie file path
    pieFilePath="${baseDir}/${subFolder}.pie"
    
    # Run the cairo-run command
    cairo-run \
        --program=compiled_cairo/v1_hdp.json \
        --layout=starknet_with_keccak \
        --program_input="${inputFilePath}" \
        --cairo_pie_output "${pieFilePath}" \
        --print_output
    
    # Check if cairo-run was successful
    if [ $? -ne 0 ]; then
        echo "Error processing file: ${inputFilePath}"
    else
        echo "Successfully processed file: ${inputFilePath}"
    fi
}

# Loop through 'storage' and 'account' directories
for dir in storage account header; do
    # Find all input.json files within the subdirectories of each main directory
    find "${BASE_DIR}/${dir}" -type f -name "input.json" | while read -r file; do
        process_file "$file"
    done
done
