use crate::{
    error::ContractError,
    msg::{ExecuteMsg, InstantiateMsg, MigrateMsg, StrategyExecuteMsg},
    state::{MINTER, OWNER, STRATEGY_MANAGER},
};

use cosmwasm_std::{
    entry_point, to_json_binary, Addr, CosmosMsg, Deps, DepsMut, Env, Event, MessageInfo, Response,
    Uint128, WasmMsg,
};

use cw2::set_contract_version;
use cw20::Cw20ExecuteMsg;

const CONTRACT_NAME: &str = env!("CARGO_PKG_NAME");
const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    deps: DepsMut,
    _env: Env,
    _info: MessageInfo,
    msg: InstantiateMsg,
) -> Result<Response, ContractError> {
    let strategy_manager = deps.api.addr_validate(&msg.strategy_manager)?;
    let minter = deps.api.addr_validate(&msg.minter)?;
    let owner = deps.api.addr_validate(&msg.owner)?;

    MINTER.save(deps.storage, &minter)?;
    OWNER.save(deps.storage, &owner)?;
    STRATEGY_MANAGER.save(deps.storage, &strategy_manager)?;

    Ok(Response::new()
        .add_attribute("action", "instantiate")
        .add_attribute("minter", minter)
        .add_attribute("owner", owner))
}

fn only_minter(deps: Deps, info: &MessageInfo) -> Result<(), ContractError> {
    let minter = MINTER.load(deps.storage)?;
    if info.sender != minter {
        return Err(ContractError::NotMinterUnauthorized {});
    }
    Ok(())
}

fn only_owner(deps: Deps, info: &MessageInfo) -> Result<(), ContractError> {
    let owner = OWNER.load(deps.storage)?;
    if info.sender != owner {
        return Err(ContractError::NotOwnerUnauthorized {});
    }
    Ok(())
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn execute(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: ExecuteMsg,
) -> Result<Response, ContractError> {
    match msg {
        ExecuteMsg::DepositWithMintAndStrategy {
            token,
            strategy,
            recipient,
            amount,
            public_key,
            expiry,
            signature,
        } => execute_deposit_with_mint_and_strategy(
            deps, env, info, token, strategy, recipient, amount, public_key, expiry, signature,
        ),
        ExecuteMsg::SetMinter { minter } => {
            let minter_addr = deps.api.addr_validate(&minter)?;
            set_minter(deps, info, minter_addr)
        }
        ExecuteMsg::SetStrategyManager { strategy_manager } => {
            let strategy_manager_addr = deps.api.addr_validate(&strategy_manager)?;
            set_strategy_manager(deps, info, strategy_manager_addr)
        }
        ExecuteMsg::TransferOwnership { new_owner } => {
            let new_owner_addr = deps.api.addr_validate(&new_owner)?;
            transfer_ownership(deps, info, new_owner_addr)
        }
    }
}

fn execute_deposit_with_mint_and_strategy(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    token: String,
    strategy: String,
    recipient: String,
    amount: Uint128,
    public_key: String,
    expiry: u64,
    signature: String,
) -> Result<Response, ContractError> {
    only_minter(deps.as_ref(), &info)?;

    let token_addr = deps.api.addr_validate(&token)?;
    let strategy_addr = deps.api.addr_validate(&strategy)?;
    let recipient_addr = deps.api.addr_validate(&recipient)?;

    // Step 1: Mint tokens to this contract instead of recipient
    let mint_msg = CosmosMsg::Wasm(WasmMsg::Execute {
        contract_addr: token_addr.to_string(),
        msg: to_json_binary(&Cw20ExecuteMsg::Mint {
            recipient: env.contract.address.to_string(),
            amount,
        })?,
        funds: vec![],
    });

    let strategy_manager = STRATEGY_MANAGER.load(deps.storage)?;

    // Step 2: Approve the strategy to spend tokens
    let approve_msg = CosmosMsg::Wasm(WasmMsg::Execute {
        contract_addr: token_addr.to_string(),
        msg: to_json_binary(&Cw20ExecuteMsg::IncreaseAllowance {
            spender: strategy_manager.to_string(),
            amount,
            expires: None,
        })?,
        funds: vec![],
    });

    // Step 3: Deposit tokens into strategy
    let deposit_msg = CosmosMsg::Wasm(WasmMsg::Execute {
        contract_addr: strategy_manager.to_string(),
        msg: to_json_binary(&StrategyExecuteMsg::DepositViaMirroredTokenWithSignature {
            strategy: strategy_addr.to_string(),
            token: token_addr.to_string(),
            amount,
            staker: recipient_addr.to_string(),
            public_key,
            expiry,
            signature,
        })?,
        funds: vec![],
    });

    Ok(Response::new()
        .add_message(mint_msg)
        .add_message(approve_msg)
        .add_message(deposit_msg)
        .add_attribute("action", "deposit_with_mint_and_strategy")
        .add_attribute("token", token_addr)
        .add_attribute("strategy", strategy_addr)
        .add_attribute("recipient", recipient_addr)
        .add_attribute("amount", amount))
}

fn set_strategy_manager(
    deps: DepsMut,
    info: MessageInfo,
    strategy_manager: Addr,
) -> Result<Response, ContractError> {
    only_owner(deps.as_ref(), &info)?;

    STRATEGY_MANAGER.save(deps.storage, &strategy_manager)?;

    let event = Event::new("set_strategy_manager")
        .add_attribute("old_strategy_manager", strategy_manager.to_string())
        .add_attribute("new_strategy_manager", strategy_manager.to_string());

    Ok(Response::new().add_event(event))
}

fn set_minter(deps: DepsMut, info: MessageInfo, minter: Addr) -> Result<Response, ContractError> {
    only_owner(deps.as_ref(), &info)?;

    MINTER.save(deps.storage, &minter)?;

    Ok(Response::new())
}

fn transfer_ownership(
    deps: DepsMut,
    info: MessageInfo,
    new_owner: Addr,
) -> Result<Response, ContractError> {
    only_owner(deps.as_ref(), &info)?;

    OWNER.save(deps.storage, &new_owner)?;

    Ok(Response::new())
}

pub fn migrate(
    deps: DepsMut,
    _env: Env,
    info: &MessageInfo,
    _msg: MigrateMsg,
) -> Result<Response, ContractError> {
    only_owner(deps.as_ref(), info)?;

    set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;

    Ok(Response::new().add_attribute("method", "migrate"))
}
