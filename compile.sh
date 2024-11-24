#!/bin/bash
set -e

# Step 1: Download the export.js file
echo "Downloading export.js..."
wget -q -O src/data/export.js https://cgi.cse.unsw.edu.au/~xavc/hs/export.js

echo "Formatting export.js..."
prettier --write src/data/export.js

# Step 2: Modify export.js with sed to add 'export' to the const declarations
echo "Modifying export.js with sed..."
sed -i '' 's/^const /export const /' src/data/export.js

# Step 3: Run the convert_to_json.js script
echo "Running convert_to_json.js..."
node src/data/convert_to_json.js

# List of targets to compile for
targets=(
    "aarch64-apple-darwin"
    "x86_64-apple-darwin"
    "aarch64-unknown-linux-gnu"
    "x86_64-unknown-linux-gnu"
    "x86_64-pc-windows-gnu"
)

# Use cargo zigbuild for all except Windows
for target in "${targets[@]}"; do
    if [[ "$target" == "x86_64-pc-windows-gnu" ]]; then
        echo "Compiling for Windows 64-bit ($target)..."
        cargo build --release --target "$target"
    else
        echo "Compiling for $target..."
        cargo zigbuild --release --target "$target"
    fi
done

echo "Compilation for all targets completed successfully."

# Create the release directory
mkdir -p release

# Copy the compiled binaries to the release directory
for target in "${targets[@]}"; do
    if [[ "$target" == "x86_64-pc-windows-gnu" ]]; then
        # Handle Windows .exe extension
        cp "target/$target/release/help_session_to_ics.exe" "release/help_session_to_ics-$target.exe"
    else
        # Handle other targets without .exe
        cp "target/$target/release/help_session_to_ics" "release/help_session_to_ics-$target"
    fi
done

echo "All binaries have been copied to the release directory."

# Clean up .intentionally-empty-file.o files (if they exist)
rm .intentionally-empty-file.o
echo "Removed .intentionally-empty-file.o files."

rm src/data/allocations.json
rm src/data/tutors.json
rm src/data/export.js
echo "Removed data files."

