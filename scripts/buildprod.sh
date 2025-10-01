#!/bin/bash

# 生产构建脚本：
# 使用 cosmwasm/optimizer（带多源镜像）优化构建，生成压缩优化后的 wasm

set -e

echo "Building dd_blind_box contract..."

# 检查 Docker 是否可用
if ! command -v docker &> /dev/null; then
    echo "Error: Docker is not installed or not in PATH"
    echo "Please install Docker to build the contract"
    exit 1
fi

# 清理旧产物
echo "Cleaning previous builds..."
rm -rf artifacts/*.wasm

echo "Building contract with cosmwasm/optimizer (using China-friendly mirrors)..."

MIRROR_LIST=( \
  "docker.xuanyuan.me/cosmwasm/optimizer:0.16.0" \
  "docker.m.daocloud.io/cosmwasm/optimizer:0.16.0" \
  "dockerproxy.cn/cosmwasm/optimizer:0.16.0" \
  "dockerpull.org/cosmwasm/optimizer:0.16.0" \
  "cosmwasm/optimizer:0.16.0" \
  "ghcr.io/cosmwasm/optimizer:0.16.0" \
)

run_optimizer() {
  local image="$1"
  echo "Trying optimizer image: $image"
  docker run --rm -v "$(pwd)":/code \
    --mount type=volume,source="$(basename "$(pwd)")_cache",target=/target \
    --mount type=volume,source=registry_cache,target=/usr/local/cargo/registry \
    "$image"
}

OPT_SUCCESS=false
for img in "${MIRROR_LIST[@]}"; do
  if run_optimizer "$img"; then
    OPT_SUCCESS=true
    break
  else
    echo "Optimizer run failed for $img, trying next..."
  fi
done

if [ "$OPT_SUCCESS" != true ]; then
  echo "❌ All optimizer image sources failed. Please check Docker networking."
  exit 1
fi

if [ -f "artifacts/dd_blind_box.wasm" ]; then
    echo "✅ Contract built successfully!"
    echo "📁 Output: artifacts/dd_blind_box.wasm"
    file_size=$(du -h artifacts/dd_blind_box.wasm | cut -f1)
    echo "📊 File size: $file_size"
    checksum=$(sha256sum artifacts/dd_blind_box.wasm | cut -d' ' -f1)
    echo "🔐 SHA256: $checksum"
else
    echo "❌ Build failed - artifacts/dd_blind_box.wasm not found"
    exit 1
fi


