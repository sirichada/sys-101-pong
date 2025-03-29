#!/bin/bash

# Build with errors suppressed
echo "Building and running OS..."
cargo run >/dev/null 2>&1

# If it fails, build just kernel and run QEMU manually
if [ $? -ne 0 ]; then
    echo "Cargo run failed. Building kernel directly..."
    cd kernel
    cargo build --target x86_64-unknown-none
    cd ..
    
    echo "Looking for UEFI disk image..."
    BUILD_DIR=$(cargo metadata --format-version=1 | grep -o '"target_directory":"[^"]*"' | cut -d'"' -f4)
    UEFI_PATH=$(find "$BUILD_DIR" -name "uefi.img" 2>/dev/null | head -n 1)
    
    if [ -z "$UEFI_PATH" ]; then
        echo "No UEFI image found. Check build.rs output."
        exit 1
    fi
    
    echo "Running QEMU with UEFI image at $UEFI_PATH..."
    qemu-system-x86_64 \
        -bios RELEASEX64_OVMF.fd \
        -drive format=raw,file="$UEFI_PATH" \
        -serial stdio \
        -no-reboot \
        -m 128M
fi

## chmod +x run.sh
## ./run.sh
