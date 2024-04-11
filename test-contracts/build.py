import os
import subprocess
import json

def build_contracts(current_dir):
    # Check if the current directory has a Cargo.toml file
    if os.path.isfile(os.path.join(current_dir, "Cargo.toml")):
        print(f"Building contract in directory: {current_dir}")

        # Check if the targer dir exists
        target_dir = os.path.join(current_dir, "target")

        # If it exists and the .contract file also exists, skip the build
        if os.path.exists(target_dir):
            contract_file = next((os.path.join(current_dir, "target", "ink", f) for f in os.listdir(os.path.join(current_dir, "target", "ink")) if f.endswith(".contract")), None)
        else:
            contract_file = None
            
        if not contract_file:

            # Execute the build command
            subprocess.run(["cargo", "contract", "build", "--features", "fuzz-testing"], cwd=current_dir)

            # Find the .contract file and modify the version field
            contract_file = next((os.path.join(current_dir, "target", "ink", f) for f in os.listdir(os.path.join(current_dir, "target", "ink")) if f.endswith(".contract")), None)
            if contract_file:
                print(f"Parsing and modifying .contract file: {contract_file}")
                with open(contract_file, "r") as f:
                    contract_data = json.load(f)
                contract_data["version"] = int(contract_data["version"])
                with open(f"{contract_file}.tmp", "w") as f:
                    json.dump(contract_data, f)
                os.rename(f"{contract_file}.tmp", contract_file)
            else:
                print(f".contract file not found in directory: {current_dir}")
        else:
            print(f".contract file already exists in {current_dir}, skipping build.")

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