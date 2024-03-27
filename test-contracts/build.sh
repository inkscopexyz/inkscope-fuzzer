#!/bin/bash

# Define the directory where contracts are located
contracts_dir="./"

# Check if the directory exists
if [ ! -d "$contracts_dir" ]; then
    echo "Directory $contracts_dir does not exist"
    exit 1
fi

# Enter the contracts directory
cd "$contracts_dir" || exit

# Loop through each subdirectory
for contract_dir in *; do
    if [ -d "$contract_dir" ]; then
        echo "Building contract in directory: $contract_dir"
        # Enter the contract directory and execute the build command
        cd "$contract_dir" || continue
        cargo contract build --features fuzz-testing

        # Parse and modify the .contract file
        contract_file="target/ink/${contract_dir}.contract"
        echo "Parsing and modifying .contract file: $contract_file"
        if [ -f "$contract_file" ]; then
            jq '.version = (.version | tonumber)' "$contract_file" > "$contract_file.tmp" && mv "$contract_file.tmp" "$contract_file"
        else
            echo ".contract file not found in directory: $contract_dir"
        fi

        # Return to the parent directory
        cd ..
    fi
done