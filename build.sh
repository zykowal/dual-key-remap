#!/bin/bash

# Dual Key Remap - Build Script
echo "Building Dual Key Remap (Rust Version)..."

# 清理之前的构建
cargo clean

# 构建发布版本
echo "Building release version..."
cargo build --release

# 检查构建是否成功
if [ $? -eq 0 ]; then
    echo "Build successful!"
    
    # 复制配置文件到发布目录
    cp config.txt target/release/
    
    echo "Executable created at: target/release/dual-key-remap"
    echo "Configuration file copied to: target/release/config.txt"
    echo ""
    echo "To run the program:"
    echo "  ./target/release/dual-key-remap"
    echo ""
    echo "Note: On Windows, the executable will be dual-key-remap.exe"
else
    echo "Build failed!"
    exit 1
fi
