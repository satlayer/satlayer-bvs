use bvs_library::testing::{Cw20TokenContract, TestingContract};
use bvs_rewards::msg::ExecuteMsg::DistributeRewards;
use bvs_rewards::msg::{DistributionRootResponse, QueryMsg, RewardDistribution, RewardsType};
use bvs_rewards::testing::RewardsContract;
use cosmwasm_std::testing::mock_env;
use cosmwasm_std::{coin, coins, Event, HexBinary, Uint128};
use cw_multi_test::{App, Executor};

fn instantiate() -> (App, RewardsContract, Cw20TokenContract) {
    let mut app = App::new(|router, api, storage| {
        let owner = api.addr_make("owner");
        router
            .bank
            .init_balance(
                storage,
                &owner,
                vec![
                    coin(Uint128::MAX.u128(), "rock"),
                    coin(Uint128::MAX.u128(), "stone"),
                ],
            )
            .unwrap();
    });
    let env = mock_env();

    let rewards_contract = RewardsContract::new(&mut app, &env, None);
    let cw20 = Cw20TokenContract::new(&mut app, &env, None);

    (app, rewards_contract, cw20)
}

#[test]
fn test_distribute_rewards_bank_mismatch_denom() {
    let (mut app, rewards_contract, _) = instantiate();

    let service = app.api().addr_make("service");
    let owner = app.api().addr_make("owner");

    let bank_token = "stone";

    let merkle_root = "3902889975800375703a50bbe0d7a5c297977cb44348bf991cca43594fc644ef";

    let reward_distribution = RewardDistribution {
        token: bank_token.to_string(),
        amount: Uint128::new(10_000),
    };

    // mint "rock" denom to service
    app.send_tokens(owner, service.clone(), coins(10_000, "rock").as_ref())
        .expect("Failed to mint tokens");

    // send "rock" denom in info instead of "stone"
    let err = rewards_contract
        .execute_with_funds(
            &mut app,
            &service,
            &DistributeRewards {
                merkle_root: HexBinary::from_hex(merkle_root).unwrap(),
                reward_distribution,
                reward_type: RewardsType::Bank,
            },
            coins(10_000, "rock"),
        )
        .unwrap_err();

    assert_eq!(
        err.root_cause().to_string(),
        "Payment error: Received unsupported denom 'rock'"
    );
}

#[test]
fn test_distribute_rewards_bank_mismatch_amount() {
    let (mut app, rewards_contract, _) = instantiate();

    let service = app.api().addr_make("service");
    let owner = app.api().addr_make("owner");

    let bank_token = "stone";

    let merkle_root = "3902889975800375703a50bbe0d7a5c297977cb44348bf991cca43594fc644ef";

    let reward_distribution = RewardDistribution {
        token: bank_token.to_string(),
        amount: Uint128::new(10_000),
    };

    // mint "stone" denom to service
    app.send_tokens(owner, service.clone(), coins(10_001, bank_token).as_ref())
        .expect("Failed to mint tokens");

    {
        // send 9_999 denom in info instead of 10_000
        let err = rewards_contract
            .execute_with_funds(
                &mut app,
                &service,
                &DistributeRewards {
                    merkle_root: HexBinary::from_hex(merkle_root).unwrap(),
                    reward_distribution: reward_distribution.clone(),
                    reward_type: RewardsType::Bank,
                },
                coins(9_999, bank_token), // send a lower amount than reward_distribution.amount
            )
            .unwrap_err();

        assert_eq!(
            err.root_cause().to_string(),
            "Funds sent do not match the funds received"
        );
    }
    {
        // send 10_001 denom in info instead of 10_000
        let err = rewards_contract
            .execute_with_funds(
                &mut app,
                &service,
                &DistributeRewards {
                    merkle_root: HexBinary::from_hex(merkle_root).unwrap(),
                    reward_distribution,
                    reward_type: RewardsType::Bank,
                },
                coins(10_001, bank_token), // send a higher amount than reward_distribution.amount
            )
            .unwrap_err();

        assert_eq!(
            err.root_cause().to_string(),
            "Funds sent do not match the funds received"
        );
    }
}

#[test]
fn test_distribute_rewards_bank() {
    let (mut app, rewards_contract, _) = instantiate();

    let service = app.api().addr_make("service");
    let owner = app.api().addr_make("owner");

    let bank_token = "stone";

    let merkle_root = "3902889975800375703a50bbe0d7a5c297977cb44348bf991cca43594fc644ef";

    let reward_distribution = RewardDistribution {
        token: bank_token.to_string(),
        amount: Uint128::new(10_000),
    };

    // mint "stone" denom to service
    app.send_tokens(owner, service.clone(), coins(10_001, bank_token).as_ref())
        .expect("Failed to mint tokens");

    // send 10_000 denom in info
    let res = rewards_contract
        .execute_with_funds(
            &mut app,
            &service,
            &DistributeRewards {
                merkle_root: HexBinary::from_hex(merkle_root).unwrap(),
                reward_distribution: reward_distribution.clone(),
                reward_type: RewardsType::Bank,
            },
            coins(10_000, bank_token),
        )
        .unwrap();

    // assert events are correct
    assert_eq!(res.events.len(), 2);
    assert_eq!(
        res.events[1],
        Event::new("wasm-DistributeRewards")
            .add_attribute("_contract_address", rewards_contract.addr.clone())
            .add_attribute("service", service.clone())
            .add_attribute("token", bank_token)
            .add_attribute("amount", "10000")
            .add_attribute("root", merkle_root)
    );

    // assert DISTRIBUTION_ROOTS state is updated
    let DistributionRootResponse(root) = rewards_contract
        .query(
            &app,
            &QueryMsg::DistributionRoot {
                service: service.to_string(),
                token: bank_token.to_string(),
            },
        )
        .unwrap();
    assert_eq!(root, merkle_root.to_string());
}

#[test]
fn test_distribute_rewards_cw20() {
    let (mut app, rewards_contract, cw20) = instantiate();

    let service = app.api().addr_make("service");

    let merkle_root = "3902889975800375703a50bbe0d7a5c297977cb44348bf991cca43594fc644ef";

    let reward_distribution = RewardDistribution {
        token: cw20.addr.to_string(),
        amount: Uint128::new(10_000),
    };

    // send 10_001 cw20 token to service
    cw20.fund(&mut app, &service, reward_distribution.amount.u128() + 1);

    // allow rewards contract to transfer token
    cw20.increase_allowance(
        &mut app,
        &service,
        &rewards_contract.addr,
        reward_distribution.amount.u128(),
    );

    // send 10_000 cw20 token
    let res = rewards_contract
        .execute(
            &mut app,
            &service,
            &DistributeRewards {
                merkle_root: HexBinary::from_hex(merkle_root).unwrap(),
                reward_distribution: reward_distribution.clone(),
                reward_type: RewardsType::CW20,
            },
        )
        .unwrap();

    // assert events are correct
    assert_eq!(res.events.len(), 4);
    assert_eq!(
        res.events[1],
        Event::new("wasm-DistributeRewards")
            .add_attribute("_contract_address", rewards_contract.addr.clone())
            .add_attribute("service", service.clone())
            .add_attribute("token", cw20.addr.clone())
            .add_attribute("amount", "10000")
            .add_attribute("root", merkle_root)
    );

    // assert DISTRIBUTION_ROOTS state is updated
    let DistributionRootResponse(root) = rewards_contract
        .query(
            &app,
            &QueryMsg::DistributionRoot {
                service: service.to_string(),
                token: cw20.addr.to_string(),
            },
        )
        .unwrap();
    assert_eq!(root, merkle_root.to_string());

    // assert service balance is reduced
    let balance: u128 = cw20.balance(&app, &service);
    assert_eq!(balance, 1u128); // 10_001 - 10_000

    // assert contract balance is increased
    let contract_balance: u128 = cw20.balance(&app, &rewards_contract.addr);
    assert_eq!(contract_balance, 10_000u128); // 10_000
}
