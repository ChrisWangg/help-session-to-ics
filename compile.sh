#!/bin/bash
set -e

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
