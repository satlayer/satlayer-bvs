use crate::msg;
use crate::msg::VaultType;
use crate::state;
/// This file contains the migration logic for the vault factory contract.
/// Since this is migration related code, it is expected to change often between major versions.
use bvs_vault_router;
use cosmwasm_std::{to_json_binary, CosmosMsg, WasmMsg};
use cosmwasm_std::{Addr, ContractInfoResponse, QueryRequest, WasmQuery};

pub(crate) fn build_vault_migrate_msgs(
    deps: cosmwasm_std::DepsMut,
    migrate_msg: msg::MigrateMsg,
) -> Result<Vec<CosmosMsg>, cosmwasm_std::StdError> {
    let mut start_after: Option<String> = None;
    let mut atomic_migrate_msgs = Vec::new();
    loop {
        let msg = bvs_vault_router::msg::QueryMsg::ListVaults {
            start_after: start_after.clone(),
            limit: Some(100),
        };
        let vaults: bvs_vault_router::msg::VaultListResponse = deps
            .querier
            .query_wasm_smart(state::ROUTER.load(deps.storage)?, &msg)?;

        start_after = vaults.0.last().map(|v| v.vault.to_string());

        if vaults.0.is_empty() || vaults.0.len() < 100 {
            break;
        }

        for vault in vaults.0 {
            let vault_code_id = get_current_code_id(deps.as_ref(), vault.vault.clone())?;
            let vault_type = get_vault_type_by_code_id(deps.as_ref(), vault_code_id)?;

            match vault_type {
                msg::VaultType::Bank => {
                    let vault_migrate_msg: CosmosMsg = WasmMsg::Migrate {
                        contract_addr: vault.vault.to_string(),
                        new_code_id: migrate_msg.new_bank_vault_code_id,
                        msg: to_json_binary(&bvs_vault_base::msg::MigrateMsg {})?,
                    }
                    .into();
                    atomic_migrate_msgs.push(vault_migrate_msg);
                }
                msg::VaultType::Cw20 => {
                    let vault_migrate_msg: CosmosMsg = WasmMsg::Migrate {
                        contract_addr: vault.vault.to_string(),
                        new_code_id: migrate_msg.new_cw20_vault_code_id,
                        msg: to_json_binary(&bvs_vault_base::msg::MigrateMsg {})?,
                    }
                    .into();
                    atomic_migrate_msgs.push(vault_migrate_msg);
                }
                _ => {
                    // This should not happen. Only bank and cw20 non-tokenized vaults are live.
                    // This mean that vault that are not manufactured by the factory are present in
                    // the vaults list.
                }
            }
        }
    }

    Ok(atomic_migrate_msgs)
}

pub(crate) fn get_current_code_id(
    deps: cosmwasm_std::Deps,
    vault: Addr,
) -> Result<u64, cosmwasm_std::StdError> {
    let msg = QueryRequest::Wasm(WasmQuery::ContractInfo {
        contract_addr: vault.to_string(),
    });

    let contract_info: ContractInfoResponse = deps.querier.query(&msg)?;

    Ok(contract_info.code_id)
}

pub(crate) fn get_vault_type_by_code_id(
    deps: cosmwasm_std::Deps,
    code_id: u64,
) -> Result<msg::VaultType, cosmwasm_std::StdError> {
    for variant in VaultType::all_variants().iter() {
        let true_code_id = state::get_code_id(deps.storage, variant).map_err(|_| {
            cosmwasm_std::StdError::generic_err("Failed to get code id for vault type")
        })?;

        if true_code_id == code_id {
            return Ok(variant.clone());
        }
    }

    return Err(cosmwasm_std::StdError::generic_err(
        "No vault type found for the given code id",
    ));
}
