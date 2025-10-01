mod common;

use cosmwasm_std::{coins, MessageInfo};
use dd_blind_box::{
    contract::execute,
    error::ContractError,
    msg::ExecuteMsg,
    state::Scale,
};
use common::*;


#[test]
fn test_deposit_success() {
    let (mut deps, env) = setup_test_env();
    instantiate_contract(&mut deps, &env, Scale::Tiny, BASE_AMOUNT).unwrap();
    
    // 正常充值
    let (msg, info) = create_deposit_msg(BASE_AMOUNT);
    let result = execute(deps.as_mut(), env, info, msg);
    assert!(result.is_ok());
    
    // 验证NFT铸造数量
    let res = result.unwrap();
    let minted_attr = res.attributes.iter().find(|a| a.key == "minted").unwrap();
    assert_eq!(minted_attr.value, "1");
    
    // 验证充值记录
    let deposit = query_deposit_test(&deps, USER1);
    assert_eq!(deposit.principal, BASE_AMOUNT.to_string());
}

#[test]
fn test_deposit_multiple_base_amounts() {
    let (mut deps, env) = setup_test_env();
    instantiate_contract(&mut deps, &env, Scale::Tiny, BASE_AMOUNT).unwrap();
    
    // 充值3倍基础金额
    let (msg, info) = create_deposit_msg(BASE_AMOUNT * 3);
    let result = execute(deps.as_mut(), env, info, msg);
    assert!(result.is_ok());
    
    // 验证铸造了3个NFT
    let res = result.unwrap();
    let minted_attr = res.attributes.iter().find(|a| a.key == "minted").unwrap();
    assert_eq!(minted_attr.value, "3");
    
    // 验证充值记录
    let deposit = query_deposit_test(&deps, USER1);
    assert_eq!(deposit.principal, (BASE_AMOUNT * 3).to_string());
}

#[test]
fn test_deposit_insufficient_amount() {
    let (mut deps, env) = setup_test_env();
    instantiate_contract(&mut deps, &env, Scale::Tiny, BASE_AMOUNT).unwrap();
    
    // 充值金额不足
    let (msg, info) = create_deposit_msg(BASE_AMOUNT - 1);
    let result = execute(deps.as_mut(), env, info, msg);
    assert!(result.is_err());
    
    // 验证错误类型
    let err = result.unwrap_err();
    assert!(matches!(err, ContractError::Std(_)));
}

#[test]
fn test_deposit_zero_amount() {
    let (mut deps, env) = setup_test_env();
    instantiate_contract(&mut deps, &env, Scale::Tiny, BASE_AMOUNT).unwrap();
    
    // 充值零金额
    let (msg, info) = create_deposit_msg(0);
    let result = execute(deps.as_mut(), env, info, msg);
    assert!(result.is_err());
}

#[test]
fn test_deposit_wrong_denom() {
    let (mut deps, env) = setup_test_env();
    instantiate_contract(&mut deps, &env, Scale::Tiny, BASE_AMOUNT).unwrap();
    
    // 使用错误的币种
    let msg = ExecuteMsg::Deposit {};
    let info = MessageInfo {
        sender: cosmwasm_std::Addr::unchecked(USER1),
        funds: coins(BASE_AMOUNT, "uatom"),
    };
    let result = execute(deps.as_mut(), env, info, msg);
    assert!(result.is_err());
}

#[test]
fn test_deposit_accumulation() {
    let (mut deps, env) = setup_test_env();
    instantiate_contract(&mut deps, &env, Scale::Tiny, BASE_AMOUNT).unwrap();
    
    // 第一次充值
    let (msg, info) = create_deposit_msg(BASE_AMOUNT);
    execute(deps.as_mut(), env.clone(), info, msg).unwrap();
    
    // 第二次充值
    let (msg, info) = create_deposit_msg(BASE_AMOUNT * 2);
    execute(deps.as_mut(), env, info, msg).unwrap();
    
    // 验证累计充值
    let deposit = query_deposit_test(&deps, USER1);
    assert_eq!(deposit.principal, (BASE_AMOUNT * 3).to_string());
}

#[test]
fn test_deposit_exceeds_total_supply() {
    let (mut deps, env) = setup_test_env();
    instantiate_contract(&mut deps, &env, Scale::Tiny, BASE_AMOUNT).unwrap(); // total_supply = 10
    
    // 尝试充值超过总供应量的NFT
    let (msg, info) = create_deposit_msg(BASE_AMOUNT * 15); // 尝试铸造15个NFT
    let result = execute(deps.as_mut(), env, info, msg);
    assert!(result.is_ok());
    
    // 验证只铸造了10个NFT（总供应量）
    let res = result.unwrap();
    let minted_attr = res.attributes.iter().find(|a| a.key == "minted").unwrap();
    assert_eq!(minted_attr.value, "10");
}

#[test]
fn test_deposit_sequential_token_ids() {
    let (mut deps, env) = setup_test_env();
    instantiate_contract(&mut deps, &env, Scale::Tiny, BASE_AMOUNT).unwrap();
    
    // 用户1充值
    let (msg, info) = create_deposit_msg(BASE_AMOUNT);
    execute(deps.as_mut(), env.clone(), info, msg).unwrap();
    
    // 用户2充值
    let msg = ExecuteMsg::Deposit {};
    let info = MessageInfo {
        sender: cosmwasm_std::Addr::unchecked(USER2),
        funds: coins(BASE_AMOUNT, BASE_DENOM),
    };
    execute(deps.as_mut(), env.clone(), info, msg).unwrap();
    
    // 用户3充值
    let msg = ExecuteMsg::Deposit {};
    let info = MessageInfo {
        sender: cosmwasm_std::Addr::unchecked(USER3),
        funds: coins(BASE_AMOUNT, BASE_DENOM),
    };
    execute(deps.as_mut(), env, info, msg).unwrap();
    
    // 验证NFT所有者
    let owner1 = query_owner_of(&deps, 0);
    assert_eq!(owner1.owner, USER1);
    
    let owner2 = query_owner_of(&deps, 1);
    assert_eq!(owner2.owner, USER2);
    
    let owner3 = query_owner_of(&deps, 2);
    assert_eq!(owner3.owner, USER3);
}

#[test]
fn test_deposit_paused() {
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
fn test_deposit_query_nonexistent_user() {
    let (mut deps, env) = setup_test_env();
    instantiate_contract(&mut deps, &env, Scale::Tiny, BASE_AMOUNT).unwrap();
    
    // 查询未充值用户的充值记录
    let deposit = query_deposit_test(&deps, USER1);
    assert_eq!(deposit.principal, "0");
}

#[test]
fn test_tier_query_before_finalize() {
    let (mut deps, env) = setup_test_env();
    instantiate_contract(&mut deps, &env, Scale::Tiny, BASE_AMOUNT).unwrap();
    
    // 充值
    let (msg, info) = create_deposit_msg(BASE_AMOUNT);
    execute(deps.as_mut(), env, info, msg).unwrap();
    
    // 查询分层（结算前应该为0）
    let tier = query_tier_test(&deps, USER1);
    assert_eq!(tier.tier, 0);
}

#[test]
fn test_tier_query_nonexistent_user() {
    let (mut deps, env) = setup_test_env();
    instantiate_contract(&mut deps, &env, Scale::Tiny, BASE_AMOUNT).unwrap();
    
    // 查询未参与用户的分层
    let tier = query_tier_test(&deps, USER1);
    assert_eq!(tier.tier, 0);
}

#[test]
fn test_deposit_fractional_base_amount() {
    let (mut deps, env) = setup_test_env();
    instantiate_contract(&mut deps, &env, Scale::Tiny, BASE_AMOUNT).unwrap();
    
    // 充值1.5倍基础金额
    let (msg, info) = create_deposit_msg(BASE_AMOUNT + BASE_AMOUNT / 2);
    let result = execute(deps.as_mut(), env, info, msg);
    assert!(result.is_ok());
    
    // 验证只铸造了1个NFT（向下取整）
    let res = result.unwrap();
    let minted_attr = res.attributes.iter().find(|a| a.key == "minted").unwrap();
    assert_eq!(minted_attr.value, "1");
    
    // 验证充值记录包含完整金额
    let deposit = query_deposit_test(&deps, USER1);
    assert_eq!(deposit.principal, (BASE_AMOUNT + BASE_AMOUNT / 2).to_string());
}

#[test]
fn test_deposit_large_scale() {
    let (mut deps, env) = setup_test_env();
    instantiate_contract(&mut deps, &env, Scale::Large, BASE_AMOUNT).unwrap(); // total_supply = 10,000
    
    // 充值大量金额
    let (msg, info) = create_deposit_msg(BASE_AMOUNT * 100);
    let result = execute(deps.as_mut(), env, info, msg);
    assert!(result.is_ok());
    
    // 验证铸造了100个NFT
    let res = result.unwrap();
    let minted_attr = res.attributes.iter().find(|a| a.key == "minted").unwrap();
    assert_eq!(minted_attr.value, "100");
}
