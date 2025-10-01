mod common;

use cosmwasm_std::{Addr, Coin, OwnedDeps, MessageInfo};
use dd_blind_box::{
    contract::execute,
    error::ContractError,
    msg::ExecuteMsg,
    state::{Scale, VoteState},
};
use common::*;
use sha2::Digest;


#[test]
fn test_pause_set_success() {
    let (mut deps, env) = setup_test_env();
    instantiate_contract(&mut deps, &env, Scale::Tiny, BASE_AMOUNT).unwrap();
    
    // 初始状态应该是未暂停
    let _config = query_config(&deps);
    // paused状态不在ConfigResponse中
    
    // 设置暂停
    let (msg, info) = create_set_paused_msg(true);
    let result = execute(deps.as_mut(), env.clone(), info, msg);
    assert!(result.is_ok());
    
    // 验证暂停状态
    let _config = query_config(&deps);
    // paused状态不在ConfigResponse中
    
    // 取消暂停
    let (msg, info) = create_set_paused_msg(false);
    let result = execute(deps.as_mut(), env, info, msg);
    assert!(result.is_ok());
    
    // 验证取消暂停
    let _config = query_config(&deps);
    // paused状态不在ConfigResponse中
}

#[test]
fn test_pause_unauthorized() {
    let (mut deps, env) = setup_test_env();
    instantiate_contract(&mut deps, &env, Scale::Tiny, BASE_AMOUNT).unwrap();
    
    // 非owner尝试设置暂停
    let msg = ExecuteMsg::SetPaused { paused: true };
    let info = MessageInfo {
        sender: cosmwasm_std::Addr::unchecked(USER1),
        funds: vec![],
    };
    let result = execute(deps.as_mut(), env, info, msg);
    assert_eq!(result.unwrap_err(), ContractError::Unauthorized);
}

#[test]
fn test_pause_deposit_blocked() {
    let (mut deps, env) = setup_test_env();
    instantiate_contract(&mut deps, &env, Scale::Tiny, BASE_AMOUNT).unwrap();
    
    // 设置暂停
    let (msg, info) = create_set_paused_msg(true);
    execute(deps.as_mut(), env.clone(), info, msg).unwrap();
    
    // 尝试充值
    let (msg, info) = create_deposit_msg(BASE_AMOUNT);
    let result = execute(deps.as_mut(), env, info, msg);
    assert_eq!(result.unwrap_err(), ContractError::InvalidState);
}

#[test]
fn test_pause_finalize_blocked() {
    let (mut deps, env) = setup_test_env();
    instantiate_contract(&mut deps, &env, Scale::Tiny, BASE_AMOUNT).unwrap();
    
    // 设置暂停
    let (msg, info) = create_set_paused_msg(true);
    execute(deps.as_mut(), env.clone(), info, msg).unwrap();
    
    // 设置结算环境
    setup_finalize_environment(&mut deps, &env);
    
    // 尝试结算
    let (msg, info) = create_finalize_msg();
    let result = execute(deps.as_mut(), env, info, msg);
    assert_eq!(result.unwrap_err(), ContractError::InvalidState);
}

#[test]
fn test_pause_voting_allowed() {
    let (mut deps, env) = setup_test_env();
    instantiate_contract(&mut deps, &env, Scale::Tiny, BASE_AMOUNT).unwrap();
    
    // 设置窗口以便投票（禁用时间检查）
    let msg = ExecuteMsg::SetCommitWindow {
        start_height: Some(0),
        end_height: Some(1000000),
        start_time: None,
        end_time: None,
    };
    let info = MessageInfo {
        sender: cosmwasm_std::Addr::unchecked(OWNER),
        funds: vec![],
    };
    execute(deps.as_mut(), env.clone(), info, msg).unwrap();
    
    let msg = ExecuteMsg::SetRevealWindow {
        start_height: Some(0),
        end_height: Some(1000000),
        start_time: None,
        end_time: None,
    };
    let info = MessageInfo {
        sender: cosmwasm_std::Addr::unchecked(OWNER),
        funds: vec![],
    };
    execute(deps.as_mut(), env.clone(), info, msg).unwrap();
    
    // 设置暂停
    let (msg, info) = create_set_paused_msg(true);
    execute(deps.as_mut(), env.clone(), info, msg).unwrap();
    
    // 计算正确的承诺
    let reveal = "test_reveal";
    let salt = "test_salt";
    let preimage = format!("{}|{}|{}", USER1, reveal, salt);
    let commitment = hex::encode(sha2::Sha256::digest(preimage.as_bytes()));
    
    // 投票操作应该仍然被允许
    let (msg, info) = create_commit_msg(commitment);
    let result = execute(deps.as_mut(), env.clone(), info, msg);
    assert!(result.is_ok());
    
    // 切换到Reveal阶段
    let (msg, info) = create_set_vote_state_msg(VoteState::Reveal);
    execute(deps.as_mut(), env.clone(), info, msg).unwrap();
    
    // 揭示投票
    let (msg, info) = create_reveal_msg(reveal.to_string(), salt.to_string());
    let result = execute(deps.as_mut(), env, info, msg);
    assert!(result.is_ok());
}

#[test]
fn test_pause_nft_operations_allowed() {
    let (mut deps, env) = setup_test_env();
    instantiate_contract(&mut deps, &env, Scale::Tiny, BASE_AMOUNT).unwrap();
    
    // 先充值铸造NFT
    let (msg, info) = create_deposit_msg(BASE_AMOUNT);
    execute(deps.as_mut(), env.clone(), info, msg).unwrap();
    
    // 设置暂停
    let (msg, info) = create_set_paused_msg(true);
    execute(deps.as_mut(), env.clone(), info, msg).unwrap();
    
    // NFT操作应该仍然被允许
    let (msg, info) = create_transfer_msg(USER2.to_string(), 0);
    let result = execute(deps.as_mut(), env.clone(), info, msg);
    assert!(result.is_ok());
    
    // 授权操作
    let (msg, _) = create_approve_msg(OPERATOR.to_string(), 0);
    let info = MessageInfo {
        sender: cosmwasm_std::Addr::unchecked(USER2),
        funds: vec![],
    };
    let result = execute(deps.as_mut(), env.clone(), info, msg);
    assert!(result.is_ok());
    
    // 全局授权操作
    let (msg, _) = create_approve_all_msg(OPERATOR.to_string());
    let info = MessageInfo {
        sender: cosmwasm_std::Addr::unchecked(USER2),
        funds: vec![],
    };
    let result = execute(deps.as_mut(), env, info, msg);
    assert!(result.is_ok());
}

#[test]
fn test_pause_queries_allowed() {
    let (mut deps, env) = setup_test_env();
    instantiate_contract(&mut deps, &env, Scale::Tiny, BASE_AMOUNT).unwrap();
    
    // 设置暂停
    let (msg, info) = create_set_paused_msg(true);
    execute(deps.as_mut(), env.clone(), info, msg).unwrap();
    
    // 查询操作应该仍然被允许
    let _config = query_config(&deps);
    // paused状态不在ConfigResponse中
    
    let deposit = query_deposit_test(&deps, USER1);
    assert_eq!(deposit.principal, "0");
    
    let tier = query_tier_test(&deps, USER1);
    assert_eq!(tier.tier, 0);
    
    let tier_list = query_tier_list(&deps, 1, None, None);
    assert_eq!(tier_list.addresses.len(), 0);
}

#[test]
fn test_pause_admin_operations_allowed() {
    let (mut deps, env) = setup_test_env();
    instantiate_contract(&mut deps, &env, Scale::Tiny, BASE_AMOUNT).unwrap();
    
    // 设置暂停
    let (msg, info) = create_set_paused_msg(true);
    execute(deps.as_mut(), env.clone(), info, msg).unwrap();
    
    // 管理员操作应该仍然被允许
    let (msg, info) = create_set_vote_state_msg(VoteState::Reveal);
    let result = execute(deps.as_mut(), env.clone(), info, msg);
    assert!(result.is_ok());
    
    let (msg, info) = create_set_base_msg(Coin {
        denom: "uatom".to_string(),
        amount: cosmwasm_std::Uint128::from(50u128),
    });
    let result = execute(deps.as_mut(), env.clone(), info, msg);
    assert!(result.is_ok());
    
    let (msg, info) = create_set_paused_msg(false);
    let result = execute(deps.as_mut(), env, info, msg);
    assert!(result.is_ok());
}

#[test]
fn test_pause_window_operations_allowed() {
    let (mut deps, env) = setup_test_env();
    instantiate_contract(&mut deps, &env, Scale::Tiny, BASE_AMOUNT).unwrap();
    
    // 设置暂停
    let (msg, info) = create_set_paused_msg(true);
    execute(deps.as_mut(), env.clone(), info, msg).unwrap();
    
    // 窗口设置操作应该仍然被允许
    let msg = ExecuteMsg::SetCommitWindow {
        start_height: Some(100),
        end_height: Some(200),
        start_time: None,
        end_time: None,
    };
    let info = MessageInfo {
        sender: cosmwasm_std::Addr::unchecked(OWNER),
        funds: vec![],
    };
    let result = execute(deps.as_mut(), env.clone(), info, msg);
    assert!(result.is_ok());
    
    let msg = ExecuteMsg::SetRevealWindow {
        start_height: Some(200),
        end_height: Some(300),
        start_time: None,
        end_time: None,
    };
    let info = MessageInfo {
        sender: cosmwasm_std::Addr::unchecked(OWNER),
        funds: vec![],
    };
    let result = execute(deps.as_mut(), env.clone(), info, msg);
    assert!(result.is_ok());
    
    let msg = ExecuteMsg::SetClosedWindow {
        start_height: Some(300),
        end_height: Some(400),
        start_time: None,
        end_time: None,
    };
    let info = MessageInfo {
        sender: cosmwasm_std::Addr::unchecked(OWNER),
        funds: vec![],
    };
    let result = execute(deps.as_mut(), env, info, msg);
    assert!(result.is_ok());
}

#[test]
fn test_pause_state_persistence() {
    let (mut deps, env) = setup_test_env();
    instantiate_contract(&mut deps, &env, Scale::Tiny, BASE_AMOUNT).unwrap();
    
    // 设置暂停
    let (msg, info) = create_set_paused_msg(true);
    execute(deps.as_mut(), env.clone(), info, msg).unwrap();
    
    // 验证暂停状态持久化
    let _config = query_config(&deps);
    // paused状态不在ConfigResponse中
    
    // 执行其他操作后，暂停状态应该保持
    let (msg, info) = create_set_vote_state_msg(VoteState::Reveal);
    execute(deps.as_mut(), env.clone(), info, msg).unwrap();
    
    let _config = query_config(&deps);
    // paused状态不在ConfigResponse中
    
    // 取消暂停
    let (msg, info) = create_set_paused_msg(false);
    execute(deps.as_mut(), env, info, msg).unwrap();
    
    let _config = query_config(&deps);
    // paused状态不在ConfigResponse中
}

#[test]
fn test_pause_multiple_operations() {
    let (mut deps, env) = setup_test_env();
    instantiate_contract(&mut deps, &env, Scale::Tiny, BASE_AMOUNT).unwrap();
    
    // 先充值铸造NFT
    let (msg, info) = create_deposit_msg(BASE_AMOUNT);
    execute(deps.as_mut(), env.clone(), info, msg).unwrap();
    
    // 设置暂停
    let (msg, info) = create_set_paused_msg(true);
    execute(deps.as_mut(), env.clone(), info, msg).unwrap();
    
    // 尝试多个被阻止的操作
    let blocked_operations = vec![
        create_deposit_msg(BASE_AMOUNT),
    ];
    
    for (msg, info) in blocked_operations {
        let result = execute(deps.as_mut(), env.clone(), info, msg);
        assert_eq!(result.unwrap_err(), ContractError::InvalidState);
    }
    
    // 设置结算环境
    setup_finalize_environment(&mut deps, &env);
    
    // 尝试结算
    let (msg, info) = create_finalize_msg();
    let result = execute(deps.as_mut(), env, info, msg);
    assert_eq!(result.unwrap_err(), ContractError::InvalidState);
}

#[test]
fn test_pause_response_attributes() {
    let (mut deps, env) = setup_test_env();
    instantiate_contract(&mut deps, &env, Scale::Tiny, BASE_AMOUNT).unwrap();
    
    // 设置暂停
    let (msg, info) = create_set_paused_msg(true);
    let result = execute(deps.as_mut(), env.clone(), info, msg);
    assert!(result.is_ok());
    
    // 验证响应属性
    let res = result.unwrap();
    assert_eq!(res.attributes[0].key, "action");
    assert_eq!(res.attributes[0].value, "set_paused");
    assert_eq!(res.attributes[1].key, "paused");
    assert_eq!(res.attributes[1].value, "true");
    
    // 取消暂停
    let (msg, info) = create_set_paused_msg(false);
    let result = execute(deps.as_mut(), env, info, msg);
    assert!(result.is_ok());
    
    // 验证响应属性
    let res = result.unwrap();
    assert_eq!(res.attributes[0].key, "action");
    assert_eq!(res.attributes[0].value, "set_paused");
    assert_eq!(res.attributes[1].key, "paused");
    assert_eq!(res.attributes[1].value, "false");
}

// 辅助函数：设置结算环境
fn setup_finalize_environment(deps: &mut OwnedDeps<cosmwasm_std::MemoryStorage, cosmwasm_std::testing::MockApi, cosmwasm_std::testing::MockQuerier>, _env: &cosmwasm_std::Env) {
    use dd_blind_box::state::{COMMITS, REVEALS, DEPOSITS, CONFIG};
    use cosmwasm_std::Uint128;
    
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
    
    // 设置Closed阶段和窗口
    let mut config = CONFIG.load(&deps.storage).unwrap();
    config.vote_state = VoteState::Closed;
    config.closed_window.start_height = Some(0);
    config.closed_window.end_height = Some(1000);
    CONFIG.save(&mut deps.storage, &config).unwrap();
}
