#!/bin/bash

# Base directory where the folders 'storage' and 'account' and 'header' are located
BASE_DIR="example"

# Use command line arguments as target directories. If no arguments are provided, default to "header account storage".
TARGET_DIRS=${@:-"header account storage min_max_count"}

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

# Loop through specified directories
for dir in $TARGET_DIRS; do
    # Find all directories within the main directories
    find "${BASE_DIR}/${dir}" -type d | while read -r subDir; do
        # Check if run.sh exists in the directory
        if [[ -f "${subDir}/run.sh" ]]; then
            echo "Running script in ${subDir}"
            # Make sure run.sh is executable
            chmod +x "${subDir}/run.sh"
            # Execute run.sh and wait for it to finish
            "${subDir}/run.sh"
            # Check for the existence of the input.json file after run.sh has completed
            inputFilePath="${subDir}/input.json"
            if [[ -f "${inputFilePath}" ]]; then
                process_file "${inputFilePath}"
            else
                echo "No input.json found in ${subDir} after running run.sh"
            fi
        else
            echo "No run.sh found in ${subDir}"
        fi
    done
done
