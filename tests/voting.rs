mod common;

use cosmwasm_std::MessageInfo;
use dd_blind_box::{
    contract::execute,
    error::ContractError,
    msg::ExecuteMsg,
    state::{Scale, VoteState},
};
use common::*;


#[test]
fn test_commit_vote_success() {
    let (mut deps, env) = setup_test_env();
    instantiate_contract(&mut deps, &env, Scale::Tiny, BASE_AMOUNT).unwrap();
    
    // 正常提交承诺
    let commitment = "test_commitment_hash";
    let (msg, info) = create_commit_msg(commitment.to_string());
    let result = execute(deps.as_mut(), env, info, msg);
    assert!(result.is_ok());
    
    // 验证响应属性
    let res = result.unwrap();
    assert_eq!(res.attributes[0].key, "action");
    assert_eq!(res.attributes[0].value, "commit");
    assert_eq!(res.attributes[1].key, "voter");
    assert_eq!(res.attributes[1].value, USER1);
    assert_eq!(res.attributes[2].key, "commitment");
    assert_eq!(res.attributes[2].value, commitment);
}

#[test]
fn test_commit_vote_wrong_phase() {
    let (mut deps, env) = setup_test_env();
    instantiate_contract(&mut deps, &env, Scale::Tiny, BASE_AMOUNT).unwrap();
    
    // 切换到Reveal阶段
    let (msg, info) = create_set_vote_state_msg(VoteState::Reveal);
    execute(deps.as_mut(), env.clone(), info, msg).unwrap();
    
    // 尝试在Reveal阶段提交承诺
    let (msg, info) = create_commit_msg("test_commitment".to_string());
    let result = execute(deps.as_mut(), env, info, msg);
    assert_eq!(result.unwrap_err(), ContractError::CommitNotActive);
}

#[test]
fn test_commit_vote_closed_phase() {
    let (mut deps, env) = setup_test_env();
    instantiate_contract(&mut deps, &env, Scale::Tiny, BASE_AMOUNT).unwrap();
    
    // 切换到Closed阶段
    let (msg, info) = create_set_vote_state_msg(VoteState::Closed);
    execute(deps.as_mut(), env.clone(), info, msg).unwrap();
    
    // 尝试在Closed阶段提交承诺
    let (msg, info) = create_commit_msg("test_commitment".to_string());
    let result = execute(deps.as_mut(), env, info, msg);
    assert_eq!(result.unwrap_err(), ContractError::CommitNotActive);
}

#[test]
fn test_commit_vote_duplicate() {
    let (mut deps, env) = setup_test_env();
    instantiate_contract(&mut deps, &env, Scale::Tiny, BASE_AMOUNT).unwrap();
    
    // 第一次提交
    let (msg, info) = create_commit_msg("commitment1".to_string());
    execute(deps.as_mut(), env.clone(), info, msg).unwrap();
    
    // 重复提交（应该覆盖之前的承诺）
    let (msg, info) = create_commit_msg("commitment2".to_string());
    let result = execute(deps.as_mut(), env, info, msg);
    assert!(result.is_ok());
}

#[test]
fn test_reveal_vote_success() {
    let (mut deps, env) = setup_test_env();
    instantiate_contract(&mut deps, &env, Scale::Tiny, BASE_AMOUNT).unwrap();
    
    // 先提交承诺
    let reveal = "my_vote";
    let salt = "my_salt";
    let commitment = calculate_commitment(USER1, reveal, salt);
    let (msg, info) = create_commit_msg(commitment);
    execute(deps.as_mut(), env.clone(), info, msg).unwrap();
    
    // 切换到Reveal阶段
    let (msg, info) = create_set_vote_state_msg(VoteState::Reveal);
    execute(deps.as_mut(), env.clone(), info, msg).unwrap();
    
    // 揭示投票
    let (msg, info) = create_reveal_msg(reveal.to_string(), salt.to_string());
    let result = execute(deps.as_mut(), env, info, msg);
    assert!(result.is_ok());
    
    // 验证响应属性
    let res = result.unwrap();
    assert_eq!(res.attributes[0].key, "action");
    assert_eq!(res.attributes[0].value, "reveal");
    assert_eq!(res.attributes[1].key, "voter");
    assert_eq!(res.attributes[1].value, USER1);
    assert_eq!(res.attributes[2].key, "reveal");
    assert_eq!(res.attributes[2].value, reveal);
}

#[test]
fn test_reveal_vote_wrong_phase() {
    let (mut deps, env) = setup_test_env();
    instantiate_contract(&mut deps, &env, Scale::Tiny, BASE_AMOUNT).unwrap();
    
    // 先提交承诺
    let (msg, info) = create_commit_msg("test_commitment".to_string());
    execute(deps.as_mut(), env.clone(), info, msg).unwrap();
    
    // 在Commit阶段尝试揭示
    let (msg, info) = create_reveal_msg("test_reveal".to_string(), "test_salt".to_string());
    let result = execute(deps.as_mut(), env, info, msg);
    assert_eq!(result.unwrap_err(), ContractError::RevealNotActive);
}

#[test]
fn test_reveal_vote_no_commitment() {
    let (mut deps, env) = setup_test_env();
    instantiate_contract(&mut deps, &env, Scale::Tiny, BASE_AMOUNT).unwrap();
    
    // 切换到Reveal阶段
    let (msg, info) = create_set_vote_state_msg(VoteState::Reveal);
    execute(deps.as_mut(), env.clone(), info, msg).unwrap();
    
    // 尝试揭示但没有提交过承诺
    let (msg, info) = create_reveal_msg("test_reveal".to_string(), "test_salt".to_string());
    let result = execute(deps.as_mut(), env, info, msg);
    assert_eq!(result.unwrap_err(), ContractError::NothingToReveal);
}

#[test]
fn test_reveal_vote_commitment_mismatch() {
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
fn test_reveal_vote_duplicate() {
    let (mut deps, env) = setup_test_env();
    instantiate_contract(&mut deps, &env, Scale::Tiny, BASE_AMOUNT).unwrap();
    
    // 先提交承诺
    let reveal = "my_vote";
    let salt = "my_salt";
    let commitment = calculate_commitment(USER1, reveal, salt);
    let (msg, info) = create_commit_msg(commitment);
    execute(deps.as_mut(), env.clone(), info, msg).unwrap();
    
    // 切换到Reveal阶段
    let (msg, info) = create_set_vote_state_msg(VoteState::Reveal);
    execute(deps.as_mut(), env.clone(), info, msg).unwrap();
    
    // 第一次揭示
    let (msg, info) = create_reveal_msg(reveal.to_string(), salt.to_string());
    execute(deps.as_mut(), env.clone(), info, msg).unwrap();
    
    // 重复揭示（应该覆盖之前的揭示，使用相同的reveal和salt）
    let (msg, info) = create_reveal_msg(reveal.to_string(), salt.to_string());
    let result = execute(deps.as_mut(), env, info, msg);
    assert!(result.is_ok());
}

#[test]
fn test_vote_state_transition() {
    let (mut deps, env) = setup_test_env();
    instantiate_contract(&mut deps, &env, Scale::Tiny, BASE_AMOUNT).unwrap();
    
    // 初始状态应该是Commit
    let config = query_config(&deps);
    assert_eq!(config.vote_state, VoteState::Commit);
    
    // 切换到Reveal
    let (msg, info) = create_set_vote_state_msg(VoteState::Reveal);
    execute(deps.as_mut(), env.clone(), info, msg).unwrap();
    
    let config = query_config(&deps);
    assert_eq!(config.vote_state, VoteState::Reveal);
    
    // 切换到Closed
    let (msg, info) = create_set_vote_state_msg(VoteState::Closed);
    execute(deps.as_mut(), env.clone(), info, msg).unwrap();
    
    let config = query_config(&deps);
    assert_eq!(config.vote_state, VoteState::Closed);
    
    // 切换回Commit
    let (msg, info) = create_set_vote_state_msg(VoteState::Commit);
    execute(deps.as_mut(), env, info, msg).unwrap();
    
    let config = query_config(&deps);
    assert_eq!(config.vote_state, VoteState::Commit);
}

#[test]
fn test_vote_state_transition_unauthorized() {
    let (mut deps, env) = setup_test_env();
    instantiate_contract(&mut deps, &env, Scale::Tiny, BASE_AMOUNT).unwrap();
    
    // 非owner尝试切换投票状态
    let msg = ExecuteMsg::SetVoteState { state: VoteState::Reveal };
    let info = MessageInfo {
        sender: cosmwasm_std::Addr::unchecked(USER1),
        funds: vec![],
    };
    let result = execute(deps.as_mut(), env, info, msg);
    assert_eq!(result.unwrap_err(), ContractError::Unauthorized);
}

#[test]
fn test_multiple_users_voting() {
    let (mut deps, env) = setup_test_env();
    instantiate_contract(&mut deps, &env, Scale::Tiny, BASE_AMOUNT).unwrap();
    
    // 用户1提交承诺
    let reveal1 = "vote1";
    let salt1 = "salt1";
    let commitment1 = calculate_commitment(USER1, reveal1, salt1);
    let (msg, info) = create_commit_msg(commitment1);
    execute(deps.as_mut(), env.clone(), info, msg).unwrap();
    
    // 用户2提交承诺
    let reveal2 = "vote2";
    let salt2 = "salt2";
    let commitment2 = calculate_commitment(USER2, reveal2, salt2);
    let msg = ExecuteMsg::CommitVote { commitment: commitment2 };
    let info = MessageInfo {
        sender: cosmwasm_std::Addr::unchecked(USER2),
        funds: vec![],
    };
    execute(deps.as_mut(), env.clone(), info, msg).unwrap();
    
    // 切换到Reveal阶段
    let (msg, info) = create_set_vote_state_msg(VoteState::Reveal);
    execute(deps.as_mut(), env.clone(), info, msg).unwrap();
    
    // 用户1揭示
    let (msg, info) = create_reveal_msg(reveal1.to_string(), salt1.to_string());
    execute(deps.as_mut(), env.clone(), info, msg).unwrap();
    
    // 用户2揭示
    let msg = ExecuteMsg::RevealVote { reveal: reveal2.to_string(), salt: salt2.to_string() };
    let info = MessageInfo {
        sender: cosmwasm_std::Addr::unchecked(USER2),
        funds: vec![],
    };
    let result = execute(deps.as_mut(), env, info, msg);
    assert!(result.is_ok());
}

#[test]
fn test_commitment_hash_verification() {
    let (mut deps, env) = setup_test_env();
    instantiate_contract(&mut deps, &env, Scale::Tiny, BASE_AMOUNT).unwrap();
    
    // 测试不同的哈希计算
    let test_cases = vec![
        ("vote1", "salt1"),
        ("vote2", "salt2"),
        ("", ""),
        ("long_vote_string", "long_salt_string"),
        ("special!@#$%^&*()", "special!@#$%^&*()"),
    ];
    
    for (reveal, salt) in test_cases {
        // 提交承诺
        let commitment = calculate_commitment(USER1, reveal, salt);
        let (msg, info) = create_commit_msg(commitment);
        execute(deps.as_mut(), env.clone(), info, msg).unwrap();
        
        // 切换到Reveal阶段
        let (msg, info) = create_set_vote_state_msg(VoteState::Reveal);
        execute(deps.as_mut(), env.clone(), info, msg).unwrap();
        
        // 揭示投票
        let (msg, info) = create_reveal_msg(reveal.to_string(), salt.to_string());
        let result = execute(deps.as_mut(), env.clone(), info, msg);
        assert!(result.is_ok(), "Failed for reveal: '{}', salt: '{}'", reveal, salt);
        
        // 切换回Commit阶段
        let (msg, info) = create_set_vote_state_msg(VoteState::Commit);
        execute(deps.as_mut(), env.clone(), info, msg).unwrap();
    }
}
