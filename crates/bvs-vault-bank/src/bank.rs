use cosmwasm_std::{
    Addr, BalanceResponse, BankMsg, BankQuery, Coin, Deps, Env, QueryRequest, StdResult, Storage,
    Uint128,
};
use cw_storage_plus::Item;

const DENOM: Item<String> = Item::new("denom");

/// Set the denom of the contract during instantiation
pub fn set_denom(storage: &mut dyn Storage, denom: impl Into<String>) -> StdResult<()> {
    DENOM.save(storage, &denom.into())
}

/// Get the denom of the contract from storage
pub fn get_denom(storage: &dyn Storage) -> StdResult<String> {
    DENOM.load(storage)
}

/// Query the balance of the contract, using [BankQuery::Balance]
pub fn query_balance(deps: &Deps, env: &Env) -> StdResult<Uint128> {
    let denom = DENOM.load(deps.storage)?;
    let address = env.contract.address.to_string();

    let query = BankQuery::Balance { address, denom };

    let res: BalanceResponse = deps.querier.query(&QueryRequest::Bank(query))?;
    Ok(res.amount.amount)
}

/// Create a [BankMsg::Send] message to send Bank tokens to a recipient
pub fn bank_send(
    storage: &mut dyn Storage,
    recipient: &Addr,
    amount: Uint128,
) -> StdResult<cosmwasm_std::CosmosMsg> {
    let denom = DENOM.load(storage)?;
    let msg = BankMsg::Send {
        to_address: recipient.to_string(),
        amount: vec![Coin { denom, amount }],
    };
    Ok(msg.into())
}

#[cfg(test)]
mod tests {
    use crate::bank;
    use crate::bank::set_denom;
    use cosmwasm_std::testing::{mock_dependencies, mock_env};
    use cosmwasm_std::{coins, Coin, CosmosMsg, Uint128};

    #[test]
    fn test_get_denom() {
        let mut deps = mock_dependencies();
        set_denom(&mut deps.storage, "baby").unwrap();
        let denom = bank::get_denom(&deps.storage).unwrap();
        assert_eq!(denom, "baby");
    }

    #[test]
    fn test_query_balance() {
        let mut deps = mock_dependencies();
        let env = mock_env();

        {
            set_denom(&mut deps.storage, "heart").unwrap();
            let balance = coins(100_000, "heart");
            deps.querier
                .bank
                .update_balance(env.contract.address.clone(), balance);
        }

        let balance = bank::query_balance(&deps.as_ref(), &env).unwrap();
        assert_eq!(balance, Uint128::new(100_000));
    }

    #[test]
    fn test_query_balance_but_different_denom() {
        let mut deps = mock_dependencies();
        let env = mock_env();

        {
            set_denom(&mut deps.storage, "paper").unwrap();
            let balance = coins(100_000, "rocks");
            deps.querier
                .bank
                .update_balance(env.contract.address.clone(), balance);
        }

        let balance = bank::query_balance(&deps.as_ref(), &env).unwrap();
        assert_eq!(balance, Uint128::zero());
    }

    #[test]
    fn test_query_balance_two_denom() {
        let mut deps = mock_dependencies();
        let env = mock_env();

        {
            set_denom(&mut deps.storage, "paper").unwrap();
            deps.querier.bank.update_balance(
                env.contract.address.clone(),
                vec![
                    Coin {
                        denom: "rocks".to_string(),
                        amount: Uint128::new(834_545),
                    },
                    Coin {
                        denom: "paper".to_string(),
                        amount: Uint128::new(847_534_053),
                    },
                ],
            );
        }

        let balance = bank::query_balance(&deps.as_ref(), &env).unwrap();
        assert_eq!(balance, Uint128::new(847_534_053));
    }

    #[test]
    fn test_bank_send() {
        let mut deps = mock_dependencies();
        let recipient = deps.api.addr_make("recipient");
        set_denom(&mut deps.storage, "heart").unwrap();

        let msg = bank::bank_send(
            &mut deps.storage,
            &recipient,
            Uint128::new(286937928376452954),
        )
        .unwrap();

        assert_eq!(
            msg,
            CosmosMsg::Bank(cosmwasm_std::BankMsg::Send {
                to_address: recipient.to_string(),
                amount: vec![Coin {
                    denom: "heart".to_string(),
                    amount: Uint128::new(286937928376452954)
                }]
            })
        )
    }
}
