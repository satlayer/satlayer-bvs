#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;

use crate::error::RewardsError;
use crate::msg::{ExecuteMsg, InstantiateMsg, QueryMsg, RewardsType};
use bvs_library::ownership;
use cosmwasm_std::{to_json_binary, Binary, Deps, DepsMut, Env, MessageInfo, Response, StdResult};
use cw2::set_contract_version;

const CONTRACT_NAME: &str = concat!("crates.io:", env!("CARGO_PKG_NAME"));
const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    deps: DepsMut,
    _env: Env,
    _info: MessageInfo,
    msg: InstantiateMsg,
) -> Result<Response, RewardsError> {
    set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;

    let owner = deps.api.addr_validate(&msg.owner)?;
    ownership::set_owner(deps.storage, &owner)?;

    Ok(Response::new()
        .add_attribute("method", "instantiate")
        .add_attribute("owner", owner))
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn execute(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: ExecuteMsg,
) -> Result<Response, RewardsError> {
    match msg {
        ExecuteMsg::DistributeRewards {
            merkle_root,
            reward_distribution,
            reward_type,
        } => match reward_type {
            RewardsType::CW20 => {
                execute::distribute_rewards_cw20(deps, info, env, merkle_root, reward_distribution)
            }
            RewardsType::Bank => {
                execute::distribute_rewards_bank(deps, info, merkle_root, reward_distribution)
            }
        },
        ExecuteMsg::ClaimRewards {
            claim_rewards_proof,
            reward_type,
            service,
            token,
            amount,
            recipient,
        } => match reward_type {
            RewardsType::CW20 => execute::claim_rewards_cw20(
                deps,
                info,
                service,
                token,
                amount,
                claim_rewards_proof,
                recipient,
            ),
            RewardsType::Bank => execute::claim_rewards_bank(
                deps,
                info,
                service,
                token,
                amount,
                claim_rewards_proof,
                recipient,
            ),
        },
        ExecuteMsg::TransferOwnership { new_owner } => {
            let new_owner = deps.api.addr_validate(&new_owner)?;
            ownership::transfer_ownership(deps.storage, info, new_owner)
                .map_err(RewardsError::Ownership)
        }
    }
}

mod execute {
    use crate::error::RewardsError;
    use crate::merkle::{verify_merkle_proof, Leaf};
    use crate::msg::{ClaimRewardsProof, RewardDistribution};
    use crate::state::{BALANCES, CLAIMED_REWARDS, DISTRIBUTION_ROOTS};
    use cosmwasm_schema::cw_serde;
    use cosmwasm_std::{
        to_json_binary, Addr, BankMsg, Coin, DepsMut, Env, Event, HexBinary, MessageInfo, Response,
        StdError, Uint128,
    };
    use std::ops::{Add, Sub};

    #[cw_serde]
    pub struct ClaimRewardsInternalResponse {
        pub event: Event,
        pub amount_to_claim: Uint128,
    }

    pub fn distribute_rewards_bank(
        deps: DepsMut,
        info: MessageInfo,
        merkle_root: HexBinary,
        reward_distribution: RewardDistribution,
    ) -> Result<Response, RewardsError> {
        // only service (info.sender) can distribute rewards
        let service = info.sender.clone();

        // check that if bank token is transferred to the contract and same as the one in the distribution
        let info_funds_amount = cw_utils::may_pay(&info, &reward_distribution.token)?;
        if info_funds_amount != reward_distribution.amount {
            return Err(RewardsError::FundsMismatch {});
        }

        // update balances
        BALANCES.update(
            deps.storage,
            (&service, &reward_distribution.token),
            |balance| -> Result<_, RewardsError> {
                Ok(balance
                    .unwrap_or_default()
                    .checked_add(reward_distribution.amount)
                    .unwrap())
            },
        )?;

        // update distribution roots
        DISTRIBUTION_ROOTS.save(
            deps.storage,
            (&service, &reward_distribution.token),
            &merkle_root,
        )?;

        Ok(Response::new().add_event(
            Event::new("DistributeRewards")
                .add_attribute("service", service)
                .add_attribute("token", reward_distribution.token)
                .add_attribute("amount", reward_distribution.amount.to_string())
                .add_attribute("root", merkle_root.to_string()),
        ))
    }

    pub fn distribute_rewards_cw20(
        deps: DepsMut,
        info: MessageInfo,
        env: Env,
        merkle_root: HexBinary,
        reward_distribution: RewardDistribution,
    ) -> Result<Response, RewardsError> {
        // only service (info.sender) can distribute rewards
        let service = info.sender;

        let mut response = Response::new();

        // validate cw20 token address
        let token = deps.api.addr_validate(&reward_distribution.token)?;

        let transfer_msg = if reward_distribution.amount > Uint128::zero() {
            // transfer the rewards to the contract
            let transfer_msg = cosmwasm_std::WasmMsg::Execute {
                contract_addr: token.to_string(),
                msg: to_json_binary(&cw20::Cw20ExecuteMsg::TransferFrom {
                    owner: service.to_string(),
                    recipient: env.contract.address.to_string(),
                    amount: reward_distribution.amount,
                })?,
                funds: vec![],
            };

            // update balances
            BALANCES.update(
                deps.storage,
                (&service, &token.to_string()),
                |balance| -> Result<_, RewardsError> {
                    Ok(balance
                        .unwrap_or_default()
                        .checked_add(reward_distribution.amount)
                        .unwrap())
                },
            )?;

            // add transfer message as submsg
            Some(transfer_msg)
        } else {
            None
        };

        // update distribution roots
        DISTRIBUTION_ROOTS.save(deps.storage, (&service, &token.to_string()), &merkle_root)?;

        // add transfer message as submsg
        response = if let Some(transfer_msg) = transfer_msg {
            response.add_message(transfer_msg)
        } else {
            response
        };

        Ok(response.add_event(
            Event::new("DistributeRewards")
                .add_attribute("service", service)
                .add_attribute("token", token.to_string())
                .add_attribute("amount", reward_distribution.amount.to_string())
                .add_attribute("root", merkle_root.to_string()),
        ))
    }

    pub fn claim_rewards_bank(
        deps: DepsMut,
        info: MessageInfo,
        service: String,
        token: String,
        amount: Uint128,
        claim_rewards_proof: ClaimRewardsProof,
        recipient: String,
    ) -> Result<Response, RewardsError> {
        // validate service address
        let service = deps.api.addr_validate(&service)?;

        // assume earner is the sender
        let earner = info.sender.clone();

        let recipient = deps.api.addr_validate(&recipient)?;

        let claim_rewards_res = claim_rewards_internal(
            deps,
            service,
            earner,
            token.to_string(),
            amount,
            claim_rewards_proof,
            recipient.clone(),
        )?;

        // transfer the rewards to the earner
        let transfer_msg = BankMsg::Send {
            to_address: recipient.to_string(),
            amount: vec![Coin {
                denom: token.to_string(),
                amount: claim_rewards_res.amount_to_claim,
            }],
        };

        Ok(Response::new()
            .add_message(transfer_msg)
            .add_event(claim_rewards_res.event))
    }

    pub fn claim_rewards_cw20(
        deps: DepsMut,
        info: MessageInfo,
        service: String,
        token: String,
        amount: Uint128,
        claim_rewards_proof: ClaimRewardsProof,
        recipient: String,
    ) -> Result<Response, RewardsError> {
        // validate service address
        let service = deps.api.addr_validate(&service)?;
        // validate token
        let token = deps.api.addr_validate(&token)?;

        // assume earner is the sender
        let earner = info.sender.clone();

        let recipient = deps.api.addr_validate(&recipient)?;

        let claim_rewards_res = claim_rewards_internal(
            deps,
            service,
            earner,
            token.to_string(),
            amount,
            claim_rewards_proof,
            recipient.clone(),
        )?;

        // transfer the rewards to the earner
        let transfer_msg = cosmwasm_std::WasmMsg::Execute {
            contract_addr: token.to_string(),
            msg: to_json_binary(&cw20::Cw20ExecuteMsg::Transfer {
                recipient: recipient.to_string(),
                amount: claim_rewards_res.amount_to_claim,
            })?,
            funds: vec![],
        };

        Ok(Response::new()
            .add_message(transfer_msg)
            .add_event(claim_rewards_res.event))
    }

    pub fn claim_rewards_internal(
        deps: DepsMut,
        service: Addr,
        earner: Addr,
        token: String,
        amount: Uint128,
        claim_rewards_proof: ClaimRewardsProof,
        recipient: Addr,
    ) -> Result<ClaimRewardsInternalResponse, RewardsError> {
        // check root is not empty
        if claim_rewards_proof.root == HexBinary::default() {
            return Err(RewardsError::Std(StdError::generic_err("Empty root")));
        };

        // assert that total rewards must be more than rewards already claimed
        let claimed_rewards = CLAIMED_REWARDS
            .may_load(deps.storage, (&service, &token.to_string(), &earner))?
            .unwrap_or_default();
        if claimed_rewards >= amount {
            return Err(RewardsError::AlreadyClaimed {});
        }

        // no need checked_sub as amount > claimed_rewards
        let amount_to_claim = amount - claimed_rewards;

        // check if balance is enough
        let balance = BALANCES
            .load(deps.storage, (&service, &token.to_string()))
            .unwrap_or_default();

        if amount_to_claim > balance {
            return Err(RewardsError::InsufficientBalance {});
        }

        let leaf = Leaf {
            earner: earner.to_string(),
            amount,
        };

        let merkle_proof: bool = verify_merkle_proof(
            claim_rewards_proof.root,
            claim_rewards_proof.proof,
            leaf,
            claim_rewards_proof.leaf_index,
            claim_rewards_proof.total_leaves_count,
        )?;

        if !merkle_proof {
            return Err(RewardsError::InvalidProof {
                msg: "Invalid Merkle proof".to_string(),
            });
        };

        // reduce balance, no need checked_sub as amount_to_claim > balance
        BALANCES.save(
            deps.storage,
            (&service, &token.to_string()),
            &balance.sub(amount_to_claim),
        )?;

        // increment claimed rewards,
        // no need checked_add as amount_to_claim + claimed_rewards = amount which is Uin128
        CLAIMED_REWARDS.save(
            deps.storage,
            (&service, &token.to_string(), &earner),
            &claimed_rewards.add(amount_to_claim),
        )?;

        let event = Event::new("ClaimRewards")
            .add_attribute("service", service)
            .add_attribute("earner", earner)
            .add_attribute("recipient", recipient)
            .add_attribute("total_claimed_amount", amount.to_string())
            .add_attribute("amount", amount_to_claim.to_string())
            .add_attribute("token", token.to_string());

        Ok(ClaimRewardsInternalResponse {
            event,
            amount_to_claim,
        })
    }
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, _env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::DistributionRoot { service, token } => {
            let service = deps.api.addr_validate(&service)?;
            to_json_binary(&query::distribution_root(deps, service, token)?)
        }
    }
}

mod query {
    use crate::state::DISTRIBUTION_ROOTS;
    use cosmwasm_std::{Addr, Deps, StdResult};

    /// Query the distribution root for a given service and token
    ///
    /// returns HexBinary::default() if no root is found
    pub fn distribution_root(deps: Deps, service: Addr, token: String) -> StdResult<String> {
        DISTRIBUTION_ROOTS
            .may_load(deps.storage, (&service, &token))
            .map(|shares| shares.unwrap_or_default().to_string())
    }
}

#[cfg(test)]
mod tests {
    use crate::contract::execute::{distribute_rewards_bank, distribute_rewards_cw20};
    use crate::msg::RewardDistribution;
    use crate::state::{BALANCES, DISTRIBUTION_ROOTS};
    use cosmwasm_std::testing::{message_info, mock_dependencies, mock_env};
    use cosmwasm_std::{coins, Event, HexBinary, Uint128};

    #[test]
    fn test_distribute_rewards_bank() {
        let mut deps = mock_dependencies();
        let service = deps.api.addr_make("service");

        let bank_token = "stone";
        let info = message_info(&service, &coins(10000, bank_token));

        let merkle_root = "3902889975800375703a50bbe0d7a5c297977cb44348bf991cca43594fc644ef";

        let reward_distribution = RewardDistribution {
            token: bank_token.to_string(),
            amount: Uint128::new(10_000),
        };

        let res = distribute_rewards_bank(
            deps.as_mut(),
            info,
            HexBinary::from_hex(merkle_root).unwrap(),
            reward_distribution,
        )
        .unwrap();

        // assert events are correct
        assert_eq!(res.events.len(), 1);
        assert_eq!(
            res.events[0],
            Event::new("DistributeRewards")
                .add_attribute("service", service.clone())
                .add_attribute("token", bank_token)
                .add_attribute("amount", "10000")
                .add_attribute("root", merkle_root)
        );

        // assert BALANCES state is updated
        let balance = BALANCES
            .load(&deps.storage, (&service, &bank_token.to_string()))
            .unwrap();
        assert_eq!(balance, Uint128::new(10_000));

        // assert DISTRIBUTION_ROOTS state is updated
        let root = DISTRIBUTION_ROOTS
            .load(&deps.storage, (&service, &bank_token.to_string()))
            .unwrap();
        assert_eq!(root, HexBinary::from_hex(merkle_root).unwrap());
    }

    #[test]
    fn test_distribute_rewards_cw20() {
        let mut deps = mock_dependencies();
        let info = message_info(&deps.api.addr_make("service"), &[]);
        let env = mock_env();
        let service = deps.api.addr_make("service");
        let cw20 = deps.api.addr_make("cw20");

        let merkle_root = "3902889975800375703a50bbe0d7a5c297977cb44348bf991cca43594fc644ef";

        let reward_distribution = RewardDistribution {
            token: cw20.to_string(),
            amount: Uint128::new(10_000),
        };

        let res = distribute_rewards_cw20(
            deps.as_mut(),
            info,
            env,
            HexBinary::from_hex(merkle_root).unwrap(),
            reward_distribution,
        )
        .unwrap();

        // assert events are correct
        assert_eq!(res.events.len(), 1);
        assert_eq!(
            res.events[0],
            Event::new("DistributeRewards")
                .add_attribute("service", service.clone())
                .add_attribute("token", cw20.to_string())
                .add_attribute("amount", "10000")
                .add_attribute("root", merkle_root)
        );

        // assert BALANCES state is updated
        let balance = BALANCES
            .load(&deps.storage, (&service, &cw20.to_string()))
            .unwrap();
        assert_eq!(balance, Uint128::new(10_000));

        // assert DISTRIBUTION_ROOTS state is updated
        let root = DISTRIBUTION_ROOTS
            .load(&deps.storage, (&service, &cw20.to_string()))
            .unwrap();
        assert_eq!(root, HexBinary::from_hex(merkle_root).unwrap());
    }
}
