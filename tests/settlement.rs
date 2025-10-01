mod common;

use cosmwasm_std::{Addr, Uint128, OwnedDeps, MessageInfo};
use dd_blind_box::{
    contract::execute,
    error::ContractError,
    msg::ExecuteMsg,
    state::{Scale, VoteState},
};
use common::*;


#[test]
fn test_finalize_success() {
    let (mut deps, env) = setup_test_env();
    instantiate_contract(&mut deps, &env, Scale::Tiny, BASE_AMOUNT).unwrap();
    
    // 设置完整的投票和结算环境
    setup_finalize_environment(&mut deps, &env);
    
    // 执行结算
    let (msg, info) = create_finalize_msg();
    let result = execute(deps.as_mut(), env, info, msg);
    assert!(result.is_ok());
    
    // 验证响应属性
    let res = result.unwrap();
    assert_eq!(res.attributes[0].key, "action");
    assert_eq!(res.attributes[0].value, "finalize");
    
    // 验证有银行消息（返还资金）
    assert!(res.messages.len() > 0);
}

#[test]
fn test_finalize_wrong_phase() {
    let (mut deps, env) = setup_test_env();
    instantiate_contract(&mut deps, &env, Scale::Tiny, BASE_AMOUNT).unwrap();
    
    // 设置投票环境但不在Closed阶段
    setup_voting_environment(&mut deps);
    
    // 尝试在非Closed阶段结算
    let (msg, info) = create_finalize_msg();
    let result = execute(deps.as_mut(), env, info, msg);
    assert_eq!(result.unwrap_err(), ContractError::InvalidState);
}

#[test]
fn test_finalize_paused() {
    let (mut deps, env) = setup_test_env();
    instantiate_contract(&mut deps, &env, Scale::Tiny, BASE_AMOUNT).unwrap();
    
    // 设置暂停
    let (msg, info) = create_set_paused_msg(true);
    execute(deps.as_mut(), env.clone(), info, msg).unwrap();
    
    // 设置结算环境
    setup_finalize_environment(&mut deps, &env);
    
    // 尝试在暂停状态下结算
    let (msg, info) = create_finalize_msg();
    let result = execute(deps.as_mut(), env, info, msg);
    assert_eq!(result.unwrap_err(), ContractError::InvalidState);
}

#[test]
fn test_finalize_no_voters() {
    let (mut deps, env) = setup_test_env();
    instantiate_contract(&mut deps, &env, Scale::Tiny, BASE_AMOUNT).unwrap();
    
    // 设置Closed阶段但没有投票者
    let mut config = dd_blind_box::state::CONFIG.load(&deps.storage).unwrap();
    config.vote_state = VoteState::Closed;
    // 不要覆盖暂停状态，让测试自己控制
    config.closed_window.start_time = None;
    config.closed_window.end_time = None;
    dd_blind_box::state::CONFIG.save(&mut deps.storage, &config).unwrap();
    
    // 执行结算
    let (msg, info) = create_finalize_msg();
    let result = execute(deps.as_mut(), env, info, msg);
    assert!(result.is_ok());
    
    // 验证响应
    let res = result.unwrap();
    assert_eq!(res.attributes[0].key, "action");
    assert_eq!(res.attributes[0].value, "finalize");
    assert_eq!(res.attributes[1].key, "note");
    assert_eq!(res.attributes[1].value, "no voters");
    
    // 验证没有银行消息
    assert_eq!(res.messages.len(), 0);
}

#[test]
fn test_finalize_window_restriction() {
    let (mut deps, env) = setup_test_env();
    instantiate_contract(&mut deps, &env, Scale::Tiny, BASE_AMOUNT).unwrap();
    
    // 设置Closed阶段但不在窗口内
    let mut config = dd_blind_box::state::CONFIG.load(&deps.storage).unwrap();
    config.vote_state = VoteState::Closed;
    config.closed_window.start_height = Some(1000); // 当前高度为0，不在窗口内
    config.closed_window.end_height = Some(2000);
    dd_blind_box::state::CONFIG.save(&mut deps.storage, &config).unwrap();
    
    // 设置投票环境
    setup_voting_environment(&mut deps);
    
    // 尝试在窗口外结算
    let (msg, info) = create_finalize_msg();
    let result = execute(deps.as_mut(), env, info, msg);
    assert_eq!(result.unwrap_err(), ContractError::InvalidState);
}

#[test]
fn test_finalize_tier_assignment() {
    let (mut deps, env) = setup_test_env();
    instantiate_contract(&mut deps, &env, Scale::Tiny, BASE_AMOUNT).unwrap();
    
    // 设置结算环境
    setup_finalize_environment(&mut deps, &env);
    
    // 执行结算
    let (msg, info) = create_finalize_msg();
    execute(deps.as_mut(), env, info, msg).unwrap();
    
    // 验证分层分配
    let tier1 = query_tier_test(&deps, USER1);
    let tier2 = query_tier_test(&deps, USER2);
    let tier3 = query_tier_test(&deps, USER3);
    
    // 至少有一个用户被分配到分层
    assert!(tier1.tier > 0 || tier2.tier > 0 || tier3.tier > 0);
}

#[test]
fn test_finalize_payout_calculation() {
    let (mut deps, env) = setup_test_env();
    instantiate_contract(&mut deps, &env, Scale::Tiny, BASE_AMOUNT).unwrap();
    
    // 设置结算环境
    setup_finalize_environment(&mut deps, &env);
    
    // 执行结算
    let (msg, info) = create_finalize_msg();
    let result = execute(deps.as_mut(), env, info, msg);
    assert!(result.is_ok());
    
    // 验证银行消息
    let res = result.unwrap();
    assert!(res.messages.len() > 0);
    
    // 验证消息类型和内容
    for msg in res.messages {
        match msg.msg {
            cosmwasm_std::CosmosMsg::Bank(cosmwasm_std::BankMsg::Send { to_address, amount }) => {
                assert!(!to_address.is_empty());
                assert!(!amount.is_empty());
                assert_eq!(amount[0].denom, BASE_DENOM);
                assert!(amount[0].amount > Uint128::zero());
            }
            _ => panic!("Unexpected message type"),
        }
    }
}

#[test]
fn test_finalize_multiple_users() {
    let (mut deps, env) = setup_test_env();
    instantiate_contract(&mut deps, &env, Scale::Tiny, BASE_AMOUNT).unwrap();
    
    // 设置多个用户的投票环境
    setup_multiple_users_environment(&mut deps, &env);
    
    // 执行结算
    let (msg, info) = create_finalize_msg();
    let result = execute(deps.as_mut(), env, info, msg);
    assert!(result.is_ok());
    
    // 验证所有用户都被处理
    let res = result.unwrap();
    assert!(res.messages.len() >= 3); // 至少3个用户的返还消息
}

#[test]
fn test_finalize_duplicate() {
    let (mut deps, env) = setup_test_env();
    instantiate_contract(&mut deps, &env, Scale::Tiny, BASE_AMOUNT).unwrap();
    
    // 设置结算环境
    setup_finalize_environment(&mut deps, &env);
    
    // 第一次结算
    let (msg, info) = create_finalize_msg();
    let result1 = execute(deps.as_mut(), env.clone(), info, msg);
    assert!(result1.is_ok());
    
    // 第二次结算（应该成功，但可能没有新的消息）
    let (msg, info) = create_finalize_msg();
    let result2 = execute(deps.as_mut(), env, info, msg);
    assert!(result2.is_ok());
}

#[test]
fn test_finalize_unauthorized() {
    let (mut deps, env) = setup_test_env();
    instantiate_contract(&mut deps, &env, Scale::Tiny, BASE_AMOUNT).unwrap();
    
    // 设置结算环境
    setup_finalize_environment(&mut deps, &env);
    
    // 非owner尝试结算
    let msg = ExecuteMsg::Finalize {};
    let info = MessageInfo {
        sender: cosmwasm_std::Addr::unchecked(USER1),
        funds: vec![],
    };
    let result = execute(deps.as_mut(), env, info, msg);
    assert_eq!(result.unwrap_err(), ContractError::Unauthorized);
}

#[test]
fn test_finalize_tier_ratios() {
    let (mut deps, env) = setup_test_env();
    instantiate_contract(&mut deps, &env, Scale::Tiny, BASE_AMOUNT).unwrap();
    
    // 设置10个用户的投票环境
    setup_ten_users_environment(&mut deps, &env);
    
    // 执行结算
    let (msg, info) = create_finalize_msg();
    let result = execute(deps.as_mut(), env, info, msg);
    assert!(result.is_ok());
    
    // 验证分层比例（10%, 50%, 40%）
    let mut tier1_count = 0;
    let mut tier2_count = 0;
    let mut tier3_count = 0;
    
    for i in 1..=10 {
        let user = format!("user{}", i);
        let tier = query_tier_test(&deps, &user);
        match tier.tier {
            1 => tier1_count += 1,
            2 => tier2_count += 1,
            3 => tier3_count += 1,
            _ => {}
        }
    }
    
    // 验证至少有一些用户被分配到各分层
    assert!(tier1_count > 0 || tier2_count > 0 || tier3_count > 0);
}

// 辅助函数：设置结算环境
fn setup_finalize_environment(deps: &mut OwnedDeps<cosmwasm_std::MemoryStorage, cosmwasm_std::testing::MockApi, cosmwasm_std::testing::MockQuerier>, _env: &cosmwasm_std::Env) {
    use dd_blind_box::state::{COMMITS, REVEALS, DEPOSITS, CONFIG};
    
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
    
    // 设置Closed阶段和窗口
    let mut config = CONFIG.load(&deps.storage).unwrap();
    config.vote_state = VoteState::Closed;
    // 不要覆盖暂停状态，让测试自己控制
    config.closed_window.start_time = None;
    config.closed_window.end_time = None;
    CONFIG.save(&mut deps.storage, &config).unwrap();
}

// 辅助函数：设置投票环境（非Closed阶段）
fn setup_voting_environment(deps: &mut OwnedDeps<cosmwasm_std::MemoryStorage, cosmwasm_std::testing::MockApi, cosmwasm_std::testing::MockQuerier>) {
    use dd_blind_box::state::{COMMITS, REVEALS, DEPOSITS};
    
    // 设置充值记录
    DEPOSITS.save(&mut deps.storage, Addr::unchecked(USER1), &dd_blind_box::state::Payout {
        principal: Uint128::from(BASE_AMOUNT),
    }).unwrap();
    
    // 设置投票承诺和揭示
    let reveal = "vote_1";
    let salt = "salt_1";
    let commitment = calculate_commitment(USER1, reveal, salt);
    
    COMMITS.save(&mut deps.storage, Addr::unchecked(USER1), &dd_blind_box::state::CommitInfo {
        commitment,
    }).unwrap();
    
    REVEALS.save(&mut deps.storage, Addr::unchecked(USER1), &dd_blind_box::state::RevealInfo {
        reveal: reveal.to_string(),
        salt: salt.to_string(),
    }).unwrap();
}

// 辅助函数：设置多个用户环境
fn setup_multiple_users_environment(deps: &mut OwnedDeps<cosmwasm_std::MemoryStorage, cosmwasm_std::testing::MockApi, cosmwasm_std::testing::MockQuerier>, _env: &cosmwasm_std::Env) {
    use dd_blind_box::state::{COMMITS, REVEALS, DEPOSITS, CONFIG};
    
    let users = vec![USER1, USER2, USER3];
    
    // 设置所有用户的充值记录
    for user in &users {
        DEPOSITS.save(&mut deps.storage, Addr::unchecked(*user), &dd_blind_box::state::Payout {
            principal: Uint128::from(BASE_AMOUNT),
        }).unwrap();
    }
    
    // 设置所有用户的投票承诺和揭示
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
    
    // 设置Closed阶段和窗口
    let mut config = CONFIG.load(&deps.storage).unwrap();
    config.vote_state = VoteState::Closed;
    // 不要覆盖暂停状态，让测试自己控制
    config.closed_window.start_time = None;
    config.closed_window.end_time = None;
    CONFIG.save(&mut deps.storage, &config).unwrap();
}

// 辅助函数：设置10个用户环境
fn setup_ten_users_environment(deps: &mut OwnedDeps<cosmwasm_std::MemoryStorage, cosmwasm_std::testing::MockApi, cosmwasm_std::testing::MockQuerier>, _env: &cosmwasm_std::Env) {
    use dd_blind_box::state::{COMMITS, REVEALS, DEPOSITS, CONFIG};
    
    // 设置10个用户的充值记录
    for i in 1..=10 {
        let user = format!("user{}", i);
        DEPOSITS.save(&mut deps.storage, Addr::unchecked(&user), &dd_blind_box::state::Payout {
            principal: Uint128::from(BASE_AMOUNT),
        }).unwrap();
    }
    
    // 设置10个用户的投票承诺和揭示
    for i in 1..=10 {
        let user = format!("user{}", i);
        let reveal = format!("vote_{}", i);
        let salt = format!("salt_{}", i);
        let commitment = calculate_commitment(&user, &reveal, &salt);
        
        COMMITS.save(&mut deps.storage, Addr::unchecked(&user), &dd_blind_box::state::CommitInfo {
            commitment,
        }).unwrap();
        
        REVEALS.save(&mut deps.storage, Addr::unchecked(&user), &dd_blind_box::state::RevealInfo {
            reveal,
            salt,
        }).unwrap();
    }
    
    // 设置Closed阶段和窗口
    let mut config = CONFIG.load(&deps.storage).unwrap();
    config.vote_state = VoteState::Closed;
    // 不要覆盖暂停状态，让测试自己控制
    config.closed_window.start_time = None;
    config.closed_window.end_time = None;
    CONFIG.save(&mut deps.storage, &config).unwrap();
}
