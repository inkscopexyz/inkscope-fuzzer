import os
import subprocess
import json

def build_contracts(current_dir):
    # Check if the current directory has a Cargo.toml file
    if os.path.isfile(os.path.join(current_dir, "Cargo.toml")):
        print(f"Building contract in directory: {current_dir}")

        # Execute the build command
        subprocess.run(["cargo", "contract", "build", "--features", "fuzz-testing"], cwd=current_dir)

    # Recursively search in all subdirectories
    for subdir in [os.path.join(current_dir, d) for d in os.listdir(current_dir) if os.path.isdir(os.path.join(current_dir, d))]:
        build_contracts(subdir)

# Define the directory where contracts are located
contracts_dir = "./"

# Check if the directory exists
if not os.path.isdir(contracts_dir):
    print(f"Directory {contracts_dir} does not exist")
    exit(1)

# Call the recursive function starting from the contracts_dir
build_contracts(contracts_dir)