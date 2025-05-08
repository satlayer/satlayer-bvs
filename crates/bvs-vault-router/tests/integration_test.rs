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
    RequestSlashingPayload, SlashingMetadata, SlashingRequestIdResponse, SlashingRequestResponse,
    Vault,
};
use bvs_vault_router::state::SlashingRequestStatus;
use bvs_vault_router::{
    msg::{ExecuteMsg, QueryMsg, VaultListResponse},
    testing::VaultRouterContract,
    ContractError,
};
use cosmwasm_std::{coin, coins, BalanceResponse, BankQuery, QueryRequest, Uint128};
use cosmwasm_std::{from_json, testing::mock_env, Binary, Event, HexBinary, Uint64};
use cw_multi_test::{App, Executor};

struct TestContracts {
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

        let _ = PauserContract::new(&mut app, &env, None);
        let registry = RegistryContract::new(&mut app, &env, None);
        let vault_router = VaultRouterContract::new(&mut app, &env, None);
        let bank_vault = VaultBankContract::new(&mut app, &env, None);
        let cw20 = Cw20TokenContract::new(&mut app, &env, None);
        let cw20_vault = VaultCw20Contract::new(&mut app, &env, None);

        (
            app,
            Self {
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
                    "e99316f1087d1365c4e1c4a2d82de63c4029cd51cd7b6a1bccd42bfbad9d310d"
                )
                .add_attribute("reason", "test"),
        ]
    );

    // Needed due to bug in cw-multi-test https://github.com/CosmWasm/cw-multi-test/issues/257
    use bvs_vault_router::state::SlashingRequest;
    use prost::Message;

    #[derive(Clone, PartialEq, Message)]
    struct ExecuteResponse {
        #[prost(bytes, tag = "1")]
        pub data: Vec<u8>,
    }

    // TODO: fix assert response.data eq to slashing_id
    let slashing_id_bin: Binary = response.data.unwrap();
    let slashing_id_bin2 = ExecuteResponse::decode(slashing_id_bin.clone().as_slice()).unwrap();
    let slashing_id = from_json::<Option<SlashingRequestIdResponse>>(slashing_id_bin2.data);
    let SlashingRequestIdResponse(res) = slashing_id.unwrap().unwrap();
    assert_eq!(
        res,
        Some(
            HexBinary::from_hex("e99316f1087d1365c4e1c4a2d82de63c4029cd51cd7b6a1bccd42bfbad9d310d")
                .unwrap()
                .into()
        )
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
            HexBinary::from_hex("e99316f1087d1365c4e1c4a2d82de63c4029cd51cd7b6a1bccd42bfbad9d310d")
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
                msg: "Service has current pending slashing request for the operator.".to_string()
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
                        "9e2a9d4382cc5e9f3aaf99215def962ce58595ed93107a3ea052ce75b6a2249c"
                    )
                    .add_attribute("reason", "test2"),
            ]
        );
    }
}

#[test]
fn teste_slash_locking() {
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
                    "cdb763a239cb5c8d627c5cc85c65aac291680aced9d081ba7595c6d5138fc4fb"
                )
                .add_attribute("reason", "test"),
        ]
    );

    // query slashing request id
    let msg = QueryMsg::SlashingRequestId {
        service: service.to_string(),
        operator: operator.to_string(),
    };
    let slashing_request_id: SlashingRequestIdResponse =
        tc.vault_router.query(&mut app, &msg).unwrap();

    {
        let msg = ExecuteMsg::SlashLocked(slashing_request_id.clone().0.unwrap());
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
                    "cdb763a239cb5c8d627c5cc85c65aac291680aced9d081ba7595c6d5138fc4fb"
                )
                .add_attribute("reason", "test"),
        ]
    );

    let msg = QueryMsg::SlashingRequestId {
        service: service.to_string(),
        operator: operator.to_string(),
    };
    let slashing_request_id: SlashingRequestIdResponse =
        tc.vault_router.query(&mut app, &msg).unwrap();

    // cw20 vault has zero asset
    // slash lock should skip over that vault and only slash bank
    // The whole slashing should not fail just because a vault is zero.
    {
        let msg = ExecuteMsg::SlashLocked(slashing_request_id.clone().0.unwrap());
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
}
