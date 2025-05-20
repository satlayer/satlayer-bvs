use bvs_guardrail::testing::GuardrailContract;
use bvs_library::slashing::SlashingRequestId;
use bvs_library::time::DAYS;
use bvs_library::{
    ownership::OwnershipError,
    testing::{Cw20TokenContract, TestingContract},
};
use bvs_pauser::testing::PauserContract;
use bvs_registry::msg::Metadata;
use bvs_registry::testing::RegistryContract;
use bvs_registry::SlashingParameters;
use bvs_vault_bank::testing::VaultBankContract;
use bvs_vault_base::msg::RecipientAmount;
use bvs_vault_cw20::testing::VaultCw20Contract;
use bvs_vault_router::msg::{
    RequestSlashingPayload, RequestSlashingResponse, SlashingMetadata, SlashingRequestIdResponse,
    SlashingRequestResponse, Vault,
};
use bvs_vault_router::state::{SlashingRequest, SlashingRequestStatus};
use bvs_vault_router::{
    msg::{ExecuteMsg, QueryMsg, VaultListResponse},
    testing::VaultRouterContract,
    ContractError,
};
use cosmwasm_std::{coin, coins, BalanceResponse, BankQuery, Decimal, QueryRequest, Uint128};
use cosmwasm_std::{testing::mock_env, Event, HexBinary, Uint64};
use cw_multi_test::{App, Executor};
use cw_utils::Threshold;

struct TestContracts {
    guardrail: GuardrailContract,
    vault_router: VaultRouterContract,
    bank_vault: VaultBankContract,
    cw20_vault: VaultCw20Contract,
    registry: RegistryContract,
    cw20: Cw20TokenContract,
}

impl TestContracts {
    fn init() -> (App, TestContracts) {
        let mut app = App::new(|router, api, storage| {
            let owner = api.addr_make("owner");
            router
                .bank
                .init_balance(storage, &owner, coins(Uint128::MAX.u128(), "denom"))
                .unwrap();
        });
        let env = mock_env();

        let owner = app.api().addr_make("owner");
        let voter1 = app.api().addr_make("voter1");
        let voter2 = app.api().addr_make("voter2");
        let voter3 = app.api().addr_make("voter3");
        let voter4 = app.api().addr_make("voter4");

        let guardrail_init_msg = bvs_guardrail::msg::InstantiateMsg {
            owner: owner.to_string(),
            members: vec![
                cw4::Member {
                    addr: voter1.to_string(),
                    weight: 1,
                },
                cw4::Member {
                    addr: voter2.to_string(),
                    weight: 1,
                },
                cw4::Member {
                    addr: voter3.to_string(),
                    weight: 1,
                },
                cw4::Member {
                    addr: voter4.to_string(),
                    weight: 1,
                },
                cw4::Member {
                    addr: owner.to_string(),
                    weight: 0,
                },
            ],
            threshold: Threshold::AbsolutePercentage {
                percentage: Decimal::percent(50),
            },
            default_expiration: 1000,
        };

        let _ = PauserContract::new(&mut app, &env, None);
        let registry = RegistryContract::new(&mut app, &env, None);
        let guardrail = GuardrailContract::new(&mut app, &env, Some(guardrail_init_msg));
        let vault_router = VaultRouterContract::new(&mut app, &env, None);
        let bank_vault = VaultBankContract::new(&mut app, &env, None);
        let cw20 = Cw20TokenContract::new(&mut app, &env, None);
        let cw20_vault = VaultCw20Contract::new(&mut app, &env, None);

        (
            app,
            Self {
                guardrail,
                vault_router,
                bank_vault,
                cw20_vault,
                registry,
                cw20,
            },
        )
    }
}

#[test]
fn set_vault_whitelist_false_successfully() {
    let (mut app, tc) = TestContracts::init();
    let owner = app.api().addr_make("owner");
    let vault = app.api().addr_make("vault");

    let msg = &ExecuteMsg::SetVault {
        vault: vault.to_string(),
        whitelisted: false,
    };

    let response = tc.vault_router.execute(&mut app, &owner, msg).unwrap();

    assert_eq!(
        response.events,
        vec![
            Event::new("execute").add_attribute("_contract_address", tc.vault_router.addr.as_str()),
            Event::new("wasm-VaultUpdated")
                .add_attribute("_contract_address", tc.vault_router.addr.as_str())
                .add_attribute("vault", vault.to_string())
                .add_attribute("whitelisted", "false"),
        ]
    );

    // query is whitelisted
    let msg = QueryMsg::IsWhitelisted {
        vault: vault.to_string(),
    };
    let is_whitelisted: bool = tc.vault_router.query(&mut app, &msg).unwrap();
    assert!(!is_whitelisted);

    // query is delegated
    let operator = app.api().addr_make("operator");
    let msg = QueryMsg::IsValidating {
        operator: operator.to_string(),
    };
    let is_validating: bool = tc.vault_router.query(&mut app, &msg).unwrap();
    assert!(!is_validating);

    // list vaults
    let msg = QueryMsg::ListVaults {
        start_after: None,
        limit: None,
    };
    let VaultListResponse(vaults) = tc.vault_router.query(&mut app, &msg).unwrap();
    assert_eq!(
        vaults,
        vec![Vault {
            vault,
            whitelisted: false,
        }]
    );

    let msg = QueryMsg::ListVaultsByOperator {
        operator: operator.to_string(),
        start_after: None,
        limit: None,
    };

    let VaultListResponse(vaults) = tc.vault_router.query(&mut app, &msg).unwrap();
    assert_eq!(vaults.len(), 0);
}

#[test]
fn set_vault_whitelist_true_bank_vault_successfully() {
    let (mut app, tc) = TestContracts::init();
    let owner = app.api().addr_make("owner");

    let msg = &ExecuteMsg::SetVault {
        vault: tc.bank_vault.addr().to_string(),
        whitelisted: true,
    };

    let response = tc.vault_router.execute(&mut app, &owner, msg).unwrap();

    assert_eq!(
        response.events,
        vec![
            Event::new("execute").add_attribute("_contract_address", tc.vault_router.addr.as_str()),
            Event::new("wasm-VaultUpdated")
                .add_attribute("_contract_address", tc.vault_router.addr.as_str())
                .add_attribute("vault", tc.bank_vault.addr().to_string())
                .add_attribute("whitelisted", "true"),
        ]
    );

    // query is whitelisted
    let msg = QueryMsg::IsWhitelisted {
        vault: tc.bank_vault.addr().to_string(),
    };
    let is_whitelisted: bool = tc.vault_router.query(&mut app, &msg).unwrap();
    assert!(is_whitelisted);

    let operator = app.api().addr_make("operator");

    let msg = QueryMsg::ListVaultsByOperator {
        operator: operator.to_string(),
        start_after: None,
        limit: None,
    };

    let VaultListResponse(vaults) = tc.vault_router.query(&mut app, &msg).unwrap();
    assert_eq!(vaults[0].vault, tc.bank_vault.addr());
}

#[test]
fn set_vault_whitelist_true_cw20_vault_successfully() {
    let (mut app, tc) = TestContracts::init();
    let owner = app.api().addr_make("owner");

    let msg = &ExecuteMsg::SetVault {
        vault: tc.cw20_vault.addr().to_string(),
        whitelisted: true,
    };

    let response = tc.vault_router.execute(&mut app, &owner, msg).unwrap();

    assert_eq!(
        response.events,
        vec![
            Event::new("execute").add_attribute("_contract_address", tc.vault_router.addr.as_str()),
            Event::new("wasm-VaultUpdated")
                .add_attribute("_contract_address", tc.vault_router.addr.as_str())
                .add_attribute("vault", tc.cw20_vault.addr().to_string())
                .add_attribute("whitelisted", "true"),
        ]
    );

    // query is whitelisted
    let msg = QueryMsg::IsWhitelisted {
        vault: tc.cw20_vault.addr().to_string(),
    };
    let is_whitelisted: bool = tc.vault_router.query(&mut app, &msg).unwrap();
    assert!(is_whitelisted);

    let operator = app.api().addr_make("operator");

    let msg = QueryMsg::ListVaultsByOperator {
        operator: operator.to_string(),
        start_after: None,
        limit: None,
    };

    let VaultListResponse(vaults) = tc.vault_router.query(&mut app, &msg).unwrap();
    assert_eq!(vaults[0].vault, tc.cw20_vault.addr());
}

#[test]
fn set_withdrawal_lock_period() {
    let (mut app, tc) = TestContracts::init();
    let owner = app.api().addr_make("owner");

    let withdrawal_lock_period1 = Uint64::new(120);

    // set withdrawal lock period for the first time
    {
        let msg = &ExecuteMsg::SetWithdrawalLockPeriod(withdrawal_lock_period1);

        let response = tc.vault_router.execute(&mut app, &owner, msg).unwrap();

        assert_eq!(
            response.events,
            vec![
                Event::new("execute")
                    .add_attribute("_contract_address", tc.vault_router.addr.as_str()),
                Event::new("wasm-SetWithdrawalLockPeriod")
                    .add_attribute("_contract_address", tc.vault_router.addr.as_str())
                    .add_attribute(
                        "prev_withdrawal_lock_period",
                        Uint64::new(604800).to_string()
                    )
                    .add_attribute(
                        "new_withdrawal_lock_period",
                        withdrawal_lock_period1.to_string()
                    ),
            ]
        );
    }

    let withdrawal_lock_period2 = Uint64::new(150);

    // update withdrawal lock period
    {
        let msg = &ExecuteMsg::SetWithdrawalLockPeriod(withdrawal_lock_period2);

        let response = tc.vault_router.execute(&mut app, &owner, msg).unwrap();

        assert_eq!(
            response.events,
            vec![
                Event::new("execute")
                    .add_attribute("_contract_address", tc.vault_router.addr.as_str()),
                Event::new("wasm-SetWithdrawalLockPeriod")
                    .add_attribute("_contract_address", tc.vault_router.addr.as_str())
                    .add_attribute(
                        "prev_withdrawal_lock_period",
                        withdrawal_lock_period1.to_string()
                    )
                    .add_attribute(
                        "new_withdrawal_lock_period",
                        withdrawal_lock_period2.to_string()
                    ),
            ]
        );
    }

    // query the withdrawal lock period
    let msg = QueryMsg::WithdrawalLockPeriod {};
    let result: Uint64 = tc.vault_router.query(&mut app, &msg).unwrap();
    assert_eq!(result, withdrawal_lock_period2);
}

#[test]
fn transfer_ownership_successfully() {
    let (mut app, tc) = TestContracts::init();
    let owner = app.api().addr_make("owner");
    let new_owner = app.api().addr_make("new_owner");

    let transfer_msg = &ExecuteMsg::TransferOwnership {
        new_owner: new_owner.to_string(),
    };

    let response = tc
        .vault_router
        .execute(&mut app, &owner, transfer_msg)
        .unwrap();

    assert_eq!(
        response.events,
        vec![
            Event::new("execute").add_attribute("_contract_address", tc.vault_router.addr.as_str()),
            Event::new("wasm-TransferredOwnership")
                .add_attribute("_contract_address", tc.vault_router.addr.as_str())
                .add_attribute("old_owner", owner.as_str())
                .add_attribute("new_owner", new_owner.as_str()),
        ]
    );
}

#[test]
fn transfer_ownership_but_not_owner() {
    let (mut app, tc) = TestContracts::init();
    let not_owner = app.api().addr_make("not_owner");

    let transfer_msg = &ExecuteMsg::TransferOwnership {
        new_owner: not_owner.to_string(),
    };

    let err = tc
        .vault_router
        .execute(&mut app, &not_owner, transfer_msg)
        .unwrap_err();

    assert_eq!(
        err.root_cause().to_string(),
        ContractError::Ownership(OwnershipError::Unauthorized).to_string()
    );
}

#[test]
fn request_slashing_successful() {
    let (mut app, tc) = TestContracts::init();

    let operator = app.api().addr_make("operator");
    let service = app.api().addr_make("service");

    // register operator + service
    {
        tc.registry
            .execute(
                &mut app,
                &operator,
                &bvs_registry::msg::ExecuteMsg::RegisterAsOperator {
                    metadata: Metadata {
                        name: Some("operator".to_string()),
                        uri: None,
                    },
                },
            )
            .expect("failed to register operator");

        tc.registry
            .execute(
                &mut app,
                &service,
                &bvs_registry::msg::ExecuteMsg::RegisterAsService {
                    metadata: Metadata {
                        name: Some("service".to_string()),
                        uri: None,
                    },
                },
            )
            .expect("failed to register service");
    }

    // service enable slashing
    {
        let msg = &bvs_registry::msg::ExecuteMsg::EnableSlashing {
            slashing_parameters: SlashingParameters {
                destination: Some(service.clone()),
                max_slashing_bips: 5000,
                resolution_window: 100,
            },
        };
        tc.registry
            .execute(&mut app, &service, msg)
            .expect("failed to enable slashing");
    }

    app.update_block(|block| {
        block.height += 1;
        block.time = block.time.plus_seconds(10);
    });

    // register operator to service for active status
    {
        let msg = &bvs_registry::msg::ExecuteMsg::RegisterOperatorToService {
            operator: operator.to_string(),
        };
        tc.registry
            .execute(&mut app, &service, msg)
            .expect("failed to register operator to service");

        let msg = &bvs_registry::msg::ExecuteMsg::RegisterServiceToOperator {
            service: service.to_string(),
        };
        tc.registry
            .execute(&mut app, &operator, msg)
            .expect("failed to register service to operator");
    }

    app.update_block(|block| {
        block.height += 1;
        block.time = block.time.plus_seconds(10);
    });

    // service request slashing
    let slashing_request_payload = RequestSlashingPayload {
        operator: operator.to_string(),
        bips: 100,
        timestamp: app.block_info().time,
        metadata: SlashingMetadata {
            reason: "test".to_string(),
        },
    };

    let msg = &ExecuteMsg::RequestSlashing(slashing_request_payload.clone());
    let response = tc.vault_router.execute(&mut app, &service, msg).unwrap();
    assert_eq!(
        response.events,
        vec![
            Event::new("execute").add_attribute("_contract_address", tc.vault_router.addr.as_str()),
            Event::new("wasm-RequestSlashing")
                .add_attribute("_contract_address", tc.vault_router.addr.as_str())
                .add_attribute("service", service.to_string())
                .add_attribute("operator", operator.to_string())
                .add_attribute(
                    "slashing_request_id",
                    "cffcb7e810be616e5582beb8bdb8a545502733d683515410d97d262dcba1855c"
                )
                .add_attribute("reason", "test"),
        ]
    );

    let RequestSlashingResponse(slashing_id) = response.data.unwrap().into();
    assert_eq!(
        slashing_id,
        SlashingRequestId::from_hex(
            "cffcb7e810be616e5582beb8bdb8a545502733d683515410d97d262dcba1855c"
        )
        .unwrap()
    );

    // query slashing request id
    let msg = QueryMsg::SlashingRequestId {
        service: service.to_string(),
        operator: operator.to_string(),
    };
    let slashing_request_id: SlashingRequestIdResponse =
        tc.vault_router.query(&mut app, &msg).unwrap();
    assert_eq!(
        slashing_request_id,
        SlashingRequestIdResponse(Some(
            HexBinary::from_hex("cffcb7e810be616e5582beb8bdb8a545502733d683515410d97d262dcba1855c")
                .unwrap()
                .into()
        ))
    );

    // query slashing request
    let msg = QueryMsg::SlashingRequest(slashing_request_id.0.unwrap());
    let SlashingRequestResponse(slashing_request) = tc.vault_router.query(&mut app, &msg).unwrap();
    assert_eq!(
        slashing_request.unwrap(),
        SlashingRequest {
            request: slashing_request_payload,
            request_time: app.block_info().time,
            request_resolution: app.block_info().time.plus_seconds(100),
            request_expiry: app.block_info().time.plus_seconds(200),
            status: SlashingRequestStatus::Pending.into(),
            service,
        }
    );
}

/// This test is
/// to check all negative cases for slashing request and finally a success case in the end.
/// To find a simpler success case, look at the `request_slashing_successful` test.
#[test]
fn request_slashing_lifecycle() {
    let (mut app, tc) = TestContracts::init();

    let operator = app.api().addr_make("operator");
    let service = app.api().addr_make("service");

    // register operator + service
    {
        tc.registry
            .execute(
                &mut app,
                &operator,
                &bvs_registry::msg::ExecuteMsg::RegisterAsOperator {
                    metadata: Metadata {
                        name: Some("operator".to_string()),
                        uri: None,
                    },
                },
            )
            .expect("failed to register operator");

        tc.registry
            .execute(
                &mut app,
                &service,
                &bvs_registry::msg::ExecuteMsg::RegisterAsService {
                    metadata: Metadata {
                        name: Some("service".to_string()),
                        uri: None,
                    },
                },
            )
            .expect("failed to register service");
    }

    app.update_block(|block| {
        block.height += 1;
        block.time = block.time.plus_seconds(10);
    });

    // service request slashing with reason string over limit
    {
        let slashing_request_payload = RequestSlashingPayload {
            operator: operator.to_string(),
            bips: 100,
            timestamp: app.block_info().time,
            metadata: SlashingMetadata {
                reason: "this is a reason to slash the operator that is too long to be allowed in the \
                metadata of this test, and we need to make it even longer to reach exactly 251 bytes total. \
                The purpose of this test is to verify that the system properly rejects long text. ".to_string(),
            },
        };

        let msg = &ExecuteMsg::RequestSlashing(slashing_request_payload.clone());
        let err = tc
            .vault_router
            .execute(&mut app, &service, msg)
            .unwrap_err();
        assert_eq!(
            err.root_cause().to_string(),
            ContractError::InvalidSlashingRequest {
                msg: "Reason is too long.".to_string()
            }
            .to_string()
        )
    }

    // service request slashing with date too far in the past (over max_slashable_delay)
    {
        let slashing_request_payload = RequestSlashingPayload {
            operator: operator.to_string(),
            bips: 100,
            timestamp: app
                .block_info()
                .time
                .minus_seconds(7 * DAYS) // default is 7 days
                .minus_seconds(1),
            metadata: SlashingMetadata {
                reason: "test".to_string(),
            },
        };

        let msg = &ExecuteMsg::RequestSlashing(slashing_request_payload.clone());
        let err = tc
            .vault_router
            .execute(&mut app, &service, msg)
            .unwrap_err();
        assert_eq!(
            err.root_cause().to_string(),
            ContractError::InvalidSlashingRequest {
                msg: "Slash timestamp is outside of the allowable slash period.".to_string()
            }
            .to_string()
        )
    }

    // service request slashing with date in the future
    {
        let slashing_request_payload = RequestSlashingPayload {
            operator: operator.to_string(),
            bips: 100,
            timestamp: app.block_info().time.plus_seconds(1),
            metadata: SlashingMetadata {
                reason: "test".to_string(),
            },
        };

        let msg = &ExecuteMsg::RequestSlashing(slashing_request_payload.clone());
        let err = tc
            .vault_router
            .execute(&mut app, &service, msg)
            .unwrap_err();
        assert_eq!(
            err.root_cause().to_string(),
            ContractError::InvalidSlashingRequest {
                msg: "Slash timestamp is outside of the allowable slash period.".to_string()
            }
            .to_string()
        )
    }

    // service request slashing when operator and service not in ACTIVE status
    {
        // register Service to Operator for OperatorRegistered status
        let msg = &bvs_registry::msg::ExecuteMsg::RegisterServiceToOperator {
            service: service.to_string(),
        };
        tc.registry
            .execute(&mut app, &operator, msg)
            .expect("failed to register service to operator");

        app.update_block(|block| {
            block.height += 1;
            block.time = block.time.plus_seconds(10);
        });

        // service request slashing
        let slashing_request_payload = RequestSlashingPayload {
            operator: operator.to_string(),
            bips: 9999,
            timestamp: app.block_info().time,
            metadata: SlashingMetadata {
                reason: "test".to_string(),
            },
        };

        let msg = &ExecuteMsg::RequestSlashing(slashing_request_payload.clone());
        let err = tc
            .vault_router
            .execute(&mut app, &service, msg)
            .unwrap_err();
        assert_eq!(
            err.root_cause().to_string(),
            ContractError::InvalidSlashingRequest {
                msg: "Service and Operator are not active at timestamp.".to_string()
            }
            .to_string()
        )
    }

    // update registration status to ACTIVE
    {
        // register operator to service for ACTIVE status
        let msg = &bvs_registry::msg::ExecuteMsg::RegisterOperatorToService {
            operator: operator.to_string(),
        };
        tc.registry
            .execute(&mut app, &service, msg)
            .expect("failed to register operator to service");
    }

    app.update_block(|block| {
        block.height += 1;
        block.time = block.time.plus_seconds(10);
    });

    // service request slashing before enabling slashing
    {
        let slashing_request_payload = RequestSlashingPayload {
            operator: operator.to_string(),
            bips: 100,
            timestamp: app.block_info().time,
            metadata: SlashingMetadata {
                reason: "test".to_string(),
            },
        };

        let msg = &ExecuteMsg::RequestSlashing(slashing_request_payload.clone());
        let err = tc
            .vault_router
            .execute(&mut app, &service, msg)
            .unwrap_err();
        assert_eq!(
            err.root_cause().to_string(),
            ContractError::InvalidSlashingRequest {
                msg: "Service has not enabled slashing at timestamp.".to_string()
            }
            .to_string()
        )
    }

    // service enable slashing
    {
        let msg = &bvs_registry::msg::ExecuteMsg::EnableSlashing {
            slashing_parameters: SlashingParameters {
                destination: Some(service.clone()),
                max_slashing_bips: 5000,
                resolution_window: 100,
            },
        };
        tc.registry
            .execute(&mut app, &service, msg)
            .expect("failed to enable slashing");
    }

    app.update_block(|block| {
        block.height += 1;
        block.time = block.time.plus_seconds(10);
    });

    // service request slashing with bips over max_slashing_bips
    {
        let slashing_request_payload = RequestSlashingPayload {
            operator: operator.to_string(),
            bips: 9999,
            timestamp: app.block_info().time,
            metadata: SlashingMetadata {
                reason: "test".to_string(),
            },
        };

        let msg = &ExecuteMsg::RequestSlashing(slashing_request_payload.clone());
        let err = tc
            .vault_router
            .execute(&mut app, &service, msg)
            .unwrap_err();
        assert_eq!(
            err.root_cause().to_string(),
            ContractError::InvalidSlashingRequest {
                msg: "Slashing bips is over max_slashing_bips set.".to_string()
            }
            .to_string()
        )
    }

    // service request slashing before operator opt-in to slashing
    {
        let slashing_request_payload = RequestSlashingPayload {
            operator: operator.to_string(),
            bips: 100,
            timestamp: app.block_info().time,
            metadata: SlashingMetadata {
                reason: "test".to_string(),
            },
        };

        let msg = &ExecuteMsg::RequestSlashing(slashing_request_payload.clone());
        let err = tc
            .vault_router
            .execute(&mut app, &service, msg)
            .unwrap_err();
        assert_eq!(
            err.root_cause().to_string(),
            ContractError::InvalidSlashingRequest {
                msg: "Operator has not opted-in to slashing at timestamp.".to_string()
            }
            .to_string()
        )
    }

    // operator opt-in to slashing
    {
        let msg = &bvs_registry::msg::ExecuteMsg::OperatorOptInToSlashing {
            service: service.to_string(),
        };
        tc.registry
            .execute(&mut app, &operator, msg)
            .expect("failed to opt-in to slashing");
    }

    app.update_block(|block| {
        block.height += 1;
        block.time = block.time.plus_seconds(10);
    });

    // service request slashing for older timestamp before operator opted-in
    {
        let slashing_request_payload = RequestSlashingPayload {
            operator: operator.to_string(),
            bips: 100,
            timestamp: app.block_info().time.minus_seconds(10),
            metadata: SlashingMetadata {
                reason: "test".to_string(),
            },
        };

        let msg = &ExecuteMsg::RequestSlashing(slashing_request_payload.clone());
        let err = tc
            .vault_router
            .execute(&mut app, &service, msg)
            .unwrap_err();
        assert_eq!(
            err.root_cause().to_string(),
            ContractError::InvalidSlashingRequest {
                msg: "Operator has not opted-in to slashing at timestamp.".to_string()
            }
            .to_string()
        )
    }

    // service successfully request slashing
    {
        let slashing_request_payload = RequestSlashingPayload {
            operator: operator.to_string(),
            bips: 100,
            timestamp: app.block_info().time,
            metadata: SlashingMetadata {
                reason: "test".to_string(),
            },
        };

        let msg = &ExecuteMsg::RequestSlashing(slashing_request_payload.clone());
        tc.vault_router
            .execute(&mut app, &service, msg)
            .expect("failed to request slashing");
    }

    app.update_block(|block| {
        block.height += 1;
        block.time = block.time.plus_seconds(10);
    });

    // service request a second slashing to the same operator while there is an pending request
    {
        let slashing_request_payload = RequestSlashingPayload {
            operator: operator.to_string(),
            bips: 100,
            timestamp: app.block_info().time,
            metadata: SlashingMetadata {
                reason: "test".to_string(),
            },
        };

        let msg = &ExecuteMsg::RequestSlashing(slashing_request_payload.clone());
        let err = tc
            .vault_router
            .execute(&mut app, &service, msg)
            .unwrap_err();
        assert_eq!(
            err.root_cause().to_string(),
            ContractError::InvalidSlashingRequest {
                msg: "Previous slashing request is still pending.".to_string()
            }
            .to_string()
        )
    }

    // move blockchain after slashing request expiry
    app.update_block(|block| {
        block.height += 20;
        block.time = block.time.plus_seconds(200); // resolution_window * 2
    });

    // service successfully request slashing after slashing request expiry
    {
        let slashing_request_payload = RequestSlashingPayload {
            operator: operator.to_string(),
            bips: 100,
            timestamp: app.block_info().time,
            metadata: SlashingMetadata {
                reason: "test2".to_string(),
            },
        };

        let msg = &ExecuteMsg::RequestSlashing(slashing_request_payload.clone());
        let response = tc.vault_router.execute(&mut app, &service, msg).unwrap();
        assert_eq!(
            response.events,
            vec![
                Event::new("execute")
                    .add_attribute("_contract_address", tc.vault_router.addr.as_str()),
                Event::new("wasm-RequestSlashing")
                    .add_attribute("_contract_address", tc.vault_router.addr.as_str())
                    .add_attribute("service", service.to_string())
                    .add_attribute("operator", operator.to_string())
                    .add_attribute(
                        "slashing_request_id",
                        "d049decfedeb7ea90c0d4bbe6f068ddbe20729a5a13e81478cf517cc0f86bf3c"
                    )
                    .add_attribute("reason", "test2"),
            ]
        );
    }
}

#[test]
fn test_slash_locking() {
    let (mut app, tc) = TestContracts::init();

    let operator = app.api().addr_make("operator");
    let service = app.api().addr_make("service");

    // register operator + service
    {
        tc.registry
            .execute(
                &mut app,
                &operator,
                &bvs_registry::msg::ExecuteMsg::RegisterAsOperator {
                    metadata: Metadata {
                        name: Some("operator".to_string()),
                        uri: None,
                    },
                },
            )
            .expect("failed to register operator");

        tc.registry
            .execute(
                &mut app,
                &service,
                &bvs_registry::msg::ExecuteMsg::RegisterAsService {
                    metadata: Metadata {
                        name: Some("service".to_string()),
                        uri: None,
                    },
                },
            )
            .expect("failed to register service");

        let owner = app.api().addr_make("owner");

        let msg = &ExecuteMsg::SetVault {
            vault: tc.bank_vault.addr().to_string(),
            whitelisted: true,
        };

        tc.vault_router.execute(&mut app, &owner, msg).unwrap();

        let msg = &ExecuteMsg::SetVault {
            vault: tc.cw20_vault.addr().to_string(),
            whitelisted: true,
        };

        tc.vault_router.execute(&mut app, &owner, msg).unwrap();
    }

    // stake funds
    {
        let owner = app.api().addr_make("owner");
        let denom = "denom";

        let staker = app.api().addr_make("staker");
        let msg = bvs_vault_cw20::msg::ExecuteMsg::DepositFor(RecipientAmount {
            recipient: staker.clone(),
            amount: Uint128::new(300_u128),
        });
        tc.cw20
            .increase_allowance(&mut app, &staker, tc.cw20_vault.addr(), 300_u128);
        tc.cw20.fund(&mut app, &staker, 300_u128);
        tc.cw20_vault.execute(&mut app, &staker, &msg).unwrap();

        // Fund the staker with some initial tokens
        app.send_tokens(owner.clone(), staker.clone(), &coins(1_000_000_000, denom))
            .unwrap();

        // Deposit 115_687_654 tokens from staker to Vault
        let msg = bvs_vault_bank::msg::ExecuteMsg::DepositFor(RecipientAmount {
            recipient: staker.clone(),
            amount: Uint128::new(200),
        });
        tc.bank_vault
            .execute_with_funds(&mut app, &staker, &msg, coins(200, denom))
            .unwrap();

        let bank_vault_info = tc
            .bank_vault
            .query::<bvs_vault_base::msg::VaultInfoResponse>(
                &mut app,
                &bvs_vault_bank::msg::QueryMsg::VaultInfo {},
            )
            .unwrap();
        let cw20_vault_info = tc
            .cw20_vault
            .query::<bvs_vault_base::msg::VaultInfoResponse>(
                &mut app,
                &bvs_vault_bank::msg::QueryMsg::VaultInfo {},
            )
            .unwrap();

        assert_eq!(bank_vault_info.total_assets, Uint128::new(200));
        assert_eq!(cw20_vault_info.total_assets, Uint128::new(300));
    }

    // service enable slashing
    {
        let msg = &bvs_registry::msg::ExecuteMsg::EnableSlashing {
            slashing_parameters: SlashingParameters {
                destination: Some(service.clone()),
                max_slashing_bips: 5000,
                resolution_window: 100,
            },
        };
        tc.registry
            .execute(&mut app, &service, msg)
            .expect("failed to enable slashing");
    }

    app.update_block(|block| {
        block.height += 1;
        block.time = block.time.plus_seconds(10);
    });

    // register operator to service for active status
    {
        let msg = &bvs_registry::msg::ExecuteMsg::RegisterOperatorToService {
            operator: operator.to_string(),
        };
        tc.registry
            .execute(&mut app, &service, msg)
            .expect("failed to register operator to service");

        let msg = &bvs_registry::msg::ExecuteMsg::RegisterServiceToOperator {
            service: service.to_string(),
        };
        tc.registry
            .execute(&mut app, &operator, msg)
            .expect("failed to register service to operator");
    }

    app.update_block(|block| {
        block.height += 1;
        block.time = block.time.plus_seconds(10);
    });

    // Pending slashing requests that are idle beyond expiry
    // should not block new slashing requests
    // and will be transitioned implicitly to cancel status
    let active_slashing_id;
    {
        let slashing_request_payload_1 = RequestSlashingPayload {
            operator: operator.to_string(),
            bips: 100,
            timestamp: app.block_info().time,
            metadata: SlashingMetadata {
                reason: "test".to_string(),
            },
        };
        let msg = &ExecuteMsg::RequestSlashing(slashing_request_payload_1.clone());
        tc.vault_router
            .execute(&mut app, &service, msg)
            .expect("failed to request slashing");

        let slash_req_1_id = tc
            .vault_router
            .query::<SlashingRequestIdResponse>(
                &mut app,
                &QueryMsg::SlashingRequestId {
                    service: service.to_string(),
                    operator: operator.to_string(),
                },
            )
            .unwrap();

        app.update_block(|block| {
            block.height += 60;
            block.time = block.time.plus_seconds(600);
        });

        let slashing_request_payload_2 = RequestSlashingPayload {
            operator: operator.to_string(),
            bips: 100,
            timestamp: app.block_info().time,
            metadata: SlashingMetadata {
                reason: "test".to_string(),
            },
        };
        let msg = &ExecuteMsg::RequestSlashing(slashing_request_payload_2.clone());
        tc.vault_router
            .execute(&mut app, &service, msg)
            .expect("failed to request slashing");

        app.update_block(|block| {
            block.height += 1;
            block.time = block.time.plus_seconds(10);
        });

        let slash_request_1_status = tc
            .vault_router
            .query::<SlashingRequestResponse>(
                &mut app,
                &QueryMsg::SlashingRequest(slash_req_1_id.0.unwrap()),
            )
            .unwrap();

        let slash_request_1_status = slash_request_1_status.0.unwrap().status;

        // The previous slash request should be canceled implicitly
        // by the slash_request handler
        assert_eq!(slash_request_1_status, SlashingRequestStatus::Canceled);

        let slash_req_2_id = tc
            .vault_router
            .query::<SlashingRequestIdResponse>(
                &mut app,
                &QueryMsg::SlashingRequestId {
                    service: service.to_string(),
                    operator: operator.to_string(),
                },
            )
            .unwrap();

        // The second slashing request id is now the current active one
        // using this for later slash_locking tests
        active_slashing_id = slash_req_2_id.0.unwrap();
    }

    {
        // pass the resolution window
        app.update_block(|block| {
            block.height += 10;
            block.time = block.time.plus_seconds(100);
        });

        let msg = ExecuteMsg::LockSlashing(active_slashing_id.clone());
        let res = tc.vault_router.execute(&mut app, &service, &msg).unwrap();

        assert_eq!(
            res.events,
            vec![
                Event::new("execute")
                    .add_attribute("_contract_address", tc.vault_router.addr.as_str()),
                Event::new("wasm-LockSlashing")
                    .add_attribute("_contract_address", tc.vault_router.addr.as_str())
                    .add_attribute("service", service.to_string())
                    .add_attribute("operator", operator.to_string())
                    .add_attribute("slashing_request_id", active_slashing_id.to_string())
                    .add_attribute("bips", "100")
                    .add_attribute("affected_vaults", "2"),
                Event::new("execute")
                    .add_attribute("_contract_address", tc.bank_vault.addr().to_string()),
                Event::new("wasm-SlashLocked")
                    .add_attribute("_contract_address", tc.bank_vault.addr().to_string())
                    .add_attribute("sender", tc.vault_router.addr().to_string())
                    .add_attribute("amount", "2")
                    .add_attribute("denom", "denom"),
                Event::new("transfer")
                    .add_attribute("recipient", tc.vault_router.addr().to_string(),)
                    .add_attribute("sender", tc.bank_vault.addr().to_string(),)
                    .add_attribute("amount", "2denom"),
                Event::new("execute")
                    .add_attribute("_contract_address", tc.cw20_vault.addr().to_string()),
                Event::new("wasm-SlashLocked")
                    .add_attribute("_contract_address", tc.cw20_vault.addr().to_string())
                    .add_attribute("sender", tc.vault_router.addr().to_string())
                    .add_attribute("amount", "3")
                    .add_attribute("token", tc.cw20.addr().to_string()),
                Event::new("execute")
                    .add_attribute("_contract_address", tc.cw20.addr().to_string(),),
                Event::new("wasm")
                    .add_attribute("_contract_address", tc.cw20.addr().to_string())
                    .add_attribute("action", "transfer")
                    .add_attribute("from", tc.cw20_vault.addr().to_string(),)
                    .add_attribute("to", tc.vault_router.addr().to_string(),)
                    .add_attribute("amount", "3"),
            ]
        );

        let bank_vault_info = tc
            .bank_vault
            .query::<bvs_vault_base::msg::VaultInfoResponse>(
                &mut app,
                &bvs_vault_bank::msg::QueryMsg::VaultInfo {},
            )
            .unwrap();
        let cw20_vault_info = tc
            .cw20_vault
            .query::<bvs_vault_base::msg::VaultInfoResponse>(
                &mut app,
                &bvs_vault_bank::msg::QueryMsg::VaultInfo {},
            )
            .unwrap();
        assert_eq!(bank_vault_info.total_assets, Uint128::from(198_u128));
        assert_eq!(cw20_vault_info.total_assets, Uint128::from(297_u128));

        let query = BankQuery::Balance {
            address: tc.vault_router.addr().to_string(),
            denom: "denom".to_string(),
        };

        let router_bank_balance: BalanceResponse =
            app.wrap().query(&QueryRequest::Bank(query)).unwrap();

        assert_eq!(router_bank_balance.amount, coin(2, "denom")); // 1% of 200

        let router_cw20_balance = tc.cw20.balance(&app, tc.vault_router.addr());

        // due to the decimal
        assert_eq!(router_cw20_balance, 3_u128); // 1% of 300
    }
}

#[test]
fn test_finalize_slashing() {
    let (mut app, tc) = TestContracts::init();

    let operator = app.api().addr_make("operator");
    let service = app.api().addr_make("service");

    // register operator + service
    {
        tc.registry
            .execute(
                &mut app,
                &operator,
                &bvs_registry::msg::ExecuteMsg::RegisterAsOperator {
                    metadata: Metadata {
                        name: Some("operator".to_string()),
                        uri: None,
                    },
                },
            )
            .expect("failed to register operator");

        tc.registry
            .execute(
                &mut app,
                &service,
                &bvs_registry::msg::ExecuteMsg::RegisterAsService {
                    metadata: Metadata {
                        name: Some("service".to_string()),
                        uri: None,
                    },
                },
            )
            .expect("failed to register service");

        let owner = app.api().addr_make("owner");

        let msg = &ExecuteMsg::SetVault {
            vault: tc.bank_vault.addr().to_string(),
            whitelisted: true,
        };

        tc.vault_router.execute(&mut app, &owner, msg).unwrap();

        let msg = &ExecuteMsg::SetVault {
            vault: tc.cw20_vault.addr().to_string(),
            whitelisted: true,
        };

        tc.vault_router.execute(&mut app, &owner, msg).unwrap();
    }

    // stake funds
    {
        let owner = app.api().addr_make("owner");
        let denom = "denom";

        let staker = app.api().addr_make("staker");
        tc.cw20
            .increase_allowance(&mut app, &staker, tc.cw20_vault.addr(), 300_u128);
        tc.cw20.fund(&mut app, &staker, 300_u128);
        // staker stake 300 cw20 tokens
        let msg = bvs_vault_cw20::msg::ExecuteMsg::DepositFor(RecipientAmount {
            recipient: staker.clone(),
            amount: Uint128::new(300_u128),
        });
        tc.cw20_vault.execute(&mut app, &staker, &msg).unwrap();

        // fund the staker with some initial bank tokens
        app.send_tokens(owner.clone(), staker.clone(), &coins(1_000_000_000, denom))
            .unwrap();

        // staker stake 200 bank token
        let msg = bvs_vault_bank::msg::ExecuteMsg::DepositFor(RecipientAmount {
            recipient: staker.clone(),
            amount: Uint128::new(200),
        });
        tc.bank_vault
            .execute_with_funds(&mut app, &staker, &msg, coins(200, denom))
            .unwrap();
    }

    // service enable slashing
    {
        let msg = &bvs_registry::msg::ExecuteMsg::EnableSlashing {
            slashing_parameters: SlashingParameters {
                destination: Some(service.clone()),
                max_slashing_bips: 5000,
                resolution_window: 100,
            },
        };
        tc.registry
            .execute(&mut app, &service, msg)
            .expect("failed to enable slashing");
    }

    app.update_block(|block| {
        block.height += 1;
        block.time = block.time.plus_seconds(10);
    });

    // register operator to service and vice versa for active status
    {
        let msg = &bvs_registry::msg::ExecuteMsg::RegisterOperatorToService {
            operator: operator.to_string(),
        };
        tc.registry
            .execute(&mut app, &service, msg)
            .expect("failed to register operator to service");

        let msg = &bvs_registry::msg::ExecuteMsg::RegisterServiceToOperator {
            service: service.to_string(),
        };
        tc.registry
            .execute(&mut app, &operator, msg)
            .expect("failed to register service to operator");
    }

    app.update_block(|block| {
        block.height += 1;
        block.time = block.time.plus_seconds(10);
    });

    // service request slashing
    let slashing_request_payload = RequestSlashingPayload {
        operator: operator.to_string(),
        bips: 100,
        timestamp: app.block_info().time,
        metadata: SlashingMetadata {
            reason: "test".to_string(),
        },
    };

    let msg = &ExecuteMsg::RequestSlashing(slashing_request_payload.clone());
    tc.vault_router.execute(&mut app, &service, msg).unwrap();

    // query slashing request id
    let msg = QueryMsg::SlashingRequestId {
        service: service.to_string(),
        operator: operator.to_string(),
    };
    let SlashingRequestIdResponse(slashing_request_id) =
        tc.vault_router.query(&mut app, &msg).unwrap();
    let slashing_request_id = slashing_request_id.unwrap();

    // pass the resolution window
    app.update_block(|block| {
        block.height += 10;
        block.time = block.time.plus_seconds(100);
    });

    // Lock slashing
    let msg = ExecuteMsg::LockSlashing(slashing_request_id.clone());
    tc.vault_router.execute(&mut app, &service, &msg).unwrap();

    let owner = app.api().addr_make("owner");

    // Create a proposal for the slashing request by owner
    let proposal_msg = bvs_guardrail::msg::ExecuteMsg::Propose {
        slashing_request_id: slashing_request_id.clone(),
        reason: "test slashing".to_string(),
    };
    tc.guardrail
        .execute(&mut app, &owner, &proposal_msg)
        .unwrap();

    // Have voters vote on the proposal
    let voter1 = app.api().addr_make("voter1");
    let voter2 = app.api().addr_make("voter2");
    let voter3 = app.api().addr_make("voter3");
    let voter4 = app.api().addr_make("voter4");

    // Voter1 votes "yes"
    let vote_yes_msg = bvs_guardrail::msg::ExecuteMsg::Vote {
        slashing_request_id: slashing_request_id.clone(),
        vote: cw3::Vote::Yes,
    };
    tc.guardrail
        .execute(&mut app, &voter1, &vote_yes_msg)
        .unwrap();

    // Voter2 votes "yes"
    tc.guardrail
        .execute(&mut app, &voter2, &vote_yes_msg)
        .unwrap();

    // Voter3 votes "no"
    let vote_no_msg = bvs_guardrail::msg::ExecuteMsg::Vote {
        slashing_request_id: slashing_request_id.clone(),
        vote: cw3::Vote::No,
    };
    tc.guardrail
        .execute(&mut app, &voter3, &vote_no_msg)
        .unwrap();

    // Voter4 votes "yes" to pass the proposal
    let vote_msg = bvs_guardrail::msg::ExecuteMsg::Vote {
        slashing_request_id: slashing_request_id.clone(),
        vote: cw3::Vote::Yes,
    };
    tc.guardrail.execute(&mut app, &voter4, &vote_msg).unwrap();

    // Check the proposal status is Passed
    let proposal_status = bvs_guardrail::msg::QueryMsg::ProposalBySlashingRequestId {
        slashing_request_id: slashing_request_id.clone(),
    };
    let proposal: cw3::ProposalResponse = tc.guardrail.query(&app, &proposal_status).unwrap();
    assert_eq!(proposal.status, cw3::Status::Passed);

    // Finalize slashing
    let msg = ExecuteMsg::FinalizeSlashing(slashing_request_id.clone());
    let res = tc.vault_router.execute(&mut app, &service, &msg).unwrap();

    // Verify the slashing is finalized
    // Check events
    assert_eq!(
        res.events,
        vec![
            Event::new("execute").add_attribute("_contract_address", tc.vault_router.addr.as_str()),
            Event::new("wasm-FinalizeSlashing")
                .add_attribute("_contract_address", tc.vault_router.addr.as_str())
                .add_attribute("service", service.to_string())
                .add_attribute("operator", operator)
                .add_attribute("slashing_request_id", slashing_request_id.to_string())
                .add_attribute("destination", service.to_string())
                .add_attribute("affected_vaults", 2.to_string()),
            Event::new("transfer")
                .add_attribute("recipient", service.to_string())
                .add_attribute("sender", tc.vault_router.addr().to_string())
                .add_attribute("amount", "2denom"),
            Event::new("execute").add_attribute("_contract_address", tc.cw20.addr().to_string()),
            Event::new("wasm")
                .add_attribute("_contract_address", tc.cw20.addr().to_string())
                .add_attribute("action", "transfer")
                .add_attribute("from", tc.vault_router.addr().to_string())
                .add_attribute("to", service.to_string())
                .add_attribute("amount", "3"),
        ]
    );

    // Check that the slashed assets are transferred to the destination
    let query = BankQuery::Balance {
        address: service.to_string(),
        denom: "denom".to_string(),
    };
    let service_bank_balance: BalanceResponse =
        app.wrap().query(&QueryRequest::Bank(query)).unwrap();
    assert_eq!(service_bank_balance.amount, coin(2, "denom")); // 1% of 200

    let service_cw20_balance = tc.cw20.balance(&app, &service);
    assert_eq!(service_cw20_balance, 3_u128); // 1% of 300

    // Check that the router no longer has the slashed assets
    let query = BankQuery::Balance {
        address: tc.vault_router.addr().to_string(),
        denom: "denom".to_string(),
    };
    let router_bank_balance: BalanceResponse =
        app.wrap().query(&QueryRequest::Bank(query)).unwrap();
    assert_eq!(router_bank_balance.amount, coin(0, "denom"));

    let router_cw20_balance = tc.cw20.balance(&app, tc.vault_router.addr());
    assert_eq!(router_cw20_balance, 0_u128);

    // Check that the slashing request is marked as finalized
    let msg = QueryMsg::SlashingRequest(slashing_request_id.clone());
    let SlashingRequestResponse(slashing_request) = tc.vault_router.query(&mut app, &msg).unwrap();
    let slashing_request = slashing_request.unwrap();

    assert_eq!(slashing_request.status, SlashingRequestStatus::Finalized);
}

#[test]
fn test_finalize_slashing_negative() {
    let (mut app, tc) = TestContracts::init();

    let owner = app.api().addr_make("owner");
    let operator = app.api().addr_make("operator");
    let service = app.api().addr_make("service");

    // register operator + service
    {
        tc.registry
            .execute(
                &mut app,
                &operator,
                &bvs_registry::msg::ExecuteMsg::RegisterAsOperator {
                    metadata: Metadata {
                        name: Some("operator".to_string()),
                        uri: None,
                    },
                },
            )
            .expect("failed to register operator");

        tc.registry
            .execute(
                &mut app,
                &service,
                &bvs_registry::msg::ExecuteMsg::RegisterAsService {
                    metadata: Metadata {
                        name: Some("service".to_string()),
                        uri: None,
                    },
                },
            )
            .expect("failed to register service");

        let owner = app.api().addr_make("owner");

        let msg = &ExecuteMsg::SetVault {
            vault: tc.bank_vault.addr().to_string(),
            whitelisted: true,
        };

        tc.vault_router.execute(&mut app, &owner, msg).unwrap();

        let msg = &ExecuteMsg::SetVault {
            vault: tc.cw20_vault.addr().to_string(),
            whitelisted: true,
        };

        tc.vault_router.execute(&mut app, &owner, msg).unwrap();
    }

    // stake funds
    {
        let owner = app.api().addr_make("owner");
        let denom = "denom";

        let staker = app.api().addr_make("staker");
        tc.cw20
            .increase_allowance(&mut app, &staker, tc.cw20_vault.addr(), 300_u128);
        tc.cw20.fund(&mut app, &staker, 300_u128);
        // staker stake 300 cw20 tokens
        let msg = bvs_vault_cw20::msg::ExecuteMsg::DepositFor(RecipientAmount {
            recipient: staker.clone(),
            amount: Uint128::new(300_u128),
        });
        tc.cw20_vault.execute(&mut app, &staker, &msg).unwrap();

        // fund the staker with some initial bank tokens
        app.send_tokens(owner.clone(), staker.clone(), &coins(1_000_000_000, denom))
            .unwrap();

        // staker stake 200 bank token
        let msg = bvs_vault_bank::msg::ExecuteMsg::DepositFor(RecipientAmount {
            recipient: staker.clone(),
            amount: Uint128::new(200),
        });
        tc.bank_vault
            .execute_with_funds(&mut app, &staker, &msg, coins(200, denom))
            .unwrap();
    }

    // service enable slashing
    {
        let msg = &bvs_registry::msg::ExecuteMsg::EnableSlashing {
            slashing_parameters: SlashingParameters {
                destination: Some(service.clone()),
                max_slashing_bips: 5000,
                resolution_window: 100,
            },
        };
        tc.registry
            .execute(&mut app, &service, msg)
            .expect("failed to enable slashing");
    }

    app.update_block(|block| {
        block.height += 1;
        block.time = block.time.plus_seconds(10);
    });

    // register operator to service and vice versa for active status
    {
        let msg = &bvs_registry::msg::ExecuteMsg::RegisterOperatorToService {
            operator: operator.to_string(),
        };
        tc.registry
            .execute(&mut app, &service, msg)
            .expect("failed to register operator to service");

        let msg = &bvs_registry::msg::ExecuteMsg::RegisterServiceToOperator {
            service: service.to_string(),
        };
        tc.registry
            .execute(&mut app, &operator, msg)
            .expect("failed to register service to operator");
    }

    app.update_block(|block| {
        block.height += 1;
        block.time = block.time.plus_seconds(10);
    });

    // service request slashing
    let slashing_request_payload = RequestSlashingPayload {
        operator: operator.to_string(),
        bips: 100,
        timestamp: app.block_info().time,
        metadata: SlashingMetadata {
            reason: "test".to_string(),
        },
    };

    let msg = &ExecuteMsg::RequestSlashing(slashing_request_payload.clone());
    tc.vault_router.execute(&mut app, &service, msg).unwrap();

    // query slashing request id
    let msg = QueryMsg::SlashingRequestId {
        service: service.to_string(),
        operator: operator.to_string(),
    };
    let SlashingRequestIdResponse(slashing_request_id) =
        tc.vault_router.query(&mut app, &msg).unwrap();
    let slashing_request_id = slashing_request_id.unwrap();

    // create guardrail proposal
    {
        let msg = bvs_guardrail::msg::ExecuteMsg::Propose {
            slashing_request_id: slashing_request_id.clone(),
            reason: "test".to_string(),
        };
        tc.guardrail.execute(&mut app, &owner, &msg).unwrap();
    }

    // Negative - fails to Finalize slashing
    {
        let msg = ExecuteMsg::FinalizeSlashing(slashing_request_id.clone());
        let err = tc
            .vault_router
            .execute(&mut app, &service, &msg)
            .unwrap_err();
        assert_eq!(
            err.root_cause().to_string(),
            ContractError::InvalidSlashingRequest {
                msg: "Slashing request has not passed the guardrail".to_string()
            }
            .to_string()
        );
    }

    // pass the resolution window
    app.update_block(|block| {
        block.height += 10;
        block.time = block.time.plus_seconds(100);
    });

    // Pass guardrail vote
    {
        let voter1 = app.api().addr_make("voter1");
        let voter2 = app.api().addr_make("voter2");
        let voter3 = app.api().addr_make("voter3");

        let msg = bvs_guardrail::msg::ExecuteMsg::Vote {
            slashing_request_id: slashing_request_id.clone(),
            vote: cw3::Vote::Yes,
        };
        tc.guardrail.execute(&mut app, &voter1, &msg).unwrap();
        tc.guardrail.execute(&mut app, &voter2, &msg).unwrap();
        tc.guardrail.execute(&mut app, &voter3, &msg).unwrap();
    }

    // Negative - Finalize slashing passed guardrail but fails due to request not locked
    {
        let msg = ExecuteMsg::FinalizeSlashing(slashing_request_id.clone());
        let err = tc
            .vault_router
            .execute(&mut app, &service, &msg)
            .unwrap_err();
        assert_eq!(
            err.root_cause().to_string(),
            ContractError::InvalidSlashingRequest {
                msg: "Slashing request is not locked".to_string()
            }
            .to_string()
        );
    }

    // Lock slashing
    let msg = ExecuteMsg::LockSlashing(slashing_request_id.clone());
    tc.vault_router.execute(&mut app, &service, &msg).unwrap();

    // pass the guardrail proposal expiry
    app.update_block(|block| {
        block.height += 100;
        block.time = block.time.plus_seconds(1000);
    });

    // check that guardrail proposal is still passed
    let proposal_status = bvs_guardrail::msg::QueryMsg::ProposalBySlashingRequestId {
        slashing_request_id: slashing_request_id.clone(),
    };
    let proposal: cw3::ProposalResponse = tc.guardrail.query(&app, &proposal_status).unwrap();
    assert_eq!(proposal.status, cw3::Status::Passed);

    // Negative - Other addr calls finalize
    {
        let msg = ExecuteMsg::FinalizeSlashing(slashing_request_id.clone());
        let err = tc.vault_router.execute(&mut app, &owner, &msg).unwrap_err();
        assert_eq!(
            err.root_cause().to_string(),
            ContractError::Unauthorized {
                msg: "Only the service that requested slashing can finalize it".to_string()
            }
            .to_string()
        );
    }

    // Positive - service disable slashing, should not affect finalize slashing
    {
        let msg = &bvs_registry::msg::ExecuteMsg::DisableSlashing {};
        tc.registry
            .execute(&mut app, &service, msg)
            .expect("failed to disable slashing");
    }

    // Finalize slashing should go through
    let msg = ExecuteMsg::FinalizeSlashing(slashing_request_id.clone());
    let res = tc.vault_router.execute(&mut app, &service, &msg).unwrap();

    // Verify the slashing is finalized
    // Check events
    assert_eq!(
        res.events,
        vec![
            Event::new("execute").add_attribute("_contract_address", tc.vault_router.addr.as_str()),
            Event::new("wasm-FinalizeSlashing")
                .add_attribute("_contract_address", tc.vault_router.addr.as_str())
                .add_attribute("service", service.to_string())
                .add_attribute("operator", operator)
                .add_attribute("slashing_request_id", slashing_request_id.to_string())
                .add_attribute("destination", service.to_string())
                .add_attribute("affected_vaults", 2.to_string()),
            Event::new("transfer")
                .add_attribute("recipient", service.to_string())
                .add_attribute("sender", tc.vault_router.addr().to_string())
                .add_attribute("amount", "2denom"),
            Event::new("execute").add_attribute("_contract_address", tc.cw20.addr().to_string()),
            Event::new("wasm")
                .add_attribute("_contract_address", tc.cw20.addr().to_string())
                .add_attribute("action", "transfer")
                .add_attribute("from", tc.vault_router.addr().to_string())
                .add_attribute("to", service.to_string())
                .add_attribute("amount", "3"),
        ]
    );

    // Check that the slashed assets are transferred to the destination
    let query = BankQuery::Balance {
        address: service.to_string(),
        denom: "denom".to_string(),
    };
    let service_bank_balance: BalanceResponse =
        app.wrap().query(&QueryRequest::Bank(query)).unwrap();
    assert_eq!(service_bank_balance.amount, coin(2, "denom")); // 1% of 200

    let service_cw20_balance = tc.cw20.balance(&app, &service);
    assert_eq!(service_cw20_balance, 3_u128); // 1% of 300

    // Check that the router no longer has the slashed assets
    let query = BankQuery::Balance {
        address: tc.vault_router.addr().to_string(),
        denom: "denom".to_string(),
    };
    let router_bank_balance: BalanceResponse =
        app.wrap().query(&QueryRequest::Bank(query)).unwrap();
    assert_eq!(router_bank_balance.amount, coin(0, "denom"));

    let router_cw20_balance = tc.cw20.balance(&app, tc.vault_router.addr());
    assert_eq!(router_cw20_balance, 0_u128);

    // Check that the slashing request is marked as finalized
    let msg = QueryMsg::SlashingRequest(slashing_request_id.clone());
    let SlashingRequestResponse(slashing_request) = tc.vault_router.query(&mut app, &msg).unwrap();
    let slashing_request = slashing_request.unwrap();

    assert_eq!(slashing_request.status, SlashingRequestStatus::Finalized);
}

#[test]
fn test_slash_locking_negative() {
    let (mut app, tc) = TestContracts::init();

    let operator = app.api().addr_make("operator");
    let service = app.api().addr_make("service");

    // register operator + service
    {
        tc.registry
            .execute(
                &mut app,
                &operator,
                &bvs_registry::msg::ExecuteMsg::RegisterAsOperator {
                    metadata: Metadata {
                        name: Some("operator".to_string()),
                        uri: None,
                    },
                },
            )
            .expect("failed to register operator");

        tc.registry
            .execute(
                &mut app,
                &service,
                &bvs_registry::msg::ExecuteMsg::RegisterAsService {
                    metadata: Metadata {
                        name: Some("service".to_string()),
                        uri: None,
                    },
                },
            )
            .expect("failed to register service");

        let owner = app.api().addr_make("owner");

        let msg = &ExecuteMsg::SetVault {
            vault: tc.bank_vault.addr().to_string(),
            whitelisted: true,
        };

        tc.vault_router.execute(&mut app, &owner, msg).unwrap();

        let msg = &ExecuteMsg::SetVault {
            vault: tc.cw20_vault.addr().to_string(),
            whitelisted: true,
        };

        tc.vault_router.execute(&mut app, &owner, msg).unwrap();
    }

    // stake funds but only to bank vault
    {
        let owner = app.api().addr_make("owner");
        let denom = "denom";

        let staker = app.api().addr_make("staker");

        // Fund the staker with some initial tokens
        app.send_tokens(owner.clone(), staker.clone(), &coins(1_000_000_000, denom))
            .unwrap();

        // Deposit 115_687_654 tokens from staker to Vault
        let msg = bvs_vault_bank::msg::ExecuteMsg::DepositFor(RecipientAmount {
            recipient: staker.clone(),
            amount: Uint128::new(200),
        });
        tc.bank_vault
            .execute_with_funds(&mut app, &staker, &msg, coins(200, denom))
            .unwrap();

        let bank_vault_info = tc
            .bank_vault
            .query::<bvs_vault_base::msg::VaultInfoResponse>(
                &mut app,
                &bvs_vault_bank::msg::QueryMsg::VaultInfo {},
            )
            .unwrap();
        let cw20_vault_info = tc
            .cw20_vault
            .query::<bvs_vault_base::msg::VaultInfoResponse>(
                &mut app,
                &bvs_vault_bank::msg::QueryMsg::VaultInfo {},
            )
            .unwrap();

        assert_eq!(bank_vault_info.total_assets, Uint128::new(200));
        assert_eq!(cw20_vault_info.total_assets, Uint128::new(0));
    }

    // service enable slashing
    {
        let msg = &bvs_registry::msg::ExecuteMsg::EnableSlashing {
            slashing_parameters: SlashingParameters {
                destination: Some(service.clone()),
                max_slashing_bips: 5000,
                resolution_window: 100,
            },
        };
        tc.registry
            .execute(&mut app, &service, msg)
            .expect("failed to enable slashing");
    }

    app.update_block(|block| {
        block.height += 1;
        block.time = block.time.plus_seconds(10);
    });

    // register operator to service for active status
    {
        let msg = &bvs_registry::msg::ExecuteMsg::RegisterOperatorToService {
            operator: operator.to_string(),
        };
        tc.registry
            .execute(&mut app, &service, msg)
            .expect("failed to register operator to service");

        let msg = &bvs_registry::msg::ExecuteMsg::RegisterServiceToOperator {
            service: service.to_string(),
        };
        tc.registry
            .execute(&mut app, &operator, msg)
            .expect("failed to register service to operator");
    }

    app.update_block(|block| {
        block.height += 1;
        block.time = block.time.plus_seconds(10);
    });

    //expired slash should get canceled
    {
        let slashing_request_payload = RequestSlashingPayload {
            operator: operator.to_string(),
            bips: 100,
            timestamp: app.block_info().time,
            metadata: SlashingMetadata {
                reason: "expired test".to_string(),
            },
        };

        let msg = &ExecuteMsg::RequestSlashing(slashing_request_payload.clone());
        tc.vault_router.execute(&mut app, &service, msg).unwrap();

        let msg = QueryMsg::SlashingRequestId {
            service: service.to_string(),
            operator: operator.to_string(),
        };
        let slashing_request_id: SlashingRequestIdResponse =
            tc.vault_router.query(&mut app, &msg).unwrap();

        // aged the slash entry to be expired
        app.update_block(|block| {
            block.height += 80;
            block.time = block.time.plus_seconds(800);
        });

        let msg = ExecuteMsg::LockSlashing(slashing_request_id.clone().0.unwrap());
        let res = tc
            .vault_router
            .execute(&mut app, &service, &msg)
            .unwrap_err();
        assert_eq!(
            res.root_cause().to_string(),
            ContractError::InvalidSlashingRequest {
                msg: "Slashing has expired".to_string(),
            }
            .to_string()
        );
    }

    app.update_block(|block| {
        block.height += 1;
        block.time = block.time.plus_seconds(10);
    });

    // service request slashing
    let slashing_request_payload = RequestSlashingPayload {
        operator: operator.to_string(),
        bips: 100,
        timestamp: app.block_info().time,
        metadata: SlashingMetadata {
            reason: "test".to_string(),
        },
    };

    let msg = &ExecuteMsg::RequestSlashing(slashing_request_payload.clone());
    tc.vault_router.execute(&mut app, &service, msg).unwrap();

    let msg = QueryMsg::SlashingRequestId {
        service: service.to_string(),
        operator: operator.to_string(),
    };
    let slashing_request_id1: SlashingRequestIdResponse =
        tc.vault_router.query(&mut app, &msg).unwrap();

    {
        // the slash hasn't aged for resolution_window yet
        // so it should fail
        let msg = ExecuteMsg::LockSlashing(slashing_request_id1.clone().0.unwrap());
        let res = tc
            .vault_router
            .execute(&mut app, &service, &msg)
            .unwrap_err();
        assert_eq!(
            res.root_cause().to_string(),
            ContractError::InvalidSlashingRequest {
                msg: "Slashing cannot be locked until resolution time has elapsed".to_string(),
            }
            .to_string()
        );
    }

    {
        // Unauthorized slash locker
        let rogue_service = app.api().addr_make("rogue_service");
        let msg = ExecuteMsg::LockSlashing(slashing_request_id1.clone().0.unwrap());
        let res = tc
            .vault_router
            .execute(&mut app, &rogue_service, &msg)
            .unwrap_err();
        assert_eq!(
            res.root_cause().to_string(),
            ContractError::Unauthorized {
                msg: "Slash locking is restricted to the service that initiated the request."
                    .to_string(),
            }
            .to_string()
        );
    }

    // cw20 vault has zero asset
    // slash lock should skip over that vault and only slash bank
    // The whole slashing should not fail just because a vault is zero.
    {
        app.update_block(|block| {
            block.height += 10;
            block.time = block.time.plus_seconds(100);
        });

        let msg = ExecuteMsg::LockSlashing(slashing_request_id1.clone().0.unwrap());
        tc.vault_router.execute(&mut app, &service, &msg).unwrap();

        let bank_vault_info = tc
            .bank_vault
            .query::<bvs_vault_base::msg::VaultInfoResponse>(
                &mut app,
                &bvs_vault_bank::msg::QueryMsg::VaultInfo {},
            )
            .unwrap();
        let cw20_vault_info = tc
            .cw20_vault
            .query::<bvs_vault_base::msg::VaultInfoResponse>(
                &mut app,
                &bvs_vault_bank::msg::QueryMsg::VaultInfo {},
            )
            .unwrap();
        assert_eq!(bank_vault_info.total_assets, Uint128::from(198_u128));
        assert_eq!(cw20_vault_info.total_assets, Uint128::from(0_u128));

        let query = BankQuery::Balance {
            address: tc.vault_router.addr().to_string(),
            denom: "denom".to_string(),
        };

        let router_bank_balance: BalanceResponse =
            app.wrap().query(&QueryRequest::Bank(query)).unwrap();

        assert_eq!(router_bank_balance.amount, coin(2, "denom"));

        let router_cw20_balance = tc.cw20.balance(&app, tc.vault_router.addr());

        // due to the decimal
        assert_eq!(router_cw20_balance, 0_u128);
    }

    // slash replay should fail
    {
        let msg = ExecuteMsg::LockSlashing(slashing_request_id1.clone().0.unwrap());
        let res = tc
            .vault_router
            .execute(&mut app, &service, &msg)
            .unwrap_err();
        assert_eq!(
            res.root_cause().to_string(),
            ContractError::InvalidSlashingRequest {
                msg: "Slashing request is already locked".to_string(),
            }
            .to_string()
        );
    }

    app.update_block(|block| {
        block.height += 60;
        block.time = block.time.plus_seconds(600);
    });

    {
        let slashing_request_payload = RequestSlashingPayload {
            operator: operator.to_string(),
            bips: 100,
            timestamp: app.block_info().time,
            metadata: SlashingMetadata {
                reason: "test".to_string(),
            },
        };

        let msg = &ExecuteMsg::RequestSlashing(slashing_request_payload.clone());
        let err = tc
            .vault_router
            .execute(&mut app, &service, msg)
            .unwrap_err();
        assert_eq!(
            err.root_cause().to_string(),
            ContractError::InvalidSlashingRequest {
                msg: "Previous slashing request is in progress.".to_string(),
            }
            .to_string()
        );
    }
}

#[test]
fn cancel_slashing_successful() {
    let (mut app, tc) = TestContracts::init();

    let operator = app.api().addr_make("operator");
    let service = app.api().addr_make("service");

    // register operator + service
    {
        tc.registry
            .execute(
                &mut app,
                &operator,
                &bvs_registry::msg::ExecuteMsg::RegisterAsOperator {
                    metadata: Metadata {
                        name: Some("operator".to_string()),
                        uri: None,
                    },
                },
            )
            .expect("failed to register operator");

        tc.registry
            .execute(
                &mut app,
                &service,
                &bvs_registry::msg::ExecuteMsg::RegisterAsService {
                    metadata: Metadata {
                        name: Some("service".to_string()),
                        uri: None,
                    },
                },
            )
            .expect("failed to register service");
    }

    // service enable slashing
    {
        let msg = &bvs_registry::msg::ExecuteMsg::EnableSlashing {
            slashing_parameters: SlashingParameters {
                destination: Some(service.clone()),
                max_slashing_bips: 5000,
                resolution_window: 100,
            },
        };
        tc.registry
            .execute(&mut app, &service, msg)
            .expect("failed to enable slashing");
    }

    app.update_block(|block| {
        block.height += 1;
        block.time = block.time.plus_seconds(10);
    });

    // register operator to service for active status
    {
        let msg = &bvs_registry::msg::ExecuteMsg::RegisterOperatorToService {
            operator: operator.to_string(),
        };
        tc.registry
            .execute(&mut app, &service, msg)
            .expect("failed to register operator to service");

        let msg = &bvs_registry::msg::ExecuteMsg::RegisterServiceToOperator {
            service: service.to_string(),
        };
        tc.registry
            .execute(&mut app, &operator, msg)
            .expect("failed to register service to operator");
    }

    app.update_block(|block| {
        block.height += 1;
        block.time = block.time.plus_seconds(10);
    });

    // request slashing
    {
        // service request slashing
        let slashing_request_payload = RequestSlashingPayload {
            operator: operator.to_string(),
            bips: 100,
            timestamp: app.block_info().time,
            metadata: SlashingMetadata {
                reason: "test".to_string(),
            },
        };

        let msg = &ExecuteMsg::RequestSlashing(slashing_request_payload.clone());
        tc.vault_router
            .execute(&mut app, &service, msg)
            .expect("failed to request slashing");
    }

    // query slashing request id
    let msg = QueryMsg::SlashingRequestId {
        service: service.to_string(),
        operator: operator.to_string(),
    };
    let slashing_request_id_response: SlashingRequestIdResponse =
        tc.vault_router.query(&mut app, &msg).unwrap();
    let slashing_request_id = slashing_request_id_response.0.unwrap();

    // cancel slashing request
    let msg = &ExecuteMsg::CancelSlashing(slashing_request_id.clone());

    let response = tc.vault_router.execute(&mut app, &service, msg).unwrap();
    assert_eq!(
        response.events,
        vec![
            Event::new("execute").add_attribute("_contract_address", tc.vault_router.addr.as_str()),
            Event::new("wasm-CancelSlashing")
                .add_attribute("_contract_address", tc.vault_router.addr.as_str())
                .add_attribute("service", service.to_string())
                .add_attribute("operator", operator.to_string())
                .add_attribute("slashing_request_id", slashing_request_id.to_string())
        ]
    );

    // query slashing request
    let msg = QueryMsg::SlashingRequest(slashing_request_id);
    let slashing_request_response: SlashingRequestResponse =
        tc.vault_router.query(&mut app, &msg).unwrap();
    let slashing_request = slashing_request_response.0.unwrap();
    assert_eq!(slashing_request.status, SlashingRequestStatus::Canceled);
}

/// Test cancel slashing error cases
#[test]
fn cancel_slashing_error_cases() {
    let (mut app, tc) = TestContracts::init();

    let operator = app.api().addr_make("operator");
    let service = app.api().addr_make("service");

    // register operator + service
    {
        tc.registry
            .execute(
                &mut app,
                &operator,
                &bvs_registry::msg::ExecuteMsg::RegisterAsOperator {
                    metadata: Metadata {
                        name: Some("operator".to_string()),
                        uri: None,
                    },
                },
            )
            .expect("failed to register operator");

        tc.registry
            .execute(
                &mut app,
                &service,
                &bvs_registry::msg::ExecuteMsg::RegisterAsService {
                    metadata: Metadata {
                        name: Some("service".to_string()),
                        uri: None,
                    },
                },
            )
            .expect("failed to register service");
    }

    // service enable slashing
    {
        let msg = &bvs_registry::msg::ExecuteMsg::EnableSlashing {
            slashing_parameters: SlashingParameters {
                destination: Some(service.clone()),
                max_slashing_bips: 5000,
                resolution_window: 100,
            },
        };
        tc.registry
            .execute(&mut app, &service, msg)
            .expect("failed to enable slashing");
    }

    app.update_block(|block| {
        block.height += 1;
        block.time = block.time.plus_seconds(10);
    });

    // register operator to service for active status
    {
        let msg = &bvs_registry::msg::ExecuteMsg::RegisterOperatorToService {
            operator: operator.to_string(),
        };
        tc.registry
            .execute(&mut app, &service, msg)
            .expect("failed to register operator to service");

        let msg = &bvs_registry::msg::ExecuteMsg::RegisterServiceToOperator {
            service: service.to_string(),
        };
        tc.registry
            .execute(&mut app, &operator, msg)
            .expect("failed to register service to operator");
    }

    app.update_block(|block| {
        block.height += 1;
        block.time = block.time.plus_seconds(10);
    });

    let hex_bin =
        HexBinary::from_hex("3902889975800375703a50bbe0d7a5c297977cb44348bf991cca43594fc644ef")
            .unwrap();
    let invalid_slashing_request_id = SlashingRequestId(hex_bin);

    // No slashing request found to cancel slash
    {
        let msg = &ExecuteMsg::CancelSlashing(invalid_slashing_request_id.clone());
        let err = tc
            .vault_router
            .execute(&mut app, &service, msg)
            .unwrap_err();

        assert_eq!(
            err.root_cause().to_string(),
            ContractError::InvalidSlashingRequest {
                msg: "No slashing request found by slashing request id".to_string()
            }
            .to_string()
        );
    }

    // request slashing
    {
        let slashing_request_payload = RequestSlashingPayload {
            operator: operator.to_string(),
            bips: 100,
            timestamp: app.block_info().time,
            metadata: SlashingMetadata {
                reason: "test".to_string(),
            },
        };

        let msg = &ExecuteMsg::RequestSlashing(slashing_request_payload.clone());
        tc.vault_router
            .execute(&mut app, &service, msg)
            .expect("failed to request slashing");
    }

    // invalid info.sender: cancel slashing request isn't from service
    {
        // query slashing request id
        let msg = QueryMsg::SlashingRequestId {
            service: service.to_string(),
            operator: operator.to_string(),
        };
        let slashing_request_id_response: SlashingRequestIdResponse =
            tc.vault_router.query(&mut app, &msg).unwrap();
        let slashing_request_id = slashing_request_id_response.0.unwrap();

        // cancel slashing request
        let msg = &ExecuteMsg::CancelSlashing(slashing_request_id.clone());

        let err = tc
            .vault_router
            .execute(&mut app, &operator.clone(), msg)
            .unwrap_err();

        assert_eq!(
            err.root_cause().to_string(),
            ContractError::InvalidSlashingRequest {
                msg: "Invalid service sends a cancel slashing request".to_string()
            }
            .to_string()
        );
    }
}
