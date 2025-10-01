use cosmwasm_std::{Addr, Coin, Uint128};
use cw_storage_plus::{Item, Map};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

/// 全局配置（只存一份）：包括拥有者、总供应量、基础币、阶段、下一个 token_id、规模、暂停与阶段窗口
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct Config {
    pub owner: Addr,
    pub total_supply: u64,
    pub base: Coin,
    pub vote_state: VoteState,
    pub next_token_id: u64,
    pub scale: Scale,
    pub paused: bool,
    pub commit_window: PhaseWindow,
    pub reveal_window: PhaseWindow,
    pub closed_window: PhaseWindow,
}

/// 投票状态机：提交/揭示/关闭
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub enum VoteState {
    Commit,
    Reveal,
    Closed,
}

/// 预设规模（决定总供应量）
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub enum Scale {
    Tiny,
    Small,
    Medium,
    Large,
    Huge,
}

/// 阶段窗口（可设置区块高度或时间的闭区间，满足已设置的所有维度）
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct PhaseWindow {
    pub start_height: Option<u64>,
    pub end_height: Option<u64>,
    pub start_time: Option<u64>,
    pub end_time: Option<u64>,
}

/// 最小化的 Token 信息：顺序 id → 所有者，单次授权地址
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct TokenInfo {
    pub owner: Addr,
    pub approved: Option<Addr>,
}

/// 承诺记录：保存 commitment 字符串
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct CommitInfo {
    pub commitment: String,
}

/// 揭示记录：保存 reveal 与 salt
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct RevealInfo {
    pub reveal: String,
    pub salt: String,
}

/// 充值本金：按地址累计
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct Payout {
    pub principal: Uint128,
}

/// 单实例配置项
pub const CONFIG: Item<Config> = Item::new("config");
pub const TOKENS: Map<u64, TokenInfo> = Map::new("tokens");
/// （owner, operator）→ 是否为全局操作员
pub const OPERATORS: Map<(Addr, Addr), bool> = Map::new("operators");
pub const COMMITS: Map<Addr, CommitInfo> = Map::new("commits");
pub const REVEALS: Map<Addr, RevealInfo> = Map::new("reveals");
pub const DEPOSITS: Map<Addr, Payout> = Map::new("deposits");
/// 地址 → 分层结果（1/2/3）
pub const TIERS: Map<Addr, u8> = Map::new("tiers");


