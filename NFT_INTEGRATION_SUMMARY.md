# NFT合约集成总结

## 概述
已成功将 `dd_blind_box` 工程中的NFT实现替换为使用 `/home/lc/luckee_dao/luckee_nft` 定义的NFT合约。

## 主要更改

### 1. 依赖配置 (`Cargo.toml`)
- 添加了 `luckee_nft = { path = "../luckee_nft" }` 依赖

### 2. 状态管理 (`src/state.rs`)
- 在 `Config` 结构体中添加了 `nft_contract: Option<Addr>` 字段
- 移除了本地NFT存储：
  - 删除了 `TokenInfo` 结构体
  - 注释掉了 `TOKENS` 和 `OPERATORS` 存储映射

### 3. 消息定义 (`src/msg.rs`)
- 在 `ExecuteMsg` 中添加了 `SetNftContract { nft_contract: String }` 消息
- 在 `ConfigResponse` 中添加了 `nft_contract: Option<String>` 字段

### 4. 合约实现 (`src/contract.rs`)

#### 新增功能：
- `exec_set_nft_contract()`: 设置NFT合约地址（仅拥有者可执行）

#### 修改的功能：

**NFT铸造 (`exec_deposit`)**：
- 改为通过外部NFT合约进行批量铸造
- 使用 `luckee_nft::msg::ExecuteMsg::BatchMint` 消息
- 创建符合 `luckee_nft::types::NftMeta` 结构的元数据
- 盲盒NFT默认为 `NftKind::Clover`（四叶草）类型

**NFT操作函数**：
- `exec_transfer()`: 通过外部NFT合约执行转移
- `exec_approve()`: 通过外部NFT合约执行授权
- `exec_revoke()`: 通过外部NFT合约执行撤销授权
- `exec_approve_all()`: 通过外部NFT合约执行全局授权
- `exec_revoke_all()`: 通过外部NFT合约执行全局撤销授权

**查询函数**：
- 所有NFT相关查询函数现在返回错误信息，提示用户直接查询NFT合约
- `query_config()`: 现在包含NFT合约地址信息

## 使用方式

### 1. 部署和初始化
1. 首先部署 `luckee_nft` 合约
2. 部署 `dd_blind_box` 合约
3. 调用 `SetNftContract` 设置NFT合约地址

### 2. NFT铸造
- 用户通过 `Deposit` 消息充值
- 系统自动通过外部NFT合约铸造相应数量的NFT
- NFT元数据包含盲盒规模、系列ID等信息

### 3. NFT操作
- 所有NFT操作（转移、授权等）现在通过外部NFT合约执行
- 查询NFT信息需要直接查询NFT合约

## 技术细节

### NFT元数据结构
```rust
NftMeta {
    kind: NftKind::Clover,  // 盲盒NFT默认为四叶草
    scale_origin: Scale,    // 来源规模
    physical_sku: None,    // 物理SKU（可选）
    crafted_from: None,    // 合成来源（可选）
    series_id: String,     // 系列ID
    collection_group_id: Some(String), // 集合组ID
    serial_in_series: u64, // 系列内序号
}
```

### 类型转换
- 当前工程的 `Scale` 枚举与 `luckee_nft` 的 `Scale` 枚举兼容
- 在创建NFT元数据时进行类型转换

## 兼容性
- 保持了原有的盲盒功能（充值、投票、结算）
- NFT相关接口保持兼容，但实现改为调用外部合约
- 查询接口会提示用户直接查询NFT合约

## 注意事项
1. 必须先设置NFT合约地址才能进行充值操作
2. NFT查询需要直接查询NFT合约，而不是盲盒合约
3. 所有NFT操作现在通过外部合约执行，需要确保NFT合约已正确部署和配置
