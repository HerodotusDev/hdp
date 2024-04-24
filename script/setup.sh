#!/bin/bash

# Setup hdp-cairo virtual environment
echo "Setting up virtual environment..."
cd hdp-cairo

# Attempt to run 'make setup' and handle potential errors
if ! make setup VENV_PATH=../venv; then
  echo "Failed to install environment. Please check the makefile within hdp-cairo."
  exit 1
fi

cd .. 

# If we get this far, it means the installation was successful
echo "hdp-cairo setup completed successfully."
