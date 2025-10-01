# 部署说明（Neutron）

## 依赖
- Rust 1.74+，wasm32-unknown-unknown 目标
- neutrond CLI（`neutrond`）
- jq
- 可选：Docker（用于优化构建）

## 构建
- 开发构建：
```
./scripts/build.sh
```
- 优化构建（需 Docker）：
```
./scripts/buildprod.sh
```
产物：`artifacts/dd_blind_box.wasm`

## 部署
```
# pion-1 测试网示例
./scripts/deploy.sh neutron-pion-1 deployer Medium 100 untrn
```
参数说明：
- network：`neutron-pion-1` 或 `neutron-1`
- key_name：本地 key 名（test keyring）
- scale：`Tiny|Small|Medium|Large|Huge`
- base_amount / base_denom：实例化 base
- 可选 rpc、gas_prices、gas_adjustment

脚本流程：
1) 构建 wasm
2) `neutrond tx wasm store` 上传，提取 code_id
3) `neutrond tx wasm instantiate` 实例化，打印合约地址

## 常见问题
- 无法连接 RPC：手动传入第 6 个参数指定 `--node`
- 余额不足：给部署人地址充值 NTRN（测试网可申请水龙头）
