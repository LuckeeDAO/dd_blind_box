#!/bin/bash

# 构建脚本（不使用 Docker）：
# 用于本地开发环境编译 CosmWasm 合约并输出 wasm 文件到 artifacts 目录

set -e

echo "Building dd_blind_box contract..."

# 清理旧产物
echo "Cleaning previous builds..."
rm -rf artifacts/*.wasm

# 构建合约（wasm 目标）
echo "Building contract..."
cargo build --release --target wasm32-unknown-unknown

# 构建成功性检查与产物输出信息
if [ -f "target/wasm32-unknown-unknown/release/dd_blind_box.wasm" ]; then
    echo "✅ Contract built successfully!"
    
    # Copy to artifacts directory
    mkdir -p artifacts
    cp target/wasm32-unknown-unknown/release/dd_blind_box.wasm artifacts/dd_blind_box.wasm
    
    echo "📁 Output: artifacts/dd_blind_box.wasm"
    
    # Display file size
    file_size=$(du -h artifacts/dd_blind_box.wasm | cut -f1)
    echo "📊 File size: $file_size"
    
    # Display checksum
    checksum=$(sha256sum artifacts/dd_blind_box.wasm | cut -d' ' -f1)
    echo "🔐 SHA256: $checksum"
else
    echo "❌ Build failed - target/wasm32-unknown-unknown/release/blind_box.wasm not found"
    exit 1
fi


