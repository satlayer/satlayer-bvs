use cosmwasm_std::{to_json_binary, Deps, MessageInfo, QueryRequest, WasmQuery};

use crate::{
    error::ContractError,
    state::{CODE_IDS, REGISTRY},
};

pub fn assert_operator(deps: Deps, info: &MessageInfo) -> Result<(), ContractError> {
    let msg = bvs_registry::msg::QueryMsg::IsOperator(info.sender.to_string());

    let query = WasmQuery::Smart {
        contract_addr: REGISTRY.load(deps.storage)?.to_string(),
        msg: to_json_binary(&msg)?,
    };

    let is_operator: bvs_registry::msg::IsOperatorResponse =
        deps.querier.query(&QueryRequest::Wasm(query))?;

    if !is_operator.0 {
        return Err(crate::error::ContractError::Unauthorized {});
    } else {
        return Ok(());
    }
}

pub fn assert_code_id(deps: Deps, code_id: u64) -> Result<(), ContractError> {
    match CODE_IDS.has(deps.storage, code_id) {
        true => Ok(()),
        false => Err(ContractError::InvalidCodeId {}),
    }
}
