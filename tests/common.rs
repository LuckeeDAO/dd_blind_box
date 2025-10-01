#![allow(dead_code)]

use cosmwasm_std::{
    coins, testing::{mock_dependencies, mock_env}, Coin, Uint128, OwnedDeps, MessageInfo
};
use dd_blind_box::{
    contract::{instantiate, query},
    msg::{ExecuteMsg, InstantiateMsg, QueryMsg},
    state::{Scale, VoteState},
};

/// 测试常量
pub const OWNER: &str = "owner";
pub const USER1: &str = "user1";
pub const USER2: &str = "user2";
pub const USER3: &str = "user3";
pub const OPERATOR: &str = "operator";
pub const BASE_DENOM: &str = "ujunox";
pub const BASE_AMOUNT: u128 = 100;

/// 创建测试环境
pub fn setup_test_env() -> (OwnedDeps<cosmwasm_std::MemoryStorage, cosmwasm_std::testing::MockApi, cosmwasm_std::testing::MockQuerier>, cosmwasm_std::Env) {
    let deps = mock_dependencies();
    let env = mock_env();
    (deps, env)
}

/// 初始化合约
pub fn instantiate_contract(
    deps: &mut OwnedDeps<cosmwasm_std::MemoryStorage, cosmwasm_std::testing::MockApi, cosmwasm_std::testing::MockQuerier>,
    env: &cosmwasm_std::Env,
    scale: Scale,
    base_amount: u128,
) -> Result<cosmwasm_std::Response, dd_blind_box::error::ContractError> {
    let info = MessageInfo {
        sender: cosmwasm_std::Addr::unchecked(OWNER),
        funds: vec![],
    };
    let msg = InstantiateMsg {
        scale,
        base: Coin {
            denom: BASE_DENOM.to_string(),
            amount: Uint128::from(base_amount),
        },
        first_prize_count: None,  // 使用规模默认值
    };
    instantiate(deps.as_mut(), env.clone(), info, msg)
}

/// 创建充值消息
pub fn create_deposit_msg(amount: u128) -> (ExecuteMsg, MessageInfo) {
    let msg = ExecuteMsg::Deposit {};
    let info = MessageInfo {
        sender: cosmwasm_std::Addr::unchecked(USER1),
        funds: coins(amount, BASE_DENOM),
    };
    (msg, info)
}

/// 创建投票承诺消息
pub fn create_commit_msg(commitment: String) -> (ExecuteMsg, MessageInfo) {
    let msg = ExecuteMsg::CommitVote { commitment };
    let info = MessageInfo {
        sender: cosmwasm_std::Addr::unchecked(USER1),
        funds: vec![],
    };
    (msg, info)
}

/// 创建投票揭示消息
pub fn create_reveal_msg(reveal: String, salt: String) -> (ExecuteMsg, MessageInfo) {
    let msg = ExecuteMsg::RevealVote { reveal, salt };
    let info = MessageInfo {
        sender: cosmwasm_std::Addr::unchecked(USER1),
        funds: vec![],
    };
    (msg, info)
}

/// 创建NFT转移消息
pub fn create_transfer_msg(recipient: String, token_id: u64) -> (ExecuteMsg, MessageInfo) {
    let msg = ExecuteMsg::TransferNft { recipient, token_id };
    let info = MessageInfo {
        sender: cosmwasm_std::Addr::unchecked(USER1),
        funds: vec![],
    };
    (msg, info)
}

/// 创建NFT授权消息
pub fn create_approve_msg(spender: String, token_id: u64) -> (ExecuteMsg, MessageInfo) {
    let msg = ExecuteMsg::Approve { spender, token_id };
    let info = MessageInfo {
        sender: cosmwasm_std::Addr::unchecked(USER1),
        funds: vec![],
    };
    (msg, info)
}

/// 创建全局授权消息
pub fn create_approve_all_msg(operator: String) -> (ExecuteMsg, MessageInfo) {
    let msg = ExecuteMsg::ApproveAll { operator };
    let info = MessageInfo {
        sender: cosmwasm_std::Addr::unchecked(USER1),
        funds: vec![],
    };
    (msg, info)
}

/// 创建结算消息
pub fn create_finalize_msg() -> (ExecuteMsg, MessageInfo) {
    let msg = ExecuteMsg::Finalize {};
    let info = MessageInfo {
        sender: cosmwasm_std::Addr::unchecked(OWNER),
        funds: vec![],
    };
    (msg, info)
}

/// 计算承诺哈希
pub fn calculate_commitment(addr: &str, reveal: &str, salt: &str) -> String {
    use sha2::{Digest, Sha256};
    let preimage = format!("{}|{}|{}", addr, reveal, salt);
    let hash = Sha256::digest(preimage.as_bytes());
    hex::encode(hash)
}

/// 创建设置投票状态消息
pub fn create_set_vote_state_msg(state: VoteState) -> (ExecuteMsg, MessageInfo) {
    let msg = ExecuteMsg::SetVoteState { state };
    let info = MessageInfo {
        sender: cosmwasm_std::Addr::unchecked(OWNER),
        funds: vec![],
    };
    (msg, info)
}

/// 创建设置暂停消息
pub fn create_set_paused_msg(paused: bool) -> (ExecuteMsg, MessageInfo) {
    let msg = ExecuteMsg::SetPaused { paused };
    let info = MessageInfo {
        sender: cosmwasm_std::Addr::unchecked(OWNER),
        funds: vec![],
    };
    (msg, info)
}

/// 创建设置基础币种消息
pub fn create_set_base_msg(base: Coin) -> (ExecuteMsg, MessageInfo) {
    let msg = ExecuteMsg::SetBase { base };
    let info = MessageInfo {
        sender: cosmwasm_std::Addr::unchecked(OWNER),
        funds: vec![],
    };
    (msg, info)
}


/// 查询配置
pub fn query_config(deps: &OwnedDeps<cosmwasm_std::MemoryStorage, cosmwasm_std::testing::MockApi, cosmwasm_std::testing::MockQuerier>) -> dd_blind_box::msg::ConfigResponse {
    let msg = QueryMsg::Config {};
    let res = query(deps.as_ref(), mock_env(), msg).unwrap();
    cosmwasm_std::from_json(res).unwrap()
}

/// 查询充值
pub fn query_deposit(deps: &OwnedDeps<cosmwasm_std::MemoryStorage, cosmwasm_std::testing::MockApi, cosmwasm_std::testing::MockQuerier>, address: &str) -> dd_blind_box::msg::DepositResponse {
    let msg = QueryMsg::DepositOf { address: address.to_string() };
    let res = query(deps.as_ref(), mock_env(), msg).unwrap();
    cosmwasm_std::from_json(res).unwrap()
}

/// 查询分层
pub fn query_tier(deps: &OwnedDeps<cosmwasm_std::MemoryStorage, cosmwasm_std::testing::MockApi, cosmwasm_std::testing::MockQuerier>, address: &str) -> dd_blind_box::msg::TierResponse {
    let msg = QueryMsg::TierOf { address: address.to_string() };
    let res = query(deps.as_ref(), mock_env(), msg).unwrap();
    cosmwasm_std::from_json(res).unwrap()
}

/// 测试专用的查询充值函数，绕过地址验证
pub fn query_deposit_test(deps: &OwnedDeps<cosmwasm_std::MemoryStorage, cosmwasm_std::testing::MockApi, cosmwasm_std::testing::MockQuerier>, address: &str) -> dd_blind_box::msg::DepositResponse {
    use dd_blind_box::state::DEPOSITS;
    use cosmwasm_std::Addr;
    use cosmwasm_std::Uint128;
    
    let addr = Addr::unchecked(address);
    let p = DEPOSITS.may_load(&deps.storage, addr).unwrap().unwrap_or(dd_blind_box::state::Payout { principal: Uint128::zero() });
    dd_blind_box::msg::DepositResponse { principal: p.principal.to_string() }
}

/// 测试专用的查询分层函数，绕过地址验证
pub fn query_tier_test(deps: &OwnedDeps<cosmwasm_std::MemoryStorage, cosmwasm_std::testing::MockApi, cosmwasm_std::testing::MockQuerier>, address: &str) -> dd_blind_box::msg::TierResponse {
    use dd_blind_box::state::TIERS;
    use cosmwasm_std::Addr;
    
    let addr = Addr::unchecked(address);
    let t = TIERS.may_load(&deps.storage, addr).unwrap().unwrap_or(0);
    dd_blind_box::msg::TierResponse { tier: t }
}

/// 查询NFT所有者
pub fn query_owner_of(deps: &OwnedDeps<cosmwasm_std::MemoryStorage, cosmwasm_std::testing::MockApi, cosmwasm_std::testing::MockQuerier>, token_id: u64) -> dd_blind_box::msg::OwnerOfResponse {
    let msg = QueryMsg::OwnerOf { token_id };
    let res = query(deps.as_ref(), mock_env(), msg).unwrap();
    cosmwasm_std::from_json(res).unwrap()
}

/// 查询分层列表
pub fn query_tier_list(
    deps: &OwnedDeps<cosmwasm_std::MemoryStorage, cosmwasm_std::testing::MockApi, cosmwasm_std::testing::MockQuerier>,
    tier: u8,
    start_after: Option<String>,
    limit: Option<u32>,
) -> dd_blind_box::msg::TierListResponse {
    let msg = QueryMsg::TierList { tier, start_after, limit };
    let res = query(deps.as_ref(), mock_env(), msg).unwrap();
    cosmwasm_std::from_json(res).unwrap()
}

/// 查询NFT信息
pub fn query_nft_info(deps: &OwnedDeps<cosmwasm_std::MemoryStorage, cosmwasm_std::testing::MockApi, cosmwasm_std::testing::MockQuerier>, token_id: u64) -> dd_blind_box::msg::NftInfoResponse {
    let msg = QueryMsg::NftInfo { token_id };
    let res = query(deps.as_ref(), mock_env(), msg).unwrap();
    cosmwasm_std::from_json(res).unwrap()
}

/// 查询授权
pub fn query_approval(deps: &OwnedDeps<cosmwasm_std::MemoryStorage, cosmwasm_std::testing::MockApi, cosmwasm_std::testing::MockQuerier>, token_id: u64) -> dd_blind_box::msg::ApprovalResponse {
    let msg = QueryMsg::Approval { token_id };
    let res = query(deps.as_ref(), mock_env(), msg).unwrap();
    cosmwasm_std::from_json(res).unwrap()
}

/// 查询全局授权
pub fn query_is_approved_for_all(
    deps: &OwnedDeps<cosmwasm_std::MemoryStorage, cosmwasm_std::testing::MockApi, cosmwasm_std::testing::MockQuerier>,
    owner: &str,
    operator: &str,
) -> dd_blind_box::msg::IsApprovedForAllResponse {
    let msg = QueryMsg::IsApprovedForAll {
        owner: owner.to_string(),
        operator: operator.to_string(),
    };
    let res = query(deps.as_ref(), mock_env(), msg).unwrap();
    cosmwasm_std::from_json(res).unwrap()
}

