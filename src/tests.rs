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
        instantiate(deps.as_mut(), env.clone(), info, InstantiateMsg { scale: Scale::new_tiny(), base: coins(100, "ujunox")[0].clone() }).unwrap();

        let info_user = MessageInfo { sender: cosmwasm_std::Addr::unchecked("user"), funds: coins(250, "ujunox") };
        let res = execute(deps.as_mut(), env, info_user, ExecuteMsg::Deposit {}).unwrap();
        assert_eq!(res.attributes.iter().find(|a| a.key == "minted").unwrap().value, "2");
    }
}

// 集成测试模块
#[cfg(test)]
mod integration_tests {
    include!("../tests/mod.rs");
}


