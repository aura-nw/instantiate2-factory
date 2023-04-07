use cosmwasm_std::entry_point;

use cosmwasm_std::{
    instantiate2_address, Binary, CodeInfoResponse, DepsMut, Env, MessageInfo, Response,
    WasmMsg,
};

use crate::errors::ContractError;
use crate::msg::{ExecuteMsg, InstantiateMsg};
use crate::state::{Config, config, config_read};

#[entry_point]
pub fn instantiate(
    deps: DepsMut,
    _env: Env,
    _info: MessageInfo,
    msg: InstantiateMsg,
) -> Result<Response, ContractError> {
    let factory_owner_addr = deps.api.addr_validate(msg.factory_owner.as_str())?;
    let state = Config {
        factory_owner: factory_owner_addr.clone(),
        contract_code_id: msg.contract_code_id,
    };
    config(deps.storage).save(&state)?;
    
    Ok(Response::new()
        .add_attribute("method", "instantiate")
        .add_attribute("factory_owner", factory_owner_addr.to_string())
        .add_attribute("contract_code_id", msg.contract_code_id.to_string()))
}

#[entry_point]
pub fn execute(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: ExecuteMsg,
) -> Result<Response, ContractError> {
    match msg {
        ExecuteMsg::Deploy { input_salt } => execute_deploy(deps, env, info, input_salt),
    }
}

pub fn execute_deploy(
    deps: DepsMut,
    env: Env,
    _info: MessageInfo,
    input_salt: String,
) -> Result<Response, ContractError> {
    let config = config_read(deps.storage).load()?;
    let creator = deps.api.addr_canonicalize(env.contract.address.as_str())?;
    let CodeInfoResponse { checksum, .. } = deps.querier.query_wasm_code_info(config.contract_code_id)?;
    
    let mut msgs = Vec::<WasmMsg>::new();
    let label = "Instance".to_string();
    let salt = Binary::from(input_salt.as_bytes());
    let address = deps
        .api
        .addr_humanize(&instantiate2_address(&checksum, &creator, &salt)?)?;

    msgs.push(WasmMsg::Instantiate2 {
        admin: None,
        code_id: config.contract_code_id,
        label,
        msg: b"".into(),
        funds: vec![],
        salt,
    });
    Ok(Response::new()
        .add_attribute("method", "instantiate2")
        .add_attribute("contract_code_id", config.contract_code_id.to_string())
        .add_attribute("predicted_address", address)
        .add_messages(msgs))
}
#[cfg(test)]
mod tests {
    use super::*;
    use cosmwasm_std::testing::{mock_dependencies, mock_env, mock_info};

    const CREATOR: &str = "creator";
    const CODE_ID: u64 = 1;

    #[test]
    fn instantiate_works() {
        let mut deps = mock_dependencies();
        let msg = InstantiateMsg {
            factory_owner: CREATOR.to_string(),
            contract_code_id: CODE_ID,
        };
        let info = mock_info(CREATOR, &[]);
        let res = instantiate(deps.as_mut(), mock_env(), info, msg).unwrap();
        println!("{:?}", &res);
        assert_eq!(3, res.attributes.len());
    }
}