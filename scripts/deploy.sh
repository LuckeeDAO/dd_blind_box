#!/bin/bash

# 部署脚本：将 dd_blind_box 合约部署到 Neutron 网络
# 用法: ./scripts/deploy.sh [network] [key_name] [scale] [base_amount] [base_denom] [rpc?] [gas_prices?] [gas_adjustment?]
# 例如: ./scripts/deploy.sh neutron-pion-1 deployer Medium 100 ujunox
# 说明：
# - network: neutron-mainnet/neutron-1 或测试网 neutron-pion-1/pion-1
# - key_name: 本地 neutrond key 名称（--keyring-backend test）
# - scale: Tiny/Small/Medium/Large/Huge（将影响总供应量）
# - base_amount/base_denom: 基础币的最小充值单位与 denom

set -e

NETWORK=${1:-"neutron-pion-1"}
KEY_NAME=${2:-"deployer"}
SCALE=${3:-"Medium"}
BASE_AMOUNT=${4:-"100"}
BASE_DENOM=${5:-"untrn"}
RPC_URL=${6:-""}
GAS_PRICES=${7:-"0.025untrn"}
GAS_ADJUSTMENT=${8:-"1.5"}

case "$NETWORK" in
  "neutron-mainnet"|"neutron-1")
    CHAIN_ID="neutron-1"
    declare -a RPC_ENDPOINTS=(
      "https://rpc-neutron.whispernode.com"
      "https://neutron-rpc.polkachu.com"
      "https://rpc.neutron.whispernode.com"
    )
    EXPLORER_URL="https://neutron.celat.one/neutron-1/"
    ;;
  "neutron-pion-1"|"pion-1")
    CHAIN_ID="pion-1"
    declare -a RPC_ENDPOINTS=(
      "https://rpc-pion.neutron.org"
      "https://neutron-pion-rpc.polkachu.com"
      "https://rpc.pion.rs-testnet.polypore.xyz"
    )
    EXPLORER_URL="https://neutron.celat.one/pion-1/"
    ;;
  *)
    echo "❌ 不支持的网络: $NETWORK"; exit 1;;
esac

# 提示当前部署信息
echo "🚀 部署 dd_blind_box 到 $NETWORK"

if [ -z "$RPC_URL" ]; then
  for endpoint in "${RPC_ENDPOINTS[@]}"; do
    if curl -s --connect-timeout 10 "$endpoint/status" > /dev/null 2>&1; then
      RPC_URL="$endpoint"; break
    fi
  done
fi

if ! command -v neutrond &> /dev/null; then
  echo "❌ 缺少 neutrond CLI"; exit 1
fi

USE_DOCKER=false
if command -v docker &> /dev/null && docker ps &> /dev/null; then
  USE_DOCKER=true
fi

echo "🔨 构建合约..."
if [ "$USE_DOCKER" = true ]; then
  ./scripts/buildprod.sh
else
  ./scripts/build.sh
fi

CONTRACT_WASM="artifacts/dd_blind_box.wasm"
if [ ! -f "$CONTRACT_WASM" ]; then echo "❌ 未找到 $CONTRACT_WASM"; exit 1; fi

ADDRESS=$(neutrond keys show "$KEY_NAME" -a --keyring-backend test)

echo "📤 上传合约..."
UPLOAD_RESULT=$(neutrond tx wasm store "$CONTRACT_WASM" \
  --from "$KEY_NAME" \
  --keyring-backend test \
  --chain-id "$CHAIN_ID" \
  --node "$RPC_URL" \
  --gas-prices "$GAS_PRICES" \
  --gas-adjustment "$GAS_ADJUSTMENT" \
  --gas auto \
  --yes \
  --output json)

CODE_ID=$(echo "$UPLOAD_RESULT" | jq -r '.logs[0].events[] | select(.type=="store_code") | .attributes[] | select(.key=="code_id") | .value')
if [ -z "$CODE_ID" ] || [ "$CODE_ID" = "null" ]; then echo "❌ 获取 Code ID 失败"; echo "$UPLOAD_RESULT"; exit 1; fi
echo "🆔 Code ID: $CODE_ID"

INSTANTIATE_MSG='{"scale":"'$SCALE'","base":{"denom":"'$BASE_DENOM'","amount":"'$BASE_AMOUNT'"}}'

echo "🏗️  实例化..."
INSTANTIATE_RESULT=$(neutrond tx wasm instantiate "$CODE_ID" "$INSTANTIATE_MSG" \
  --from "$KEY_NAME" \
  --keyring-backend test \
  --chain-id "$CHAIN_ID" \
  --node "$RPC_URL" \
  --gas-prices "$GAS_PRICES" \
  --gas-adjustment "$GAS_ADJUSTMENT" \
  --gas auto \
  --label "dd_blind_box_$(date +%s)" \
  --yes \
  --output json)

CONTRACT_ADDRESS=$(echo "$INSTANTIATE_RESULT" | jq -r '.logs[0].events[] | select(.type=="instantiate") | .attributes[] | select(.key=="_contract_address") | .value')
if [ -z "$CONTRACT_ADDRESS" ] || [ "$CONTRACT_ADDRESS" = "null" ]; then echo "❌ 无法获取合约地址"; echo "$INSTANTIATE_RESULT"; exit 1; fi

echo "✅ 部署完成: $CONTRACT_ADDRESS"
echo "🔗 浏览器: ${EXPLORER_URL}contracts/$CONTRACT_ADDRESS"


