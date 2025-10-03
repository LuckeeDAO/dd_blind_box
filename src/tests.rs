  #[cfg(test)]
mod tests {
    // 单元测试：验证实例化与充值后按倍数铸造 NFT 的逻辑
    use cosmwasm_std::{coins, testing::{mock_dependencies, mock_env}, MessageInfo};
    use crate::{contract::{execute, instantiate}, msg::{ExecuteMsg, InstantiateMsg}, state::Scale};

    #[test]
    fn instantiate_and_deposit_mints() {
        let mut deps = mock_dependencies();
        let env = mock_env();
        let info = MessageInfo { sender: cosmwasm_std::Addr::unchecked("owner"), funds: vec![] };
        instantiate(deps.as_mut(), env.clone(), info.clone(), InstantiateMsg { scale: Scale::Tiny, base: coins(100, "ujunox")[0].clone(), first_prize_count: None }).unwrap();

        // 设置NFT合约地址（使用unchecked地址避免验证问题）
        execute(deps.as_mut(), env.clone(), info.clone(), ExecuteMsg::SetNftContract { 
            nft_contract: cosmwasm_std::Addr::unchecked("nft_contract").to_string()
        }).unwrap();

        let info_user = MessageInfo { sender: cosmwasm_std::Addr::unchecked("user"), funds: coins(250, "ujunox") };
        let res = execute(deps.as_mut(), env, info_user, ExecuteMsg::Deposit {}).unwrap();
        assert_eq!(res.attributes.iter().find(|a| a.key == "minted").unwrap().value, "2");
    }

    #[test]
    fn test_nft_code_id_setting() {
        let mut deps = mock_dependencies();
        let env = mock_env();
        let info = MessageInfo { sender: cosmwasm_std::Addr::unchecked("owner"), funds: vec![] };
        
        // 实例化盲盒合约
        instantiate(deps.as_mut(), env.clone(), info.clone(), InstantiateMsg { 
            scale: Scale::Tiny, 
            base: coins(100, "ujunox")[0].clone(), 
            first_prize_count: None 
        }).unwrap();

        // 设置NFT合约代码ID
        let res = execute(deps.as_mut(), env.clone(), info.clone(), ExecuteMsg::SetNftCodeId { 
            code_id: 123 
        }).unwrap();
        
        assert_eq!(res.attributes.iter().find(|a| a.key == "action").unwrap().value, "set_nft_code_id");
        assert_eq!(res.attributes.iter().find(|a| a.key == "code_id").unwrap().value, "123");
    }

    #[test]
    fn test_nft_contract_instantiation_preparation() {
        let mut deps = mock_dependencies();
        let env = mock_env();
        let info = MessageInfo { sender: cosmwasm_std::Addr::unchecked("owner"), funds: vec![] };
        
        // 实例化盲盒合约
        instantiate(deps.as_mut(), env.clone(), info.clone(), InstantiateMsg { 
            scale: Scale::Tiny, 
            base: coins(100, "ujunox")[0].clone(), 
            first_prize_count: None 
        }).unwrap();

        // 设置NFT合约代码ID
        execute(deps.as_mut(), env.clone(), info.clone(), ExecuteMsg::SetNftCodeId { 
            code_id: 123 
        }).unwrap();

        // 尝试实例化NFT合约（这会创建子消息）
        let res = execute(deps.as_mut(), env.clone(), info.clone(), ExecuteMsg::InstantiateNftContract {
            name: "Test NFT".to_string(),
            symbol: "TNFT".to_string(),
            base_uri: Some("https://example.com/metadata/".to_string()),
        }).unwrap();
        
        // 验证返回了子消息
        assert_eq!(res.messages.len(), 1);
        assert_eq!(res.attributes.iter().find(|a| a.key == "action").unwrap().value, "instantiate_nft_contract");
        assert_eq!(res.attributes.iter().find(|a| a.key == "name").unwrap().value, "Test NFT");
        assert_eq!(res.attributes.iter().find(|a| a.key == "symbol").unwrap().value, "TNFT");
    }
}

// 集成测试模块
#[cfg(test)]
mod integration_tests {
    include!("../tests/mod.rs");
}


