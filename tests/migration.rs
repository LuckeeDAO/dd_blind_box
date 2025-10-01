mod common;

use cosmwasm_std::OwnedDeps;
use dd_blind_box::{
    contract::{execute, migrate},
    msg::MigrateMsg,
    state::Scale,
};
use common::*;

// Test constants
const OWNER: &str = "owner";
const USER1: &str = "user1";
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
fn create_set_vote_state_msg(state: dd_blind_box::state::VoteState) -> (dd_blind_box::msg::ExecuteMsg, cosmwasm_std::MessageInfo) {
    let msg = dd_blind_box::msg::ExecuteMsg::SetVoteState { state };
    let info = cosmwasm_std::MessageInfo {
        sender: cosmwasm_std::Addr::unchecked(OWNER),
        funds: vec![],
    };
    (msg, info)
}


fn query_config(deps: &cosmwasm_std::OwnedDeps<cosmwasm_std::MemoryStorage, cosmwasm_std::testing::MockApi, cosmwasm_std::testing::MockQuerier>) -> dd_blind_box::msg::ConfigResponse {
    let msg = dd_blind_box::msg::QueryMsg::Config {};
    let res = dd_blind_box::contract::query(deps.as_ref(), cosmwasm_std::testing::mock_env(), msg).unwrap();
    cosmwasm_std::from_json(res).unwrap()
}


fn create_set_paused_msg(paused: bool) -> (dd_blind_box::msg::ExecuteMsg, cosmwasm_std::MessageInfo) {
    let msg = dd_blind_box::msg::ExecuteMsg::SetPaused { paused };
    let info = cosmwasm_std::MessageInfo {
        sender: cosmwasm_std::Addr::unchecked(OWNER),
        funds: vec![],
    };
    (msg, info)
}

#[test]
fn test_migrate_success() {
    let (mut deps, env) = setup_test_env();
    
    // 初始化为Tiny规模
    instantiate_contract(&mut deps, &env, Scale::Tiny, BASE_AMOUNT).unwrap();
    
    // 验证初始配置
    let config_before = query_config(&deps);
    assert_eq!(config_before.scale, Scale::Tiny);
    assert_eq!(config_before.total_supply, 10);
    
    // 迁移到Small规模
    let msg = MigrateMsg { scale: Scale::Small };
    let result = migrate(deps.as_mut(), env, msg);
    assert!(result.is_ok());
    
    // 验证迁移后的配置
    let config_after = query_config(&deps);
    assert_eq!(config_after.scale, Scale::Small);
    assert_eq!(config_after.total_supply, 100);
    
    // 验证其他字段保持不变
    assert_eq!(config_after.owner, config_before.owner);
    assert_eq!(config_after.base, config_before.base);
    assert_eq!(config_after.vote_state, config_before.vote_state);
}

#[test]
fn test_migrate_all_scales() {
    let scales = vec![
        (Scale::Tiny, 10),
        (Scale::Small, 100),
        (Scale::Medium, 1_000),
        (Scale::Large, 10_000),
        (Scale::Huge, 100_000),
    ];
    
    for (scale, expected_supply) in scales {
        let (mut deps, env) = setup_test_env();
        
        // 初始化为Tiny规模
        instantiate_contract(&mut deps, &env, Scale::Tiny, BASE_AMOUNT).unwrap();
        
        // 迁移到目标规模
        let msg = MigrateMsg { scale: scale.clone() };
        let result = migrate(deps.as_mut(), env, msg);
        assert!(result.is_ok());
        
        // 验证迁移结果
        let config = query_config(&deps);
        assert_eq!(config.scale, scale);
        assert_eq!(config.total_supply, expected_supply);
    }
}

#[test]
fn test_migrate_same_scale() {
    let (mut deps, env) = setup_test_env();
    
    // 初始化为Medium规模
    instantiate_contract(&mut deps, &env, Scale::Medium, BASE_AMOUNT).unwrap();
    
    // 迁移到相同的Medium规模
    let msg = MigrateMsg { scale: Scale::Medium };
    let result = migrate(deps.as_mut(), env, msg);
    assert!(result.is_ok());
    
    // 验证配置没有变化
    let config = query_config(&deps);
    assert_eq!(config.scale, Scale::Medium);
    assert_eq!(config.total_supply, 1_000);
}

#[test]
fn test_migrate_preserves_state() {
    let (mut deps, env) = setup_test_env();
    
    // 初始化为Tiny规模
    instantiate_contract(&mut deps, &env, Scale::Tiny, BASE_AMOUNT).unwrap();
    
    // 设置一些状态
    let (msg, info) = create_set_paused_msg(true);
    execute(deps.as_mut(), env.clone(), info, msg).unwrap();
    
    let (msg, info) = create_set_vote_state_msg(dd_blind_box::state::VoteState::Reveal);
    execute(deps.as_mut(), env.clone(), info, msg).unwrap();
    
    // 取消暂停以便充值
    let (msg, info) = create_set_paused_msg(false);
    execute(deps.as_mut(), env.clone(), info, msg).unwrap();
    
    // 充值一些NFT
    let (msg, info) = create_deposit_msg(BASE_AMOUNT * 2);
    execute(deps.as_mut(), env.clone(), info, msg).unwrap();
    
    // 记录迁移前的状态
    let config_before = query_config(&deps);
    let deposit_before = query_deposit_test(&deps, USER1);
    
    // 迁移到Large规模
    let msg = MigrateMsg { scale: Scale::Large };
    let result = migrate(deps.as_mut(), env, msg);
    assert!(result.is_ok());
    
    // 验证状态被保留
    let config_after = query_config(&deps);
    // paused状态不在ConfigResponse中
    assert_eq!(config_after.vote_state, config_before.vote_state);
    // next_token_id不在ConfigResponse中
    
    let deposit_after = query_deposit_test(&deps, USER1);
    assert_eq!(deposit_after.principal, deposit_before.principal);
    
    // 验证NFT仍然存在
    let owner0 = query_owner_of(&deps, 0);
    assert_eq!(owner0.owner, USER1);
    
    let owner1 = query_owner_of(&deps, 1);
    assert_eq!(owner1.owner, USER1);
}

#[test]
fn test_migrate_scale_up() {
    let (mut deps, env) = setup_test_env();
    
    // 初始化为Small规模
    instantiate_contract(&mut deps, &env, Scale::Small, BASE_AMOUNT).unwrap();
    
    // 迁移到Huge规模
    let msg = MigrateMsg { scale: Scale::Huge };
    let result = migrate(deps.as_mut(), env, msg);
    assert!(result.is_ok());
    
    // 验证规模升级
    let config = query_config(&deps);
    assert_eq!(config.scale, Scale::Huge);
    assert_eq!(config.total_supply, 100_000);
}

#[test]
fn test_migrate_scale_down() {
    let (mut deps, env) = setup_test_env();
    
    // 初始化为Large规模
    instantiate_contract(&mut deps, &env, Scale::Large, BASE_AMOUNT).unwrap();
    
    // 迁移到Medium规模
    let msg = MigrateMsg { scale: Scale::Medium };
    let result = migrate(deps.as_mut(), env, msg);
    assert!(result.is_ok());
    
    // 验证规模降级
    let config = query_config(&deps);
    assert_eq!(config.scale, Scale::Medium);
    assert_eq!(config.total_supply, 1_000);
}

#[test]
fn test_migrate_with_existing_nfts() {
    let (mut deps, env) = setup_test_env();
    
    // 初始化为Tiny规模
    instantiate_contract(&mut deps, &env, Scale::Tiny, BASE_AMOUNT).unwrap();
    
    // 铸造一些NFT
    let (msg, info) = create_deposit_msg(BASE_AMOUNT * 5);
    execute(deps.as_mut(), env.clone(), info, msg).unwrap();
    
    // 验证NFT存在
    for i in 0..5 {
        let owner = query_owner_of(&deps, i);
        assert_eq!(owner.owner, USER1);
    }
    
    // 迁移到Medium规模
    let msg = MigrateMsg { scale: Scale::Medium };
    let result = migrate(deps.as_mut(), env.clone(), msg);
    assert!(result.is_ok());
    
    // 验证NFT仍然存在
    for i in 0..5 {
        let owner = query_owner_of(&deps, i);
        assert_eq!(owner.owner, USER1);
    }
    
    // 验证可以继续铸造NFT
    let (msg, info) = create_deposit_msg(BASE_AMOUNT);
    let result = execute(deps.as_mut(), env, info, msg);
    assert!(result.is_ok());
    
    // 验证新NFT被铸造
    let owner = query_owner_of(&deps, 5);
    assert_eq!(owner.owner, USER1);
}

#[test]
fn test_migrate_with_voting_data() {
    let (mut deps, env) = setup_test_env();
    
    // 初始化为Tiny规模
    instantiate_contract(&mut deps, &env, Scale::Tiny, BASE_AMOUNT).unwrap();
    
    // 设置投票数据
    setup_voting_data(&mut deps);
    
    // 迁移到Large规模
    let msg = MigrateMsg { scale: Scale::Large };
    let result = migrate(deps.as_mut(), env, msg);
    assert!(result.is_ok());
    
    // 验证投票数据被保留
    let config = query_config(&deps);
    assert_eq!(config.scale, Scale::Large);
    assert_eq!(config.total_supply, 10_000);
    
    // 验证投票状态被保留
    assert_eq!(config.vote_state, dd_blind_box::state::VoteState::Reveal);
}

#[test]
fn test_migrate_response_attributes() {
    let (mut deps, env) = setup_test_env();
    
    // 初始化为Tiny规模
    instantiate_contract(&mut deps, &env, Scale::Tiny, BASE_AMOUNT).unwrap();
    
    // 迁移到Small规模
    let msg = MigrateMsg { scale: Scale::Small };
    let result = migrate(deps.as_mut(), env, msg);
    assert!(result.is_ok());
    
    // 验证响应属性
    let res = result.unwrap();
    assert_eq!(res.attributes[0].key, "action");
    assert_eq!(res.attributes[0].value, "migrate");
    assert_eq!(res.attributes[1].key, "scale");
    assert_eq!(res.attributes[1].value, "small");
}

#[test]
fn test_migrate_multiple_times() {
    let (mut deps, env) = setup_test_env();
    
    // 初始化为Tiny规模
    instantiate_contract(&mut deps, &env, Scale::Tiny, BASE_AMOUNT).unwrap();
    
    // 第一次迁移
    let msg = MigrateMsg { scale: Scale::Small };
    let result = migrate(deps.as_mut(), env.clone(), msg);
    assert!(result.is_ok());
    
    let config = query_config(&deps);
    assert_eq!(config.scale, Scale::Small);
    assert_eq!(config.total_supply, 100);
    
    // 第二次迁移
    let msg = MigrateMsg { scale: Scale::Medium };
    let result = migrate(deps.as_mut(), env.clone(), msg);
    assert!(result.is_ok());
    
    let config = query_config(&deps);
    assert_eq!(config.scale, Scale::Medium);
    assert_eq!(config.total_supply, 1_000);
    
    // 第三次迁移
    let msg = MigrateMsg { scale: Scale::Large };
    let result = migrate(deps.as_mut(), env, msg);
    assert!(result.is_ok());
    
    let config = query_config(&deps);
    assert_eq!(config.scale, Scale::Large);
    assert_eq!(config.total_supply, 10_000);
}

// 辅助函数：设置投票数据
fn setup_voting_data(deps: &mut OwnedDeps<cosmwasm_std::MemoryStorage, cosmwasm_std::testing::MockApi, cosmwasm_std::testing::MockQuerier>) {
    use dd_blind_box::state::{COMMITS, REVEALS, CONFIG};
    
    // 设置投票状态
    let mut config = CONFIG.load(&deps.storage).unwrap();
    config.vote_state = dd_blind_box::state::VoteState::Reveal;
    CONFIG.save(&mut deps.storage, &config).unwrap();
    
    // 设置投票承诺
    COMMITS.save(&mut deps.storage, cosmwasm_std::Addr::unchecked(USER1), &dd_blind_box::state::CommitInfo {
        commitment: "test_commitment".to_string(),
    }).unwrap();
    
    // 设置投票揭示
    REVEALS.save(&mut deps.storage, cosmwasm_std::Addr::unchecked(USER1), &dd_blind_box::state::RevealInfo {
        reveal: "test_reveal".to_string(),
        salt: "test_salt".to_string(),
    }).unwrap();
}
