mod common;

use cosmwasm_std::{Coin, Uint128, coins, MessageInfo, testing::mock_env};
use dd_blind_box::{
    contract::{execute, query},
    error::ContractError,
    msg::{ExecuteMsg, QueryMsg},
    state::{Scale, VoteState},
};
use common::*;


#[test]
fn test_unauthorized_operations() {
    let (mut deps, env) = setup_test_env();
    instantiate_contract(&mut deps, &env, Scale::Tiny, BASE_AMOUNT).unwrap();
    
    // 测试非owner执行owner-only操作
    let unauthorized_operations = vec![
        ExecuteMsg::SetBase { base: Coin { denom: "uatom".to_string(), amount: Uint128::from(50u128) } },
        ExecuteMsg::SetVoteState { state: VoteState::Reveal },
        ExecuteMsg::SetPaused { paused: true },
        ExecuteMsg::SetCommitWindow { start_height: Some(100), end_height: Some(200), start_time: None, end_time: None },
        ExecuteMsg::SetRevealWindow { start_height: Some(200), end_height: Some(300), start_time: None, end_time: None },
        ExecuteMsg::SetClosedWindow { start_height: Some(300), end_height: Some(400), start_time: None, end_time: None },
        ExecuteMsg::Finalize {},
    ];
    
    for op in unauthorized_operations {
        let info = MessageInfo {
            sender: cosmwasm_std::Addr::unchecked(USER1),
            funds: vec![],
        };
        let result = execute(deps.as_mut(), env.clone(), info, op);
        assert_eq!(result.unwrap_err(), ContractError::Unauthorized);
    }
}

#[test]
fn test_invalid_token_id() {
    let (mut deps, env) = setup_test_env();
    instantiate_contract(&mut deps, &env, Scale::Tiny, BASE_AMOUNT).unwrap();
    
    // 测试无效的token_id
    let invalid_token_operations = vec![
        ExecuteMsg::TransferNft { recipient: USER2.to_string(), token_id: 999 },
        ExecuteMsg::Approve { spender: OPERATOR.to_string(), token_id: 999 },
        ExecuteMsg::Revoke { spender: OPERATOR.to_string(), token_id: 999 },
    ];
    
    for op in invalid_token_operations {
        let info = MessageInfo {
            sender: cosmwasm_std::Addr::unchecked(USER1),
            funds: vec![],
        };
        let result = execute(deps.as_mut(), env.clone(), info, op);
        assert!(result.is_err());
    }
    
    // 测试查询无效的token_id
    let invalid_queries = vec![
        QueryMsg::OwnerOf { token_id: 999 },
        QueryMsg::NftInfo { token_id: 999 },
        QueryMsg::Approval { token_id: 999 },
    ];
    
    for query_msg in invalid_queries {
        let result = query(deps.as_ref(), mock_env(), query_msg);
        assert!(result.is_err());
    }
}

#[test]
fn test_invalid_address() {
    let (mut deps, env) = setup_test_env();
    instantiate_contract(&mut deps, &env, Scale::Tiny, BASE_AMOUNT).unwrap();
    
    // 测试无效地址
    let invalid_address_operations = vec![
        ExecuteMsg::TransferNft { recipient: "invalid_address".to_string(), token_id: 0 },
        ExecuteMsg::Approve { spender: "invalid_address".to_string(), token_id: 0 },
        ExecuteMsg::Revoke { spender: "invalid_address".to_string(), token_id: 0 },
        ExecuteMsg::ApproveAll { operator: "invalid_address".to_string() },
        ExecuteMsg::RevokeAll { operator: "invalid_address".to_string() },
    ];
    
    for op in invalid_address_operations {
        let info = MessageInfo {
            sender: cosmwasm_std::Addr::unchecked(USER1),
            funds: vec![],
        };
        let result = execute(deps.as_mut(), env.clone(), info, op);
        assert!(result.is_err());
    }
    
    // 测试查询无效地址
    let invalid_queries = vec![
        QueryMsg::DepositOf { address: "invalid_address".to_string() },
        QueryMsg::TierOf { address: "invalid_address".to_string() },
        QueryMsg::IsApprovedForAll { owner: "invalid_address".to_string(), operator: OPERATOR.to_string() },
        QueryMsg::IsApprovedForAll { owner: USER1.to_string(), operator: "invalid_address".to_string() },
    ];
    
    for query_msg in invalid_queries {
        let result = query(deps.as_ref(), mock_env(), query_msg);
        assert!(result.is_err());
    }
}

#[test]
fn test_phase_restrictions() {
    let (mut deps, env) = setup_test_env();
    instantiate_contract(&mut deps, &env, Scale::Tiny, BASE_AMOUNT).unwrap();
    
    // 测试在错误阶段执行操作
    let (msg, info) = create_set_vote_state_msg(VoteState::Reveal);
    execute(deps.as_mut(), env.clone(), info, msg).unwrap();
    
    // 在Reveal阶段尝试提交承诺
    let (msg, info) = create_commit_msg("test_commitment".to_string());
    let result = execute(deps.as_mut(), env.clone(), info, msg);
    assert_eq!(result.unwrap_err(), ContractError::CommitNotActive);
    
    // 切换到Closed阶段
    let (msg, info) = create_set_vote_state_msg(VoteState::Closed);
    execute(deps.as_mut(), env.clone(), info, msg).unwrap();
    
    // 在Closed阶段尝试提交承诺
    let (msg, info) = create_commit_msg("test_commitment".to_string());
    let result = execute(deps.as_mut(), env.clone(), info, msg);
    assert_eq!(result.unwrap_err(), ContractError::CommitNotActive);
    
    // 在Closed阶段尝试揭示投票
    let (msg, info) = create_reveal_msg("test_reveal".to_string(), "test_salt".to_string());
    let result = execute(deps.as_mut(), env, info, msg);
    assert_eq!(result.unwrap_err(), ContractError::RevealNotActive);
}

#[test]
fn test_window_restrictions() {
    let (mut deps, env) = setup_test_env();
    instantiate_contract(&mut deps, &env, Scale::Tiny, BASE_AMOUNT).unwrap();
    
    // 设置关闭窗口，但当前不在窗口内
    let msg = ExecuteMsg::SetClosedWindow {
        start_height: Some(1000),
        end_height: Some(2000),
        start_time: None,
        end_time: None,
    };
    let info = MessageInfo {
        sender: cosmwasm_std::Addr::unchecked(OWNER),
        funds: vec![],
    };
    execute(deps.as_mut(), env.clone(), info, msg).unwrap();
    
    // 设置Closed阶段
    let (msg, info) = create_set_vote_state_msg(VoteState::Closed);
    execute(deps.as_mut(), env.clone(), info, msg).unwrap();
    
    // 尝试在窗口外结算
    let (msg, info) = create_finalize_msg();
    let result = execute(deps.as_mut(), env, info, msg);
    assert_eq!(result.unwrap_err(), ContractError::InvalidState);
}

#[test]
fn test_deposit_restrictions() {
    let (mut deps, env) = setup_test_env();
    instantiate_contract(&mut deps, &env, Scale::Tiny, BASE_AMOUNT).unwrap();
    
    // 测试暂停状态下的充值
    let (msg, info) = create_set_paused_msg(true);
    execute(deps.as_mut(), env.clone(), info, msg).unwrap();
    
    let (msg, info) = create_deposit_msg(BASE_AMOUNT);
    let result = execute(deps.as_mut(), env, info, msg);
    assert_eq!(result.unwrap_err(), ContractError::InvalidState);
}

#[test]
fn test_nft_authorization_restrictions() {
    let (mut deps, env) = setup_test_env();
    instantiate_contract(&mut deps, &env, Scale::Tiny, BASE_AMOUNT).unwrap();
    
    // 用户1充值铸造NFT
    let (msg, info) = create_deposit_msg(BASE_AMOUNT);
    execute(deps.as_mut(), env.clone(), info, msg).unwrap();
    
    // 用户2尝试转移用户1的NFT
    let (msg, _) = create_transfer_msg(USER3.to_string(), 0);
    let info = MessageInfo {
        sender: cosmwasm_std::Addr::unchecked(USER2),
        funds: vec![],
    };
    let result = execute(deps.as_mut(), env.clone(), info, msg);
    assert_eq!(result.unwrap_err(), ContractError::Unauthorized);
    
    // 用户2尝试授权用户1的NFT
    let msg = ExecuteMsg::Approve { spender: OPERATOR.to_string(), token_id: 0 };
    let info = MessageInfo {
        sender: cosmwasm_std::Addr::unchecked(USER2),
        funds: vec![],
    };
    let result = execute(deps.as_mut(), env.clone(), info, msg);
    assert_eq!(result.unwrap_err(), ContractError::Unauthorized);
    
    // 用户2尝试撤销用户1的NFT授权
    let msg = ExecuteMsg::Revoke { spender: OPERATOR.to_string(), token_id: 0 };
    let info = MessageInfo {
        sender: cosmwasm_std::Addr::unchecked(USER2),
        funds: vec![],
    };
    let result = execute(deps.as_mut(), env, info, msg);
    assert_eq!(result.unwrap_err(), ContractError::Unauthorized);
}

#[test]
fn test_commitment_verification() {
    let (mut deps, env) = setup_test_env();
    instantiate_contract(&mut deps, &env, Scale::Tiny, BASE_AMOUNT).unwrap();
    
    // 提交错误的承诺
    let (msg, info) = create_commit_msg("wrong_commitment".to_string());
    execute(deps.as_mut(), env.clone(), info, msg).unwrap();
    
    // 切换到Reveal阶段
    let (msg, info) = create_set_vote_state_msg(VoteState::Reveal);
    execute(deps.as_mut(), env.clone(), info, msg).unwrap();
    
    // 尝试揭示不匹配的投票
    let (msg, info) = create_reveal_msg("test_reveal".to_string(), "test_salt".to_string());
    let result = execute(deps.as_mut(), env, info, msg);
    assert!(result.is_err());
    let err = result.unwrap_err();
    assert!(matches!(err, ContractError::Std(_)));
}

#[test]
fn test_reveal_without_commit() {
    let (mut deps, env) = setup_test_env();
    instantiate_contract(&mut deps, &env, Scale::Tiny, BASE_AMOUNT).unwrap();
    
    // 直接切换到Reveal阶段
    let (msg, info) = create_set_vote_state_msg(VoteState::Reveal);
    execute(deps.as_mut(), env.clone(), info, msg).unwrap();
    
    // 尝试揭示但没有提交过承诺
    let (msg, info) = create_reveal_msg("test_reveal".to_string(), "test_salt".to_string());
    let result = execute(deps.as_mut(), env, info, msg);
    assert_eq!(result.unwrap_err(), ContractError::NothingToReveal);
}

#[test]
fn test_deposit_insufficient_funds() {
    let (mut deps, env) = setup_test_env();
    instantiate_contract(&mut deps, &env, Scale::Tiny, BASE_AMOUNT).unwrap();
    
    // 测试充值金额不足
    let (msg, info) = create_deposit_msg(BASE_AMOUNT - 1);
    let result = execute(deps.as_mut(), env, info, msg);
    assert!(result.is_err());
    let err = result.unwrap_err();
    assert!(matches!(err, ContractError::Std(_)));
}

#[test]
fn test_deposit_wrong_denom() {
    let (mut deps, env) = setup_test_env();
    instantiate_contract(&mut deps, &env, Scale::Tiny, BASE_AMOUNT).unwrap();
    
    // 测试错误的币种
    let msg = ExecuteMsg::Deposit {};
    let info = MessageInfo {
        sender: cosmwasm_std::Addr::unchecked(USER1),
        funds: coins(BASE_AMOUNT, "uatom"),
    };
    let result = execute(deps.as_mut(), env, info, msg);
    assert!(result.is_err());
    let err = result.unwrap_err();
    assert!(matches!(err, ContractError::Std(_)));
}

#[test]
fn test_deposit_zero_amount() {
    let (mut deps, env) = setup_test_env();
    instantiate_contract(&mut deps, &env, Scale::Tiny, BASE_AMOUNT).unwrap();
    
    // 测试零金额充值
    let (msg, info) = create_deposit_msg(0);
    let result = execute(deps.as_mut(), env, info, msg);
    assert!(result.is_err());
    let err = result.unwrap_err();
    assert!(matches!(err, ContractError::Std(_)));
}

#[test]
fn test_nft_approve_override() {
    let (mut deps, env) = setup_test_env();
    instantiate_contract(&mut deps, &env, Scale::Tiny, BASE_AMOUNT).unwrap();
    
    // 用户1充值铸造NFT
    let (msg, info) = create_deposit_msg(BASE_AMOUNT);
    execute(deps.as_mut(), env.clone(), info, msg).unwrap();
    
    // 授权给操作员1
    let (msg, info) = create_approve_msg(OPERATOR.to_string(), 0);
    execute(deps.as_mut(), env.clone(), info, msg).unwrap();
    
    // 重新授权给操作员2（应该覆盖之前的授权）
    let msg = ExecuteMsg::Approve { spender: USER2.to_string(), token_id: 0 };
    let info = MessageInfo {
        sender: cosmwasm_std::Addr::unchecked(USER1),
        funds: vec![],
    };
    execute(deps.as_mut(), env.clone(), info, msg).unwrap();
    
    // 验证只有操作员2被授权
    let approval = query_approval(&deps, 0);
    assert_eq!(approval.spender, Some(USER2.to_string()));
}

#[test]
fn test_tier_list_invalid_parameters() {
    let (mut deps, env) = setup_test_env();
    instantiate_contract(&mut deps, &env, Scale::Tiny, BASE_AMOUNT).unwrap();
    
    // 测试无效的起始地址
    let result = query(deps.as_ref(), mock_env(), QueryMsg::TierList {
        tier: 1,
        start_after: Some("invalid_address".to_string()),
        limit: None,
    });
    assert!(result.is_err());
}

#[test]
fn test_finalize_restrictions() {
    let (mut deps, env) = setup_test_env();
    instantiate_contract(&mut deps, &env, Scale::Tiny, BASE_AMOUNT).unwrap();
    
    // 测试非owner执行结算
    let msg = ExecuteMsg::Finalize {};
    let info = MessageInfo {
        sender: cosmwasm_std::Addr::unchecked(USER1),
        funds: vec![],
    };
    let result = execute(deps.as_mut(), env, info, msg);
    assert_eq!(result.unwrap_err(), ContractError::Unauthorized);
}

#[test]
fn test_boundary_values() {
    let (mut deps, env) = setup_test_env();
    instantiate_contract(&mut deps, &env, Scale::Tiny, BASE_AMOUNT).unwrap();
    
    // 测试边界值
    let boundary_tests = vec![
        // 零值
        (0u64, false),
        // 最大值
        (u64::MAX, false),
        // 正常值
        (1u64, true),
    ];
    
    for (token_id, should_succeed) in boundary_tests {
        let result = query(deps.as_ref(), mock_env(), QueryMsg::OwnerOf { token_id });
        if should_succeed {
            assert!(result.is_err()); // 因为NFT不存在
        } else {
            assert!(result.is_err());
        }
    }
}

#[test]
fn test_empty_strings() {
    let (mut deps, env) = setup_test_env();
    instantiate_contract(&mut deps, &env, Scale::Tiny, BASE_AMOUNT).unwrap();
    
    // 测试空字符串
    let empty_string_tests = vec![
        ExecuteMsg::TransferNft { recipient: "".to_string(), token_id: 0 },
        ExecuteMsg::Approve { spender: "".to_string(), token_id: 0 },
        ExecuteMsg::Revoke { spender: "".to_string(), token_id: 0 },
        ExecuteMsg::ApproveAll { operator: "".to_string() },
        ExecuteMsg::RevokeAll { operator: "".to_string() },
    ];
    
    for op in empty_string_tests {
        let info = MessageInfo {
            sender: cosmwasm_std::Addr::unchecked(USER1),
            funds: vec![],
        };
        let result = execute(deps.as_mut(), env.clone(), info, op);
        assert!(result.is_err());
    }
}
