mod common;

use cosmwasm_std::{Coin, Uint128, MessageInfo};
use dd_blind_box::{
    contract::{execute, instantiate},
    error::ContractError,
    msg::{ExecuteMsg, InstantiateMsg},
    state::{Scale, VoteState},
};
use common::*;


#[test]
fn test_instantiate_success() {
    let (mut deps, env) = setup_test_env();
    
    // 测试正常初始化
    let result = instantiate_contract(&mut deps, &env, Scale::new_tiny(), BASE_AMOUNT);
    assert!(result.is_ok());
    
    // 验证配置
    let config = query_config(&deps);
    assert_eq!(config.owner, OWNER);
    assert_eq!(config.total_supply, 10); // Tiny scale
    assert_eq!(config.base.denom, BASE_DENOM);
    assert_eq!(config.base.amount, Uint128::from(BASE_AMOUNT));
    assert_eq!(config.vote_state, VoteState::Commit);
    assert_eq!(config.scale, Scale::new_tiny());
}

#[test]
fn test_instantiate_different_scales() {
    // 测试不同规模
    let scales = vec![
        (Scale::new_tiny(), 10),
        (Scale::new_small(), 100),
        (Scale::new_medium(), 1_000),
        (Scale::new_large(), 10_000),
        (Scale::new_huge(), 100_000),
    ];
    
    for (scale, expected_supply) in scales {
        let (mut deps, env) = setup_test_env();
        let result = instantiate_contract(&mut deps, &env, scale.clone(), BASE_AMOUNT);
        assert!(result.is_ok());
        
        let config = query_config(&deps);
        assert_eq!(config.total_supply, expected_supply);
        assert_eq!(config.scale, scale);
    }
}

#[test]
fn test_instantiate_different_base_coins() {
    // 测试不同基础币种
    let base_coins = vec![
        Coin { denom: "ujunox".to_string(), amount: Uint128::from(100u128) },
        Coin { denom: "uatom".to_string(), amount: Uint128::from(50u128) },
        Coin { denom: "uosmo".to_string(), amount: Uint128::from(200u128) },
    ];
    
    for base in base_coins {
        let (mut deps, env) = setup_test_env();
        let info = MessageInfo {
            sender: cosmwasm_std::Addr::unchecked(OWNER),
            funds: vec![],
        };
        let msg = InstantiateMsg {
            scale: Scale::new_tiny(),
            base: base.clone(),
        };
        
        let result = instantiate(deps.as_mut(), env.clone(), info, msg);
        assert!(result.is_ok());
        
        let config = query_config(&deps);
        assert_eq!(config.base, base);
    }
}

#[test]
fn test_instantiate_zero_amount() {
    let (mut deps, env) = setup_test_env();
    
    // 测试零金额基础币种
    let info = MessageInfo {
        sender: cosmwasm_std::Addr::unchecked(OWNER),
        funds: vec![],
    };
    let msg = InstantiateMsg {
        scale: Scale::new_tiny(),
        base: Coin {
            denom: BASE_DENOM.to_string(),
            amount: Uint128::zero(),
        },
    };
    
    let result = instantiate(deps.as_mut(), env, info, msg);
    assert!(result.is_ok()); // 零金额应该被允许
}

#[test]
fn test_config_query_fields() {
    let (mut deps, env) = setup_test_env();
    instantiate_contract(&mut deps, &env, Scale::new_medium(), BASE_AMOUNT).unwrap();
    
    let config = query_config(&deps);
    
    // 验证所有字段
    assert_eq!(config.owner, OWNER);
    assert_eq!(config.total_supply, 1_000);
    assert_eq!(config.base.denom, BASE_DENOM);
    assert_eq!(config.base.amount, Uint128::from(BASE_AMOUNT));
    assert_eq!(config.vote_state, VoteState::Commit);
    assert_eq!(config.scale, Scale::new_medium());
}

#[test]
fn test_set_base_success() {
    let (mut deps, env) = setup_test_env();
    instantiate_contract(&mut deps, &env, Scale::new_tiny(), BASE_AMOUNT).unwrap();
    
    // 设置新的基础币种
    let new_base = Coin {
        denom: "uatom".to_string(),
        amount: Uint128::from(50u128),
    };
    
    let (msg, info) = create_set_base_msg(new_base.clone());
    let result = execute(deps.as_mut(), env, info, msg);
    assert!(result.is_ok());
    
    // 验证配置更新
    let config = query_config(&deps);
    assert_eq!(config.base, new_base);
}

#[test]
fn test_set_base_unauthorized() {
    let (mut deps, env) = setup_test_env();
    instantiate_contract(&mut deps, &env, Scale::new_tiny(), BASE_AMOUNT).unwrap();
    
    // 非owner尝试设置基础币种
    let new_base = Coin {
        denom: "uatom".to_string(),
        amount: Uint128::from(50u128),
    };
    
    let msg = ExecuteMsg::SetBase { base: new_base };
    let info = MessageInfo {
        sender: cosmwasm_std::Addr::unchecked(USER1),
        funds: vec![],
    };
    let result = execute(deps.as_mut(), env, info, msg);
    
    assert_eq!(result.unwrap_err(), ContractError::Unauthorized);
}

#[test]
fn test_set_vote_state_success() {
    let (mut deps, env) = setup_test_env();
    instantiate_contract(&mut deps, &env, Scale::new_tiny(), BASE_AMOUNT).unwrap();
    
    // 设置投票状态为Reveal
    let (msg, info) = create_set_vote_state_msg(VoteState::Reveal);
    let result = execute(deps.as_mut(), env, info, msg);
    assert!(result.is_ok());
    
    // 验证状态更新
    let config = query_config(&deps);
    assert_eq!(config.vote_state, VoteState::Reveal);
}

#[test]
fn test_set_vote_state_unauthorized() {
    let (mut deps, env) = setup_test_env();
    instantiate_contract(&mut deps, &env, Scale::new_tiny(), BASE_AMOUNT).unwrap();
    
    // 非owner尝试设置投票状态
    let msg = ExecuteMsg::SetVoteState { state: VoteState::Reveal };
    let info = MessageInfo {
        sender: cosmwasm_std::Addr::unchecked(USER1),
        funds: vec![],
    };
    let result = execute(deps.as_mut(), env, info, msg);
    
    assert_eq!(result.unwrap_err(), ContractError::Unauthorized);
}

#[test]
fn test_set_paused_success() {
    let (mut deps, env) = setup_test_env();
    instantiate_contract(&mut deps, &env, Scale::new_tiny(), BASE_AMOUNT).unwrap();
    
    // 设置暂停
    let (msg, info) = create_set_paused_msg(true);
    let result = execute(deps.as_mut(), env.clone(), info, msg);
    assert!(result.is_ok());
    
    // 验证暂停状态
    let _config = query_config(&deps);
    // 暂停状态不在ConfigResponse中，需要通过其他方式验证
    
    // 取消暂停
    let (msg, info) = create_set_paused_msg(false);
    let result = execute(deps.as_mut(), env, info, msg);
    assert!(result.is_ok());
    
    let _config = query_config(&deps);
    // 暂停状态不在ConfigResponse中，需要通过其他方式验证
}

#[test]
fn test_set_paused_unauthorized() {
    let (mut deps, env) = setup_test_env();
    instantiate_contract(&mut deps, &env, Scale::new_tiny(), BASE_AMOUNT).unwrap();
    
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
fn test_set_commit_window() {
    let (mut deps, env) = setup_test_env();
    instantiate_contract(&mut deps, &env, Scale::new_tiny(), BASE_AMOUNT).unwrap();
    
    // 设置提交窗口
    let msg = ExecuteMsg::SetCommitWindow {
        start_height: Some(100),
        end_height: Some(200),
        start_time: Some(1000),
        end_time: Some(2000),
    };
    let info = MessageInfo {
        sender: cosmwasm_std::Addr::unchecked(OWNER),
        funds: vec![],
    };
    let result = execute(deps.as_mut(), env, info, msg);
    assert!(result.is_ok());
}

#[test]
fn test_set_reveal_window() {
    let (mut deps, env) = setup_test_env();
    instantiate_contract(&mut deps, &env, Scale::new_tiny(), BASE_AMOUNT).unwrap();
    
    // 设置揭示窗口
    let msg = ExecuteMsg::SetRevealWindow {
        start_height: Some(200),
        end_height: Some(300),
        start_time: Some(2000),
        end_time: Some(3000),
    };
    let info = MessageInfo {
        sender: cosmwasm_std::Addr::unchecked(OWNER),
        funds: vec![],
    };
    let result = execute(deps.as_mut(), env, info, msg);
    assert!(result.is_ok());
}

#[test]
fn test_set_closed_window() {
    let (mut deps, env) = setup_test_env();
    instantiate_contract(&mut deps, &env, Scale::new_tiny(), BASE_AMOUNT).unwrap();
    
    // 设置关闭窗口
    let msg = ExecuteMsg::SetClosedWindow {
        start_height: Some(300),
        end_height: Some(400),
        start_time: Some(3000),
        end_time: Some(4000),
    };
    let info = MessageInfo {
        sender: cosmwasm_std::Addr::unchecked(OWNER),
        funds: vec![],
    };
    let result = execute(deps.as_mut(), env, info, msg);
    assert!(result.is_ok());
}

#[test]
fn test_set_window_unauthorized() {
    let (mut deps, env) = setup_test_env();
    instantiate_contract(&mut deps, &env, Scale::new_tiny(), BASE_AMOUNT).unwrap();
    
    // 非owner尝试设置窗口
    let msg = ExecuteMsg::SetCommitWindow {
        start_height: Some(100),
        end_height: Some(200),
        start_time: None,
        end_time: None,
    };
    let info = MessageInfo {
        sender: cosmwasm_std::Addr::unchecked(USER1),
        funds: vec![],
    };
    let result = execute(deps.as_mut(), env, info, msg);
    
    assert_eq!(result.unwrap_err(), ContractError::Unauthorized);
}
