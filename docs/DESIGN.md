# 设计说明（dd_blind_box）

## 概述
- 固定规模发售：Scale ∈ {Tiny=10, Small=100, Medium=1k, Large=10k, Huge=100k}
- 存款分发：用户按 base（denom+amount）整倍数转账，顺序领取 0..n-1 NFT
- 治理投票：承诺-揭示模型（commit=sha256(addr|reveal|salt)）
- 结算分层：使用 `dd_algorithms_lib`，按 10%/50%/40% 划分一/二/三等奖并资金返还（2x/1x/0.5x）

## 模块与文件
- `src/state.rs`：存储结构（Config、TokenInfo、Payout、Commit/Reveal、Map 常量）
- `src/msg.rs`：Instantiate/Execute/Query/Migrate 消息与响应结构
- `src/contract.rs`：instantiate/execute/query/migrate 主逻辑
- `src/lib.rs`：模块出口；`src/error.rs`：错误定义

## 状态与存储
- Config：owner、total_supply、base、vote_state、next_token_id、scale
- TOKENS：token_id → { owner }
- DEPOSITS：addr → { principal }
- COMMITS/REVEALS：addr → { commitment } / { reveal, salt }
- TIERS：addr → u8（1/2/3；未设置为 0）

## 生命周期流程
1. 实例化（Instantiate）：设置 scale 与 base，vote_state=Commit，total_supply 由 scale 决定
2. 存款（Deposit）：
   - 接受 base denom 的资金
   - 每 base.amount 为一个单位，按整倍数分发 token_id，从 0 递增
   - 记录地址的存入本金（累加）
3. 承诺（CommitVote）：记录地址的承诺字符串 commitment（推荐使用 sha256 预镜像）
4. 揭示（RevealVote）：校验 sha256(addr|reveal|salt) 与 commitment 一致，记录 reveal
5. 结算（Finalize）：
   - 要求 vote_state=Closed
   - 读取所有 reveal，将字符串映射为 3 组 u128 序列
   - 使用 `dd_algorithms_lib` 生成不相交的 10%/50%/40% 分层集合
   - 一等奖返 2x、本金保本、三等奖返 0.5x，按 base.denom 发送资金
   - 将 tier 结果写入 TIERS

## 随机数与分层策略
- 将每个揭示字符串按位散列生成 h0/h1/h2 三个值，形成 `[group0, group1, group2]`
- 先用 `get_k_dd_rand_num_with_whitelist(groups, n, num_first, [])` 选出 10%
- 再以第一层为白名单，用同方法选出 50%，剩余归三等奖
- 优点：
  - 确保分层互斥
  - `dd_algorithms_lib` 算法在 CosmWasm 环境内可运行（no_std）

## 安全与边界
- 提交/揭示状态机控制，避免提前揭示
- 承诺哈希校验防止事后伪造
- 当 `next_token_id >= total_supply` 时停止继续发放 NFT
- 对输入进行了基本健全性判断（空输入、溢出避免等）

## 扩展建议
- 引入投票时间窗口和自动切换状态
- 更严格的 reveal 规则（最小长度、字符范围）
- 增加批量查询与导出工具（如分页导出所有 tier=1 的地址）
