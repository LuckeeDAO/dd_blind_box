# API 使用示例

以下示例以 `CONTRACT` 为合约地址，`KEY` 为本地 key 名称，`RPC` 为节点 RPC。

## Instantiate
```
INSTANTIATE_MSG='{"scale":"Medium","base":{"denom":"untrn","amount":"100"}}'
neutrond tx wasm instantiate CODE_ID "$INSTANTIATE_MSG" \
  --from KEY --keyring-backend test \
  --chain-id CHAIN_ID --node RPC --gas-prices 0.025untrn \
  --gas auto --yes --label dd_blind_box_$(date +%s)
```

## Execute
- 设置 base：
```
neutrond tx wasm execute CONTRACT '{"set_base":{"base":{"denom":"untrn","amount":"100"}}}' \
  --from KEY --chain-id CHAIN_ID --node RPC --gas-prices 0.025untrn --gas auto --yes
```
- 存款领取 NFT（转账时附带 base denom）：
```
neutrond tx bank send $(neutrond keys show KEY -a --keyring-backend test) CONTRACT 300untrn \
  --chain-id CHAIN_ID --node RPC --gas-prices 0.025untrn --gas auto --yes
# 或封装 deposit 执行（本合约在 execute Deposit 中仅读取收到的资金）
neutrond tx wasm execute CONTRACT '{"deposit":{}}' --from KEY \
  --chain-id CHAIN_ID --node RPC --gas-prices 0.025untrn --gas auto --yes
```
- 提交承诺：
```
COMMITMENT="$(echo -n "<addr>|<reveal>|<salt>" | sha256sum | cut -d' ' -f1)"
neutrond tx wasm execute CONTRACT '{"commit_vote":{"commitment":"'"$COMMITMENT"'"}}' \
  --from KEY --chain-id CHAIN_ID --node RPC --gas-prices 0.025untrn --gas auto --yes
```
- 揭示：
```
neutrond tx wasm execute CONTRACT '{"reveal_vote":{"reveal":"<reveal>","salt":"<salt>"}}' \
  --from KEY --chain-id CHAIN_ID --node RPC --gas-prices 0.025untrn --gas auto --yes
```
- 切换投票状态：
```
neutrond tx wasm execute CONTRACT '{"set_vote_state":{"state":"Reveal"}}' --from KEY \
  --chain-id CHAIN_ID --node RPC --gas-prices 0.025untrn --gas auto --yes
```
- 结算：
```
neutrond tx wasm execute CONTRACT '{"finalize":{}}' --from KEY \
  --chain-id CHAIN_ID --node RPC --gas-prices 0.025untrn --gas auto --yes
```

## Query
- 配置：
```
neutrond query wasm contract-state smart CONTRACT '{"config":{}}' --node RPC
```
- 本金与分层：
```
neutrond query wasm contract-state smart CONTRACT '{"deposit_of":{"address":"ADDR"}}' --node RPC
neutrond query wasm contract-state smart CONTRACT '{"tier_of":{"address":"ADDR"}}' --node RPC
```
- NFT：
```
neutrond query wasm contract-state smart CONTRACT '{"owner_of":{"token_id":1}}' --node RPC
```
- 分层分页：
```
neutrond query wasm contract-state smart CONTRACT '{"tier_list":{"tier":1,"start_after":null,"limit":50}}' --node RPC
```
