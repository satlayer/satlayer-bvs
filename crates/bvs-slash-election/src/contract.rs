#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;

use crate::{
    error::ContractError,
    msg::{ExecuteMsg, ExecuteSlashMsg, InstantiateMsg, QueryMsg},
    state::{Offense, REGISTRY, THRESHOLD},
};
use bvs_library::ownership;
use cosmwasm_std::{Deps, DepsMut, Env, MessageInfo, Response};

const CONTRACT_NAME: &str = concat!("crates.io:", env!("CARGO_PKG_NAME"));
const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    deps: DepsMut,
    _env: Env,
    _info: MessageInfo,
    msg: InstantiateMsg,
) -> Result<Response, ContractError> {
    cw2::set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;

    let owner = deps.api.addr_validate(&msg.owner)?;
    let pauser = deps.api.addr_validate(&msg.pauser)?;
    let registry = deps.api.addr_validate(&msg.registry)?;

    ownership::set_owner(deps.storage, &owner)?;

    let _ = bvs_pauser::api::set_pauser(deps.storage, &pauser);

    THRESHOLD.save(deps.storage, &msg.threshold)?;

    REGISTRY.save(deps.storage, &registry)?;

    Ok(Response::default())
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn execute(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: ExecuteMsg,
) -> Result<Response, ContractError> {
    match msg {
        ExecuteMsg::SubmitSlash(msg) => {
            let offender = deps.api.addr_validate(&msg.offender)?;
            execute::submit_slash(deps, env, info, offender, msg.offense)
        }
        ExecuteMsg::VoteSlash(msg) => {
            let offender = deps.api.addr_validate(&msg.slash.offender)?;
            let offense = Offense::try_from(msg.slash.offense.as_str())?;
            let start_height = msg.slash.start_height;
            execute::vote_slash(
                deps,
                env,
                info,
                offender,
                offense,
                start_height,
                msg.approve,
            )
        }
        ExecuteMsg::ExecuteSlash(msg) => execute::execute_slash(deps, env, info, msg),
        ExecuteMsg::SetPunishment(msg) => execute::set_punishment(deps, env, info, msg),
        ExecuteMsg::SetThreshold(threshold) => execute::set_threshold(deps, env, info, threshold),
    }
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(
    deps: Deps,
    _env: Env,
    _info: MessageInfo,
    msg: QueryMsg,
) -> Result<Response, ContractError> {
    Ok(Response::default())
}

mod execute {

    use cosmwasm_std::Addr;

    use super::*;
    use crate::{
        auth,
        msg::{SetPunishmentMsg, SlashDetails, VoteSlashMsg},
        state::{Offense, SLASHES},
    };

    pub fn submit_slash(
        deps: DepsMut,
        env: Env,
        info: MessageInfo,
        offender: Addr,
        offense: Offense,
    ) -> Result<Response, ContractError> {
        auth::assert_operator(deps.as_ref(), &info)?;

        SLASHES.save(
            deps.storage,
            (&offender.clone(), &offense.as_str(), env.block.height),
            &0u64,
        )?;

        Ok(Response::new()
            .add_attribute("action", "submit_slash")
            .add_attribute("accuser", info.sender)
            .add_attribute("offender", offender)
            .add_attribute("offense", offense.as_str())
            .add_attribute("block_height", env.block.height.to_string()))
    }

    pub fn vote_slash(
        deps: DepsMut,
        env: Env,
        info: MessageInfo,
        offender: Addr,
        offense: Offense,
        start_height: u64,
        approve: bool,
    ) -> Result<Response, ContractError> {
        auth::assert_operator(deps.as_ref(), &info)?;
        auth::assert_voting_period(env, start_height)?;

        let slash = SLASHES.load(
            deps.storage,
            (&offender.clone(), &offense.as_str(), start_height),
        )?;

        todo!("Check if the operator already voted");
    }

    pub fn execute_slash(
        deps: DepsMut,
        env: Env,
        info: MessageInfo,
        msg: ExecuteSlashMsg,
    ) -> Result<Response, ContractError> {
        todo!()
    }

    pub fn set_punishment(
        deps: DepsMut,
        env: Env,
        info: MessageInfo,
        msg: SetPunishmentMsg,
    ) -> Result<Response, ContractError> {
        Ok(Response::default())
    }

    pub fn set_threshold(
        deps: DepsMut,
        env: Env,
        info: MessageInfo,
        threshold: u64,
    ) -> Result<Response, ContractError> {
        ownership::assert_owner(deps.storage, &info)?;

        Ok(Response::default())
    }
}

mod query {}
