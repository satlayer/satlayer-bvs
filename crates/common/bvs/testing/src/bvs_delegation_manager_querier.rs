use std::collections::HashMap;

use bvs_delegation_manager::msg::QueryMsg;
use cosmwasm_std::{to_json_binary, Binary, ContractResult, QuerierResult};

#[derive(Default)]
pub struct BvsDelegationManagerQuerier {
    pub is_operator: HashMap<String, bool>,
}

impl BvsDelegationManagerQuerier {
    pub fn handle_query(&self, query: QueryMsg) -> QuerierResult {
        let ret: ContractResult<Binary> = match query {
            QueryMsg::IsOperator { operator } => match self.is_operator.get(&operator) {
                Some(is_operator) => to_json_binary(&is_operator).into(),
                None => Err(format!(
                    "[mock]: could not find the operator for {operator}"
                ))
                .into(),
            },
            _ => Err("[mock]: Unsupported params query".to_string()).into(),
        };
        Ok(ret).into()
    }
}
