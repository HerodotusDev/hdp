#!/bin/bash

process_cairo_file() {
cd hdp-cairo && source ./venv/bin/activate 
cairo_file=./src/hdp.cairo
cairo-compile --version
echo "Compiling $cairo_file using cairo-compile ..."
pwd
cairo-compile --cairo_path="packages/eth_essentials" "$cairo_file" --output "../compiled_cairo/hdp.json"
 local status=$?
    if [ $status -eq 0 ]; then
        echo "$(date '+%Y-%m-%d %H:%M:%S') - Successfully compiled $1"
    else
        echo "$(date '+%Y-%m-%d %H:%M:%S') - Failed to compile $1"
        return $status
    fi
}



# Call the function to ensure the virtual environment is activated
process_cairo_file
