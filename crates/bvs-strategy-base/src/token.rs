use cosmwasm_std::{
    to_json_binary, Addr, Deps, Env, QueryRequest, StdResult, Storage, Uint128, WasmQuery,
};
use cw20::{BalanceResponse, Cw20QueryMsg, TokenInfoResponse};
use cw_storage_plus::Item;

const CW20_TOKEN: Item<Addr> = Item::new("cw20_token");

/// Set the underlying token of the contract during instantiation
/// This is internal, no checks are done
pub fn set_cw20_token(storage: &mut dyn Storage, underlying_token: &Addr) -> StdResult<()> {
    CW20_TOKEN.save(storage, underlying_token)
}

/// Get the underlying token of the contract from storage
pub fn get_c20_token(storage: &dyn Storage) -> StdResult<Addr> {
    CW20_TOKEN.load(storage)
}

/// Get the token info of the underlying token
pub fn get_token_info(deps: &Deps) -> StdResult<TokenInfoResponse> {
    let token_addr = CW20_TOKEN.load(deps.storage)?;
    deps.querier.query(
        &WasmQuery::Smart {
            contract_addr: token_addr.to_string(),
            msg: to_json_binary(&Cw20QueryMsg::TokenInfo {})?,
        }
        .into(),
    )
}

/// Get the underlying token balance of the contract
pub fn get_balance(deps: &Deps, env: &Env) -> StdResult<Uint128> {
    let token_addr = CW20_TOKEN.load(deps.storage)?;
    let address = env.contract.address.to_string();

    let query = WasmQuery::Smart {
        contract_addr: token_addr.to_string(),
        msg: to_json_binary(&Cw20QueryMsg::Balance { address })?,
    };

    let res: BalanceResponse = deps.querier.query(&QueryRequest::Wasm(query))?;
    Ok(res.balance)
}

/// New transfer (sub_message) to recipient
pub fn new_transfer(
    storage: &dyn Storage,
    recipient: &Addr,
    amount: Uint128,
) -> StdResult<cosmwasm_std::CosmosMsg> {
    let token_addr = CW20_TOKEN.load(storage)?;

    Ok(cosmwasm_std::WasmMsg::Execute {
        contract_addr: token_addr.to_string(),
        msg: to_json_binary(&cw20::Cw20ExecuteMsg::Transfer {
            recipient: recipient.to_string(),
            amount,
        })?,
        funds: vec![],
    }
    .into())
}

/// New transfer_from (sub_message) from owner to recipient
#[allow(dead_code)]
pub fn new_transfer_from(
    storage: &dyn Storage,
    owner: &Addr,
    recipient: &Addr,
    amount: Uint128,
) -> StdResult<cosmwasm_std::CosmosMsg> {
    let underlying_token = CW20_TOKEN.load(storage)?;

    Ok(cosmwasm_std::WasmMsg::Execute {
        contract_addr: underlying_token.to_string(),
        msg: to_json_binary(&cw20::Cw20ExecuteMsg::TransferFrom {
            owner: owner.to_string(),
            recipient: recipient.to_string(),
            amount,
        })?,
        funds: vec![],
    }
    .into())
}
