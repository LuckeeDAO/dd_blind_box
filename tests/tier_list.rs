mod common;

use cosmwasm_std::{testing::mock_env, Addr, OwnedDeps};
use dd_blind_box::{
    contract::query,
    msg::QueryMsg,
    state::{Scale, VoteState},
};
use common::*;

// Test constants
const USER1: &str = "user1";
const USER2: &str = "user2";
const USER3: &str = "user3";
const BASE_DENOM: &str = "ujunox";
const BASE_AMOUNT: u128 = 100;

// Helper functions
fn setup_test_env() -> (cosmwasm_std::OwnedDeps<cosmwasm_std::MemoryStorage, cosmwasm_std::testing::MockApi, cosmwasm_std::testing::MockQuerier>, cosmwasm_std::Env) {
    let deps = cosmwasm_std::testing::mock_dependencies();
    let env = cosmwasm_std::testing::mock_env();
    (deps, env)
}

fn instantiate_contract(
    deps: &mut cosmwasm_std::OwnedDeps<cosmwasm_std::MemoryStorage, cosmwasm_std::testing::MockApi, cosmwasm_std::testing::MockQuerier>,
    env: &cosmwasm_std::Env,
    scale: dd_blind_box::state::Scale,
    base_amount: u128,
) -> Result<cosmwasm_std::Response, dd_blind_box::error::ContractError> {
    let info = cosmwasm_std::MessageInfo {
        sender: cosmwasm_std::Addr::unchecked(OWNER),
        funds: vec![],
    };
    let msg = dd_blind_box::msg::InstantiateMsg {
        scale,
        base: cosmwasm_std::Coin {
            denom: BASE_DENOM.to_string(),
            amount: cosmwasm_std::Uint128::from(base_amount),
        },
    };
    dd_blind_box::contract::instantiate(deps.as_mut(), env.clone(), info, msg)
}

// Helper functions

fn query_tier_list(
    deps: &cosmwasm_std::OwnedDeps<cosmwasm_std::MemoryStorage, cosmwasm_std::testing::MockApi, cosmwasm_std::testing::MockQuerier>,
    tier: u8,
    start_after: Option<String>,
    limit: Option<u32>,
) -> dd_blind_box::msg::TierListResponse {
    let msg = dd_blind_box::msg::QueryMsg::TierList { tier, start_after, limit };
    let res = dd_blind_box::contract::query(deps.as_ref(), cosmwasm_std::testing::mock_env(), msg).unwrap();
    cosmwasm_std::from_json(res).unwrap()
}

fn calculate_commitment(addr: &str, reveal: &str, salt: &str) -> String {
    use sha2::{Digest, Sha256};
    let preimage = format!("{}|{}|{}", addr, reveal, salt);
    let hash = Sha256::digest(preimage.as_bytes());
    hex::encode(hash)
}


#[test]
fn test_tier_list_empty() {
    let (mut deps, env) = setup_test_env();
    instantiate_contract(&mut deps, &env, Scale::Tiny, BASE_AMOUNT).unwrap();
    
    // 查询空的分层列表
    let tier_list = query_tier_list(&deps, 1, None, None);
    assert_eq!(tier_list.addresses.len(), 0);
    assert_eq!(tier_list.next_start_after, None);
}

#[test]
fn test_tier_list_with_limit() {
    let (mut deps, env) = setup_test_env();
    instantiate_contract(&mut deps, &env, Scale::Tiny, BASE_AMOUNT).unwrap();
    
    // 设置一些分层数据（模拟结算后的状态）
    setup_multiple_tier_data(&mut deps);
    
    // 查询限制为2的分层列表
    let tier_list = query_tier_list(&deps, 1, None, Some(2));
    assert_eq!(tier_list.addresses.len(), 2);
    assert!(tier_list.next_start_after.is_some());
}

#[test]
fn test_tier_list_with_start_after() {
    let (mut deps, env) = setup_test_env();
    instantiate_contract(&mut deps, &env, Scale::Tiny, BASE_AMOUNT).unwrap();
    
    // 设置一些分层数据
    setup_multiple_tier_data(&mut deps);
    
    // 从特定地址开始查询
    let tier_list = query_tier_list(&deps, 1, Some(USER1.to_string()), None);
    assert!(tier_list.addresses.len() > 0);
    
    // 验证返回的地址不包含起始地址
    assert!(!tier_list.addresses.contains(&USER1.to_string()));
}

#[test]
fn test_tier_list_different_tiers() {
    let (mut deps, env) = setup_test_env();
    instantiate_contract(&mut deps, &env, Scale::Tiny, BASE_AMOUNT).unwrap();
    
    // 设置不同分层的数据
    setup_tier_data(&mut deps);
    
    // 查询分层1
    let tier1_list = query_tier_list(&deps, 1, None, None);
    assert_eq!(tier1_list.addresses.len(), 1);
    assert_eq!(tier1_list.addresses[0], USER1);
    
    // 查询分层2
    let tier2_list = query_tier_list(&deps, 2, None, None);
    assert_eq!(tier2_list.addresses.len(), 1);
    assert_eq!(tier2_list.addresses[0], USER2);
    
    // 查询分层3
    let tier3_list = query_tier_list(&deps, 3, None, None);
    assert_eq!(tier3_list.addresses.len(), 1);
    assert_eq!(tier3_list.addresses[0], USER3);
}

#[test]
fn test_tier_list_pagination() {
    let (mut deps, env) = setup_test_env();
    instantiate_contract(&mut deps, &env, Scale::Tiny, BASE_AMOUNT).unwrap();
    
    // 设置多个相同分层的数据
    setup_multiple_tier_data(&mut deps);
    
    // 第一页
    let tier_list1 = query_tier_list(&deps, 1, None, Some(2));
    assert_eq!(tier_list1.addresses.len(), 2);
    assert!(tier_list1.next_start_after.is_some());
    
    // 第二页
    let tier_list2 = query_tier_list(&deps, 1, tier_list1.next_start_after, Some(2));
    assert_eq!(tier_list2.addresses.len(), 0); // 没有更多地址了
    assert_eq!(tier_list2.next_start_after, None);
}

#[test]
fn test_tier_list_nonexistent_tier() {
    let (mut deps, env) = setup_test_env();
    instantiate_contract(&mut deps, &env, Scale::Tiny, BASE_AMOUNT).unwrap();
    
    // 查询不存在的分层
    let tier_list = query_tier_list(&deps, 99, None, None);
    assert_eq!(tier_list.addresses.len(), 0);
    assert_eq!(tier_list.next_start_after, None);
}

#[test]
fn test_tier_list_invalid_start_after() {
    let (mut deps, env) = setup_test_env();
    instantiate_contract(&mut deps, &env, Scale::Tiny, BASE_AMOUNT).unwrap();
    
    // 使用无效的起始地址
    let result = query(deps.as_ref(), mock_env(), QueryMsg::TierList {
        tier: 1,
        start_after: Some("invalid_address".to_string()),
        limit: None,
    });
    assert!(result.is_err());
}

#[test]
fn test_tier_list_large_limit() {
    let (mut deps, env) = setup_test_env();
    instantiate_contract(&mut deps, &env, Scale::Tiny, BASE_AMOUNT).unwrap();
    
    // 设置一些分层数据
    setup_tier_data(&mut deps);
    
    // 使用很大的限制值
    let tier_list = query_tier_list(&deps, 1, None, Some(1000));
    assert_eq!(tier_list.addresses.len(), 1); // 只有1个数据
    assert_eq!(tier_list.next_start_after, None);
}

#[test]
fn test_tier_list_zero_limit() {
    let (mut deps, env) = setup_test_env();
    instantiate_contract(&mut deps, &env, Scale::Tiny, BASE_AMOUNT).unwrap();
    
    // 设置一些分层数据
    setup_tier_data(&mut deps);
    
    // 使用零限制
    let tier_list = query_tier_list(&deps, 1, None, Some(0));
    assert_eq!(tier_list.addresses.len(), 0);
    assert!(tier_list.next_start_after.is_some());
}

#[test]
fn test_tier_list_default_limit() {
    let (mut deps, env) = setup_test_env();
    instantiate_contract(&mut deps, &env, Scale::Tiny, BASE_AMOUNT).unwrap();
    
    // 设置一些分层数据
    setup_tier_data(&mut deps);
    
    // 不指定限制（应该使用默认值50）
    let tier_list = query_tier_list(&deps, 1, None, None);
    assert_eq!(tier_list.addresses.len(), 1);
    assert_eq!(tier_list.next_start_after, None);
}

#[test]
fn test_tier_list_ordering() {
    let (mut deps, env) = setup_test_env();
    instantiate_contract(&mut deps, &env, Scale::Tiny, BASE_AMOUNT).unwrap();
    
    // 设置多个分层数据
    setup_multiple_tier_data(&mut deps);
    
    // 查询分层1，验证排序
    let tier_list = query_tier_list(&deps, 1, None, None);
    assert_eq!(tier_list.addresses.len(), 3);
    
    // 验证地址按升序排列
    let mut sorted_addresses = tier_list.addresses.clone();
    sorted_addresses.sort();
    assert_eq!(tier_list.addresses, sorted_addresses);
}

#[test]
fn test_tier_list_after_finalize() {
    let (mut deps, env) = setup_test_env();
    instantiate_contract(&mut deps, &env, Scale::Tiny, BASE_AMOUNT).unwrap();
    
    // 模拟完整的投票和结算流程
    setup_voting_and_finalize(&mut deps, &env);
    
    // 查询各分层的地址列表
    let tier1_list = query_tier_list(&deps, 1, None, None);
    let tier2_list = query_tier_list(&deps, 2, None, None);
    let tier3_list = query_tier_list(&deps, 3, None, None);
    
    // 验证至少有一些地址被分配到各分层
    assert!(tier1_list.addresses.len() > 0 || tier2_list.addresses.len() > 0 || tier3_list.addresses.len() > 0);
}

// 辅助函数：设置分层数据
fn setup_tier_data(deps: &mut OwnedDeps<cosmwasm_std::MemoryStorage, cosmwasm_std::testing::MockApi, cosmwasm_std::testing::MockQuerier>) {
    use dd_blind_box::state::TIERS;
    
    // 设置不同用户的分层
    TIERS.save(&mut deps.storage, Addr::unchecked(USER1), &1).unwrap();
    TIERS.save(&mut deps.storage, Addr::unchecked(USER2), &2).unwrap();
    TIERS.save(&mut deps.storage, Addr::unchecked(USER3), &3).unwrap();
}

// 辅助函数：设置多个相同分层的数据
fn setup_multiple_tier_data(deps: &mut OwnedDeps<cosmwasm_std::MemoryStorage, cosmwasm_std::testing::MockApi, cosmwasm_std::testing::MockQuerier>) {
    use dd_blind_box::state::TIERS;
    
    // 设置多个用户为分层1
    TIERS.save(&mut deps.storage, Addr::unchecked(USER1), &1).unwrap();
    TIERS.save(&mut deps.storage, Addr::unchecked(USER2), &1).unwrap();
    TIERS.save(&mut deps.storage, Addr::unchecked(USER3), &1).unwrap();
}

// 辅助函数：设置完整的投票和结算流程
fn setup_voting_and_finalize(deps: &mut OwnedDeps<cosmwasm_std::MemoryStorage, cosmwasm_std::testing::MockApi, cosmwasm_std::testing::MockQuerier>, _env: &cosmwasm_std::Env) {
    use dd_blind_box::state::{COMMITS, REVEALS, DEPOSITS};
    use cosmwasm_std::Uint128;
    
    // 设置充值记录
    DEPOSITS.save(&mut deps.storage, Addr::unchecked(USER1), &dd_blind_box::state::Payout {
        principal: Uint128::from(BASE_AMOUNT),
    }).unwrap();
    
    DEPOSITS.save(&mut deps.storage, Addr::unchecked(USER2), &dd_blind_box::state::Payout {
        principal: Uint128::from(BASE_AMOUNT),
    }).unwrap();
    
    DEPOSITS.save(&mut deps.storage, Addr::unchecked(USER3), &dd_blind_box::state::Payout {
        principal: Uint128::from(BASE_AMOUNT),
    }).unwrap();
    
    // 设置投票承诺和揭示
    let users = vec![USER1, USER2, USER3];
    for (i, user) in users.iter().enumerate() {
        let reveal = format!("vote_{}", i);
        let salt = format!("salt_{}", i);
        let commitment = calculate_commitment(user, &reveal, &salt);
        
        COMMITS.save(&mut deps.storage, Addr::unchecked(*user), &dd_blind_box::state::CommitInfo {
            commitment,
        }).unwrap();
        
        REVEALS.save(&mut deps.storage, Addr::unchecked(*user), &dd_blind_box::state::RevealInfo {
            reveal,
            salt,
        }).unwrap();
    }
    
    // 设置关闭窗口
    let mut config = dd_blind_box::state::CONFIG.load(&deps.storage).unwrap();
    config.vote_state = VoteState::Closed;
    config.closed_window.start_time = None;
    config.closed_window.end_time = None;
    dd_blind_box::state::CONFIG.save(&mut deps.storage, &config).unwrap();
    
    // 执行结算
    let (msg, info) = create_finalize_msg();
    let env = cosmwasm_std::testing::mock_env();
    dd_blind_box::contract::execute(deps.as_mut(), env, info, msg).unwrap();
}
