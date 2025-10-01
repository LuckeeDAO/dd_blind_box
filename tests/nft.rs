mod common;

use cosmwasm_std::{MessageInfo, testing::mock_env};
use dd_blind_box::{
    contract::{execute, query},
    error::ContractError,
    msg::{ExecuteMsg, QueryMsg},
    state::Scale,
};
use common::*;


#[test]
fn test_nft_mint_after_deposit() {
    let (mut deps, env) = setup_test_env();
    instantiate_contract(&mut deps, &env, Scale::Tiny, BASE_AMOUNT).unwrap();
    
    // 充值后自动铸造NFT
    let (msg, info) = create_deposit_msg(BASE_AMOUNT);
    execute(deps.as_mut(), env.clone(), info, msg).unwrap();
    
    // 验证NFT信息
    let nft_info = query_nft_info(&deps, 0);
    assert_eq!(nft_info.owner, USER1);
    assert_eq!(nft_info.approved, None);
    
    // 验证所有者查询
    let owner = query_owner_of(&deps, 0);
    assert_eq!(owner.owner, USER1);
}

#[test]
fn test_nft_transfer_owner() {
    let (mut deps, env) = setup_test_env();
    instantiate_contract(&mut deps, &env, Scale::Tiny, BASE_AMOUNT).unwrap();
    
    // 充值铸造NFT
    let (msg, info) = create_deposit_msg(BASE_AMOUNT);
    execute(deps.as_mut(), env.clone(), info, msg).unwrap();
    
    // 所有者转移NFT
    let (msg, info) = create_transfer_msg(USER2.to_string(), 0);
    let result = execute(deps.as_mut(), env, info, msg);
    assert!(result.is_ok());
    
    // 验证转移结果
    let owner = query_owner_of(&deps, 0);
    assert_eq!(owner.owner, USER2);
    
    let nft_info = query_nft_info(&deps, 0);
    assert_eq!(nft_info.owner, USER2);
    assert_eq!(nft_info.approved, None); // 转移后清除授权
}

#[test]
fn test_nft_transfer_unauthorized() {
    let (mut deps, env) = setup_test_env();
    instantiate_contract(&mut deps, &env, Scale::Tiny, BASE_AMOUNT).unwrap();
    
    // 用户1充值铸造NFT
    let (msg, info) = create_deposit_msg(BASE_AMOUNT);
    execute(deps.as_mut(), env.clone(), info, msg).unwrap();
    
    // 用户2尝试转移用户1的NFT
    let msg = ExecuteMsg::TransferNft { recipient: USER3.to_string(), token_id: 0 };
    let info = MessageInfo {
        sender: cosmwasm_std::Addr::unchecked(USER2),
        funds: vec![],
    };
    let result = execute(deps.as_mut(), env, info, msg);
    assert_eq!(result.unwrap_err(), ContractError::Unauthorized);
}

#[test]
fn test_nft_transfer_nonexistent_token() {
    let (mut deps, env) = setup_test_env();
    instantiate_contract(&mut deps, &env, Scale::Tiny, BASE_AMOUNT).unwrap();
    
    // 尝试转移不存在的NFT
    let (msg, info) = create_transfer_msg(USER2.to_string(), 999);
    let result = execute(deps.as_mut(), env, info, msg);
    assert!(result.is_err());
}

#[test]
fn test_nft_approve_success() {
    let (mut deps, env) = setup_test_env();
    instantiate_contract(&mut deps, &env, Scale::Tiny, BASE_AMOUNT).unwrap();
    
    // 充值铸造NFT
    let (msg, info) = create_deposit_msg(BASE_AMOUNT);
    execute(deps.as_mut(), env.clone(), info, msg).unwrap();
    
    // 授权给操作员
    let (msg, info) = create_approve_msg(OPERATOR.to_string(), 0);
    let result = execute(deps.as_mut(), env.clone(), info, msg);
    assert!(result.is_ok());
    
    // 验证授权状态
    let approval = query_approval(&deps, 0);
    assert_eq!(approval.spender, Some(OPERATOR.to_string()));
    
    let nft_info = query_nft_info(&deps, 0);
    assert_eq!(nft_info.approved, Some(OPERATOR.to_string()));
}

#[test]
fn test_nft_approve_unauthorized() {
    let (mut deps, env) = setup_test_env();
    instantiate_contract(&mut deps, &env, Scale::Tiny, BASE_AMOUNT).unwrap();
    
    // 用户1充值铸造NFT
    let (msg, info) = create_deposit_msg(BASE_AMOUNT);
    execute(deps.as_mut(), env.clone(), info, msg).unwrap();
    
    // 用户2尝试授权
    let msg = ExecuteMsg::Approve { spender: OPERATOR.to_string(), token_id: 0 };
    let info = MessageInfo {
        sender: cosmwasm_std::Addr::unchecked(USER2),
        funds: vec![],
    };
    let result = execute(deps.as_mut(), env, info, msg);
    assert_eq!(result.unwrap_err(), ContractError::Unauthorized);
}

#[test]
fn test_nft_approve_nonexistent_token() {
    let (mut deps, env) = setup_test_env();
    instantiate_contract(&mut deps, &env, Scale::Tiny, BASE_AMOUNT).unwrap();
    
    // 尝试授权不存在的NFT
    let (msg, info) = create_approve_msg(OPERATOR.to_string(), 999);
    let result = execute(deps.as_mut(), env, info, msg);
    assert!(result.is_err());
}

#[test]
fn test_nft_revoke_success() {
    let (mut deps, env) = setup_test_env();
    instantiate_contract(&mut deps, &env, Scale::Tiny, BASE_AMOUNT).unwrap();
    
    // 充值铸造NFT
    let (msg, info) = create_deposit_msg(BASE_AMOUNT);
    execute(deps.as_mut(), env.clone(), info, msg).unwrap();
    
    // 先授权
    let (msg, info) = create_approve_msg(OPERATOR.to_string(), 0);
    execute(deps.as_mut(), env.clone(), info, msg).unwrap();
    
    // 撤销授权
    let msg = ExecuteMsg::Revoke { spender: OPERATOR.to_string(), token_id: 0 };
    let info = MessageInfo {
        sender: cosmwasm_std::Addr::unchecked(USER1),
        funds: vec![],
    };
    let result = execute(deps.as_mut(), env, info, msg);
    assert!(result.is_ok());
    
    // 验证授权被撤销
    let approval = query_approval(&deps, 0);
    assert_eq!(approval.spender, None);
}

#[test]
fn test_nft_revoke_unauthorized() {
    let (mut deps, env) = setup_test_env();
    instantiate_contract(&mut deps, &env, Scale::Tiny, BASE_AMOUNT).unwrap();
    
    // 用户1充值铸造NFT
    let (msg, info) = create_deposit_msg(BASE_AMOUNT);
    execute(deps.as_mut(), env.clone(), info, msg).unwrap();
    
    // 用户2尝试撤销授权
    let msg = ExecuteMsg::Revoke { spender: OPERATOR.to_string(), token_id: 0 };
    let info = MessageInfo {
        sender: cosmwasm_std::Addr::unchecked(USER2),
        funds: vec![],
    };
    let result = execute(deps.as_mut(), env, info, msg);
    assert_eq!(result.unwrap_err(), ContractError::Unauthorized);
}

#[test]
fn test_nft_transfer_by_approved() {
    let (mut deps, env) = setup_test_env();
    instantiate_contract(&mut deps, &env, Scale::Tiny, BASE_AMOUNT).unwrap();
    
    // 用户1充值铸造NFT
    let (msg, info) = create_deposit_msg(BASE_AMOUNT);
    execute(deps.as_mut(), env.clone(), info, msg).unwrap();
    
    // 用户1授权给操作员
    let (msg, info) = create_approve_msg(OPERATOR.to_string(), 0);
    execute(deps.as_mut(), env.clone(), info, msg).unwrap();
    
    // 操作员转移NFT
    let msg = ExecuteMsg::TransferNft { recipient: USER2.to_string(), token_id: 0 };
    let info = MessageInfo {
        sender: cosmwasm_std::Addr::unchecked(OPERATOR),
        funds: vec![],
    };
    let result = execute(deps.as_mut(), env, info, msg);
    assert!(result.is_ok());
    
    // 验证转移结果
    let owner = query_owner_of(&deps, 0);
    assert_eq!(owner.owner, USER2);
}

#[test]
fn test_nft_approve_all_success() {
    let (mut deps, env) = setup_test_env();
    instantiate_contract(&mut deps, &env, Scale::Tiny, BASE_AMOUNT).unwrap();
    
    // 充值铸造多个NFT
    let (msg, info) = create_deposit_msg(BASE_AMOUNT * 3);
    execute(deps.as_mut(), env.clone(), info, msg).unwrap();
    
    // 设置全局操作员
    let (msg, info) = create_approve_all_msg(OPERATOR.to_string());
    let result = execute(deps.as_mut(), env.clone(), info, msg);
    assert!(result.is_ok());
    
    // 验证全局授权状态
    let is_approved = query_is_approved_for_all(&deps, USER1, OPERATOR);
    assert!(is_approved.approved);
}

#[test]
fn test_nft_approve_all_unauthorized() {
    let (mut deps, env) = setup_test_env();
    instantiate_contract(&mut deps, &env, Scale::Tiny, BASE_AMOUNT).unwrap();
    
    // 用户1充值铸造NFT
    let (msg, info) = create_deposit_msg(BASE_AMOUNT);
    execute(deps.as_mut(), env.clone(), info, msg).unwrap();
    
    // 用户2设置自己的全局操作员（这是允许的，符合CW721标准）
    let msg = ExecuteMsg::ApproveAll { operator: OPERATOR.to_string() };
    let info = MessageInfo {
        sender: cosmwasm_std::Addr::unchecked(USER2),
        funds: vec![],
    };
    let result = execute(deps.as_mut(), env.clone(), info, msg);
    assert!(result.is_ok());
    
    // 验证用户2的全局授权状态
    let is_approved = query_is_approved_for_all(&deps, USER2, OPERATOR);
    assert!(is_approved.approved);
}

#[test]
fn test_nft_revoke_all_success() {
    let (mut deps, env) = setup_test_env();
    instantiate_contract(&mut deps, &env, Scale::Tiny, BASE_AMOUNT).unwrap();
    
    // 充值铸造NFT
    let (msg, info) = create_deposit_msg(BASE_AMOUNT);
    execute(deps.as_mut(), env.clone(), info, msg).unwrap();
    
    // 先设置全局操作员
    let (msg, info) = create_approve_all_msg(OPERATOR.to_string());
    execute(deps.as_mut(), env.clone(), info, msg).unwrap();
    
    // 撤销全局操作员
    let msg = ExecuteMsg::RevokeAll { operator: OPERATOR.to_string() };
    let info = MessageInfo {
        sender: cosmwasm_std::Addr::unchecked(USER1),
        funds: vec![],
    };
    let result = execute(deps.as_mut(), env, info, msg);
    assert!(result.is_ok());
    
    // 验证全局授权被撤销
    let is_approved = query_is_approved_for_all(&deps, USER1, OPERATOR);
    assert!(!is_approved.approved);
}

#[test]
fn test_nft_revoke_all_unauthorized() {
    let (mut deps, env) = setup_test_env();
    instantiate_contract(&mut deps, &env, Scale::Tiny, BASE_AMOUNT).unwrap();
    
    // 用户1充值铸造NFT
    let (msg, info) = create_deposit_msg(BASE_AMOUNT);
    execute(deps.as_mut(), env.clone(), info, msg).unwrap();
    
    // 用户2撤销自己的全局操作员（这是允许的，符合CW721标准）
    let msg = ExecuteMsg::RevokeAll { operator: OPERATOR.to_string() };
    let info = MessageInfo {
        sender: cosmwasm_std::Addr::unchecked(USER2),
        funds: vec![],
    };
    let result = execute(deps.as_mut(), env.clone(), info, msg);
    assert!(result.is_ok());
    
    // 验证用户2的全局授权状态已被撤销
    let is_approved = query_is_approved_for_all(&deps, USER2, OPERATOR);
    assert!(!is_approved.approved);
}

#[test]
fn test_nft_transfer_by_global_operator() {
    let (mut deps, env) = setup_test_env();
    instantiate_contract(&mut deps, &env, Scale::Tiny, BASE_AMOUNT).unwrap();
    
    // 用户1充值铸造NFT
    let (msg, info) = create_deposit_msg(BASE_AMOUNT);
    execute(deps.as_mut(), env.clone(), info, msg).unwrap();
    
    // 用户1设置全局操作员
    let (msg, info) = create_approve_all_msg(OPERATOR.to_string());
    execute(deps.as_mut(), env.clone(), info, msg).unwrap();
    
    // 全局操作员转移NFT
    let msg = ExecuteMsg::TransferNft { recipient: USER2.to_string(), token_id: 0 };
    let info = MessageInfo {
        sender: cosmwasm_std::Addr::unchecked(OPERATOR),
        funds: vec![],
    };
    let result = execute(deps.as_mut(), env, info, msg);
    assert!(result.is_ok());
    
    // 验证转移结果
    let owner = query_owner_of(&deps, 0);
    assert_eq!(owner.owner, USER2);
}

#[test]
fn test_nft_query_nonexistent_token() {
    let (mut deps, env) = setup_test_env();
    instantiate_contract(&mut deps, &env, Scale::Tiny, BASE_AMOUNT).unwrap();
    
    // 查询不存在的NFT
    let result = query(deps.as_ref(), mock_env(), QueryMsg::OwnerOf { token_id: 999 });
    assert!(result.is_err());
    
    let result = query(deps.as_ref(), mock_env(), QueryMsg::NftInfo { token_id: 999 });
    assert!(result.is_err());
    
    let result = query(deps.as_ref(), mock_env(), QueryMsg::Approval { token_id: 999 });
    assert!(result.is_err());
}

#[test]
fn test_nft_multiple_tokens() {
    let (mut deps, env) = setup_test_env();
    instantiate_contract(&mut deps, &env, Scale::Tiny, BASE_AMOUNT).unwrap();
    
    // 用户1充值铸造3个NFT
    let (msg, info) = create_deposit_msg(BASE_AMOUNT * 3);
    execute(deps.as_mut(), env.clone(), info, msg).unwrap();
    
    // 验证所有NFT的所有者
    for i in 0..3 {
        let owner = query_owner_of(&deps, i);
        assert_eq!(owner.owner, USER1);
    }
    
    // 转移第一个NFT
    let (msg, info) = create_transfer_msg(USER2.to_string(), 0);
    execute(deps.as_mut(), env.clone(), info, msg).unwrap();
    
    // 验证转移结果
    let owner0 = query_owner_of(&deps, 0);
    assert_eq!(owner0.owner, USER2);
    
    let owner1 = query_owner_of(&deps, 1);
    assert_eq!(owner1.owner, USER1);
    
    let owner2 = query_owner_of(&deps, 2);
    assert_eq!(owner2.owner, USER1);
}

#[test]
fn test_nft_approve_override() {
    let (mut deps, env) = setup_test_env();
    instantiate_contract(&mut deps, &env, Scale::Tiny, BASE_AMOUNT).unwrap();
    
    // 充值铸造NFT
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
