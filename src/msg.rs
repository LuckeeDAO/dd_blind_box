use cosmwasm_schema::{cw_serde, QueryResponses};
use cosmwasm_std::{Coin};
use crate::state::{VoteState, Scale};

/// 实例化参数：用于部署时设置规模、基础币种与一等奖中奖人数
#[cw_serde]
pub struct InstantiateMsg {
    pub scale: Scale,
    pub base: Coin,
    pub first_prize_count: Option<u32>,  // 可选的一等奖中奖人数，如果不提供则使用规模默认值
}

/// 执行消息入口（Execute）：涵盖参数更新、充值、投票、结算以及 NFT 合约操作
#[cw_serde]
pub enum ExecuteMsg {
    SetBase { base: Coin },
    Deposit {},
    SetVoteState { state: VoteState },
    // admin controls
    SetPaused { paused: bool },
    SetCommitWindow { start_height: Option<u64>, end_height: Option<u64>, start_time: Option<u64>, end_time: Option<u64> },
    SetRevealWindow { start_height: Option<u64>, end_height: Option<u64>, start_time: Option<u64>, end_time: Option<u64> },
    SetClosedWindow { start_height: Option<u64>, end_height: Option<u64>, start_time: Option<u64>, end_time: Option<u64> },
    SetNftContract { nft_contract: String },  // 设置NFT合约地址
    SetNftCodeId { code_id: u64 },           // 设置NFT合约代码ID
    InstantiateNftContract {                 // 实例化NFT合约
        name: String,
        symbol: String,
        base_uri: Option<String>,
    },
    CommitVote { commitment: String },
    RevealVote { reveal: String, salt: String },
    Finalize {},
    // NFT合约操作（通过外部NFT合约）
    TransferNft { recipient: String, token_id: u64 },
    Approve { spender: String, token_id: u64 },
    Revoke { spender: String, token_id: u64 },
    ApproveAll { operator: String },
    RevokeAll { operator: String },
}

/// 查询消息入口（Query）：查询配置、充值、本人的分层、NFT、授权等
#[cw_serde]
#[derive(QueryResponses)]
pub enum QueryMsg {
    #[returns(ConfigResponse)]
    Config {},
    #[returns(DepositResponse)]
    DepositOf { address: String },
    #[returns(TierResponse)]
    TierOf { address: String },
    #[returns(OwnerOfResponse)]
    OwnerOf { token_id: u64 },
    #[returns(TierListResponse)]
    TierList { tier: u8, start_after: Option<String>, limit: Option<u32> },
    // CW721-like
    #[returns(NftInfoResponse)]
    NftInfo { token_id: u64 },
    #[returns(ApprovalResponse)]
    Approval { token_id: u64 },
    #[returns(IsApprovedForAllResponse)]
    IsApprovedForAll { owner: String, operator: String },
    // CW721标准扩展查询
    #[returns(TokenUriResponse)]
    TokenUri { token_id: u64 },
    #[returns(AllTokensResponse)]
    AllTokens { start_after: Option<u64>, limit: Option<u32> },
    #[returns(TokensResponse)]
    Tokens { owner: String, start_after: Option<u64>, limit: Option<u32> },
}

/// 配置查询返回：拥有者、总供应量、基础币、阶段、规模、一等奖中奖人数、NFT合约地址、NFT代码ID
#[cw_serde]
pub struct ConfigResponse {
    pub owner: String,
    pub total_supply: u64,
    pub base: Coin,
    pub vote_state: VoteState,
    pub scale: Scale,
    pub first_prize_count: u32,
    pub nft_contract: Option<String>,
    pub nft_code_id: Option<u64>,
}

/// 迁移参数：空置接口，为未来升级预留
#[cw_serde]
pub struct MigrateMsg {
    // 空置接口，当前不需要任何参数
    // 为未来可能的合约升级预留
}

/// 充值查询返回：累计充值本金（字符串表示）
#[cw_serde]
pub struct DepositResponse { pub principal: String }

/// 分层查询返回：1/2/3（未设置为 0）
#[cw_serde]
pub struct TierResponse { pub tier: u8 }

/// NFT 拥有者查询返回
#[cw_serde]
pub struct OwnerOfResponse { pub owner: String }

/// 分层列表查询返回：地址数组与下一页起点
#[cw_serde]
pub struct TierListResponse { pub addresses: Vec<String>, pub next_start_after: Option<String> }

#[cw_serde]
pub struct NftInfoResponse { pub owner: String, pub approved: Option<String> }

#[cw_serde]
pub struct ApprovalResponse { pub spender: Option<String> }

#[cw_serde]
pub struct IsApprovedForAllResponse { pub approved: bool }

#[cw_serde]
pub struct TokenUriResponse { pub token_uri: Option<String> }

#[cw_serde]
pub struct AllTokensResponse { pub tokens: Vec<u64> }

#[cw_serde]
pub struct TokensResponse { pub tokens: Vec<u64> }


