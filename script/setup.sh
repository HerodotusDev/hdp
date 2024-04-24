#!/bin/bash

# Setup the hdp-cairo submodule
echo "Setting up hdp-cairo submodule..."
git submodule update --init --recursive

# Setup hdp-cairo virtual environment
echo "Setting up virtual environment..."
cd hdp-cairo

# Attempt to run 'make setup' and handle potential errors
if ! make setup; then
  echo "Failed to install hdp-cairo submodule. Please check the makefile within hdp-cairo."
  exit 1
fi

cd ..

# If we get this far, it means the installation was successful
echo "hdp-cairo submodule setup completed successfully."
