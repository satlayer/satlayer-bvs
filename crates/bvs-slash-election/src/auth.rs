use cosmwasm_std::{to_json_binary, Deps, Env, MessageInfo, QueryRequest, WasmQuery};

use crate::{error::ContractError, state::REGISTRY};

pub fn assert_operator(deps: Deps, info: &MessageInfo) -> Result<(), ContractError> {
    let msg = bvs_registry::msg::QueryMsg::IsOperator(info.sender.to_string());

    let query = WasmQuery::Smart {
        contract_addr: REGISTRY.load(deps.storage)?.to_string(),
        msg: to_json_binary(&msg)?,
    };

    let bvs_registry::msg::IsOperatorResponse(is_operator) =
        deps.querier.query(&QueryRequest::Wasm(query))?;

    if !is_operator {
        Err(ContractError::Unauthorized {})
    } else {
        Ok(())
    }
}

pub fn assert_voting_period(env: Env, start_height: u64) -> Result<(), ContractError> {
    let current_height = env.block.height;

    if current_height > start_height + 100 {
        Err(ContractError::VotingPeriodExpired {})
    } else {
        Ok(())
    }
}
