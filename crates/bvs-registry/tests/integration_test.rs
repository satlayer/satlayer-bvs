use bvs_library::testing::TestingContract;
use bvs_pauser::api::PauserError;
use bvs_pauser::testing::PauserContract;
use bvs_registry::msg::{
    ExecuteMsg, IsOperatorOptedInToSlashingResponse, Metadata, QueryMsg, StatusResponse,
};
use bvs_registry::testing::RegistryContract;
use bvs_registry::{ContractError, RegistrationStatus, SlashingParameters};
use cosmwasm_std::testing::mock_env;
use cosmwasm_std::{Addr, Event, StdError};
use cw_multi_test::App;
use cw_storage_plus::Map;

fn instantiate() -> (App, RegistryContract, PauserContract) {
    let mut app = App::default();
    let env = mock_env();

    let pauser = PauserContract::new(&mut app, &env, None);
    let registry = RegistryContract::new(&mut app, &env, None);

    (app, registry, pauser)
}

#[test]
fn register_service_successfully() {
    let (mut app, registry, ..) = instantiate();

    let register_msg = &ExecuteMsg::RegisterAsService {
        metadata: Metadata {
            name: Some("Service Name".to_string()),
            uri: Some("https://service.com".to_string()),
        },
    };

    let service = app.api().addr_make("service/11111");
    let response = registry.execute(&mut app, &service, register_msg).unwrap();

    assert_eq!(
        response.events,
        vec![
            Event::new("execute").add_attribute("_contract_address", registry.addr.as_str()),
            Event::new("wasm-ServiceRegistered")
                .add_attribute("_contract_address", registry.addr.as_str())
                .add_attribute("service", service.as_str()),
            Event::new("wasm-MetadataUpdated")
                .add_attribute("_contract_address", registry.addr.as_str())
                .add_attribute("metadata.uri", "https://service.com")
                .add_attribute("metadata.name", "Service Name")
                .add_attribute("service", service.as_str())
        ]
    );
}

#[test]
fn register_service_but_paused() {
    let (mut app, registry, pauser) = instantiate();
    let owner = app.api().addr_make("owner");

    let register_msg = &ExecuteMsg::RegisterAsService {
        metadata: Metadata {
            name: Some("Service Name".to_string()),
            uri: Some("https://service.com".to_string()),
        },
    };

    pauser
        .execute(
            &mut app,
            &owner,
            &bvs_pauser::msg::ExecuteMsg::Pause {
                method: "RegisterAsService".to_string(),
                contract: registry.addr.to_string(),
            },
        )
        .unwrap();

    let err = registry
        .execute(&mut app, &owner, register_msg)
        .unwrap_err();

    assert_eq!(
        err.root_cause().to_string(),
        ContractError::Pauser(PauserError::IsPaused).to_string()
    );
}

#[test]
fn register_service_but_already_registered() {
    let (mut app, registry, ..) = instantiate();

    let register_msg = &ExecuteMsg::RegisterAsService {
        metadata: Metadata {
            name: Some("Service Name".to_string()),
            uri: Some("https://service.com".to_string()),
        },
    };

    let service = app.api().addr_make("service/11111");
    registry.execute(&mut app, &service, register_msg).unwrap();

    let err = registry
        .execute(&mut app, &service, register_msg)
        .unwrap_err();

    assert_eq!(
        err.root_cause().to_string(),
        ContractError::ServiceRegistered {}.to_string()
    );
}

#[test]
fn operator_register_service_but_service_not_registered() {
    let (mut app, registry, _) = instantiate();
    let operator = app.api().addr_make("operator");

    let register_msg = &ExecuteMsg::RegisterServiceToOperator {
        service: app.api().addr_make("service/11111").to_string(),
    };

    let err = registry
        .execute(&mut app, &operator, register_msg)
        .unwrap_err();

    assert_eq!(
        err.root_cause().to_string(),
        ContractError::Std(StdError::not_found("service")).to_string()
    );
}

#[test]
fn operator_register_service_but_self_not_operator() {
    let (mut app, registry, _) = instantiate();
    let not_operator = app.api().addr_make("not_operator");

    let register_msg = &ExecuteMsg::RegisterAsService {
        metadata: Metadata {
            name: Some("Service Name".to_string()),
            uri: Some("https://service.com".to_string()),
        },
    };

    let service = app.api().addr_make("service/11111");
    registry.execute(&mut app, &service, register_msg).unwrap();

    let register_msg = &ExecuteMsg::RegisterServiceToOperator {
        service: service.to_string(),
    };

    let err = registry
        .execute(&mut app, &not_operator, register_msg)
        .unwrap_err();

    assert_eq!(
        err.root_cause().to_string(),
        ContractError::Std(StdError::not_found("operator")).to_string()
    );
}

#[test]
fn register_lifecycle_operator_first() {
    let (mut app, registry, ..) = instantiate();

    // Register as Service
    let register_as_service_msg = &ExecuteMsg::RegisterAsService {
        metadata: Metadata {
            name: Some("C4 Service".to_string()),
            uri: Some("https://c4.service.com".to_string()),
        },
    };
    let service = app.api().addr_make("service/bvs");
    registry
        .execute(&mut app, &service, register_as_service_msg)
        .unwrap();

    // Register as Operator
    let register_as_operator_msg = &ExecuteMsg::RegisterAsOperator {
        metadata: Metadata {
            name: Some("operator1".to_string()),
            uri: Some("https://operator.com".to_string()),
        },
    };
    let operator = app.api().addr_make("operator");
    registry
        .execute(&mut app, &operator, register_as_operator_msg)
        .unwrap();

    // Register Service to Operator
    let register_msg = &ExecuteMsg::RegisterServiceToOperator {
        service: service.to_string(),
    };
    let res = registry.execute(&mut app, &operator, register_msg).unwrap();
    assert_eq!(
        res.events,
        vec![
            Event::new("execute").add_attribute("_contract_address", registry.addr.as_str()),
            Event::new("wasm-RegistrationStatusUpdated")
                .add_attribute("_contract_address", registry.addr.as_str())
                .add_attribute("method", "register_service_to_operator")
                .add_attribute("operator", operator.as_str())
                .add_attribute("service", service.as_str())
                .add_attribute("status", "OperatorRegistered"),
        ]
    );

    // assert OperatorRegistered status
    let status: StatusResponse = registry
        .query(
            &mut app,
            &QueryMsg::Status {
                service: service.to_string(),
                operator: operator.to_string(),
                timestamp: None,
            },
        )
        .unwrap();
    assert_eq!(status, StatusResponse(2));

    // Register Operator to Service
    let register_msg = &ExecuteMsg::RegisterOperatorToService {
        operator: operator.to_string(),
    };

    let res = registry.execute(&mut app, &service, register_msg).unwrap();

    assert_eq!(
        res.events,
        vec![
            Event::new("execute").add_attribute("_contract_address", registry.addr.as_str()),
            Event::new("wasm-RegistrationStatusUpdated")
                .add_attribute("_contract_address", registry.addr.as_str())
                .add_attribute("method", "register_operator_to_service")
                .add_attribute("operator", operator.as_str())
                .add_attribute("service", service.as_str())
                .add_attribute("status", "Active"),
        ]
    );

    // assert Active status
    let status: StatusResponse = registry
        .query(
            &mut app,
            &QueryMsg::Status {
                service: service.to_string(),
                operator: operator.to_string(),
                timestamp: None,
            },
        )
        .unwrap();
    assert_eq!(status, StatusResponse(1));
}

#[test]
fn register_lifecycle_service_first() {
    let (mut app, registry, ..) = instantiate();

    // Register as Service
    let register_as_service_msg = &ExecuteMsg::RegisterAsService {
        metadata: Metadata {
            name: Some("C4 Service".to_string()),
            uri: Some("https://c4.service.com".to_string()),
        },
    };
    let service = app.api().addr_make("service/c4");
    registry
        .execute(&mut app, &service, register_as_service_msg)
        .unwrap();

    // Register as Operator
    let register_as_operator_msg = &ExecuteMsg::RegisterAsOperator {
        metadata: Metadata {
            name: Some("operator1".to_string()),
            uri: Some("https://operator.com".to_string()),
        },
    };
    let operator = app.api().addr_make("operator");
    registry
        .execute(&mut app, &operator, register_as_operator_msg)
        .unwrap();

    // Register Operator to Service
    let register_msg = &ExecuteMsg::RegisterOperatorToService {
        operator: operator.to_string(),
    };

    let res = registry.execute(&mut app, &service, register_msg).unwrap();

    assert_eq!(
        res.events,
        vec![
            Event::new("execute").add_attribute("_contract_address", registry.addr.as_str()),
            Event::new("wasm-RegistrationStatusUpdated")
                .add_attribute("_contract_address", registry.addr.as_str())
                .add_attribute("method", "register_operator_to_service")
                .add_attribute("operator", operator.as_str())
                .add_attribute("service", service.as_str())
                .add_attribute("status", "ServiceRegistered"),
        ]
    );

    // assert ServiceRegistered status
    let status: StatusResponse = registry
        .query(
            &mut app,
            &QueryMsg::Status {
                service: service.to_string(),
                operator: operator.to_string(),
                timestamp: None,
            },
        )
        .unwrap();
    assert_eq!(status, StatusResponse(3));

    // Register Service to Operator
    let register_msg = &ExecuteMsg::RegisterServiceToOperator {
        service: service.to_string(),
    };
    let res = registry.execute(&mut app, &operator, register_msg).unwrap();
    assert_eq!(
        res.events,
        vec![
            Event::new("execute").add_attribute("_contract_address", registry.addr.as_str()),
            Event::new("wasm-RegistrationStatusUpdated")
                .add_attribute("_contract_address", registry.addr.as_str())
                .add_attribute("method", "register_service_to_operator")
                .add_attribute("operator", operator.as_str())
                .add_attribute("service", service.as_str())
                .add_attribute("status", "Active"),
        ]
    );

    // assert Active status
    let status: StatusResponse = registry
        .query(
            &mut app,
            &QueryMsg::Status {
                service: service.to_string(),
                operator: operator.to_string(),
                timestamp: None,
            },
        )
        .unwrap();
    assert_eq!(status, StatusResponse(1));
}

#[test]
fn update_metadata_successfully() {
    let (mut app, registry, ..) = instantiate();

    let register_msg = &ExecuteMsg::RegisterAsService {
        metadata: Metadata {
            name: Some("Service Name".to_string()),
            uri: Some("https://service.com".to_string()),
        },
    };

    let service = app.api().addr_make("service/11111");
    registry.execute(&mut app, &service, register_msg).unwrap();

    let update_msg = &ExecuteMsg::UpdateServiceMetadata(Metadata {
        name: Some("New Service Name".to_string()),
        uri: Some("https://new-service.com".to_string()),
    });

    let response = registry.execute(&mut app, &service, update_msg).unwrap();

    assert_eq!(
        response.events,
        vec![
            Event::new("execute").add_attribute("_contract_address", registry.addr.as_str()),
            Event::new("wasm-MetadataUpdated")
                .add_attribute("_contract_address", registry.addr.as_str())
                .add_attribute("metadata.uri", "https://new-service.com")
                .add_attribute("metadata.name", "New Service Name")
                .add_attribute("service", service.as_str()),
        ]
    );

    // Don't update the name
    let update_msg = &ExecuteMsg::UpdateServiceMetadata(Metadata {
        name: None,
        uri: Some("https://new-new-service.com".to_string()),
    });

    let response = registry.execute(&mut app, &service, update_msg).unwrap();

    assert_eq!(
        response.events,
        vec![
            Event::new("execute").add_attribute("_contract_address", registry.addr.as_str()),
            Event::new("wasm-MetadataUpdated")
                .add_attribute("_contract_address", registry.addr.as_str())
                .add_attribute("metadata.uri", "https://new-new-service.com")
                .add_attribute("service", service.as_str())
        ]
    );
}

#[test]
fn transfer_ownership_successfully() {
    let (mut app, registry, _) = instantiate();
    let owner = app.api().addr_make("owner");
    let new_owner = app.api().addr_make("new_owner");

    let transfer_msg = &ExecuteMsg::TransferOwnership {
        new_owner: new_owner.to_string(),
    };

    let response = registry.execute(&mut app, &owner, transfer_msg).unwrap();

    assert_eq!(
        response.events,
        vec![
            Event::new("execute").add_attribute("_contract_address", registry.addr.as_str()),
            Event::new("wasm-TransferredOwnership")
                .add_attribute("_contract_address", registry.addr.as_str())
                .add_attribute("old_owner", owner.as_str())
                .add_attribute("new_owner", new_owner.as_str()),
        ]
    );
}

#[test]
fn transfer_ownership_but_not_owner() {
    let (mut app, registry, _) = instantiate();
    let not_owner = app.api().addr_make("not_owner");

    let transfer_msg = &ExecuteMsg::TransferOwnership {
        new_owner: not_owner.to_string(),
    };

    let err = registry
        .execute(&mut app, &not_owner, transfer_msg)
        .unwrap_err();

    assert_eq!(
        err.root_cause().to_string(),
        ContractError::Unauthorized {}.to_string()
    );
}

#[test]
fn register_deregister_lifecycle() {
    let (mut app, registry, ..) = instantiate();

    let service = app.api().addr_make("service/1");
    let service2 = app.api().addr_make("service/2");
    let operator = app.api().addr_make("operator/1");
    let operator2 = app.api().addr_make("operator/2");

    // register service + service2 + operator + operator2
    {
        registry
            .execute(
                &mut app,
                &service,
                &ExecuteMsg::RegisterAsService {
                    metadata: Metadata {
                        name: Some(service.to_string()),
                        uri: Some("https://service.com".to_string()),
                    },
                },
            )
            .unwrap();
        registry
            .execute(
                &mut app,
                &service2,
                &ExecuteMsg::RegisterAsService {
                    metadata: Metadata {
                        name: Some(service2.to_string()),
                        uri: Some("https://service2.com".to_string()),
                    },
                },
            )
            .unwrap();

        registry
            .execute(
                &mut app,
                &operator,
                &ExecuteMsg::RegisterAsOperator {
                    metadata: Metadata {
                        name: Some(operator.to_string()),
                        uri: Some("https://operator.com".to_string()),
                    },
                },
            )
            .unwrap();
        registry
            .execute(
                &mut app,
                &operator2,
                &ExecuteMsg::RegisterAsOperator {
                    metadata: Metadata {
                        name: Some(operator2.to_string()),
                        uri: Some("https://operator.com".to_string()),
                    },
                },
            )
            .unwrap();
    }

    // register service and service2 to operator and operator2
    {
        for curr_service in [service.clone(), service2.clone()].iter() {
            registry
                .execute(
                    &mut app,
                    &operator,
                    &ExecuteMsg::RegisterServiceToOperator {
                        service: curr_service.to_string(),
                    },
                )
                .unwrap();
            registry
                .execute(
                    &mut app,
                    &operator2,
                    &ExecuteMsg::RegisterServiceToOperator {
                        service: curr_service.to_string(),
                    },
                )
                .unwrap();

            registry
                .execute(
                    &mut app,
                    curr_service,
                    &ExecuteMsg::RegisterOperatorToService {
                        operator: operator.to_string(),
                    },
                )
                .unwrap();
            registry
                .execute(
                    &mut app,
                    curr_service,
                    &ExecuteMsg::RegisterOperatorToService {
                        operator: operator2.to_string(),
                    },
                )
                .unwrap();
        }
    }

    // check if all services are registered to operator and operator2
    {
        for curr_service in [service.clone(), service2.clone()].iter() {
            let status: StatusResponse = registry
                .query(
                    &app,
                    &QueryMsg::Status {
                        service: curr_service.to_string(),
                        operator: operator.to_string(),
                        timestamp: None,
                    },
                )
                .unwrap();
            assert_eq!(status, StatusResponse(1));

            let status: StatusResponse = registry
                .query(
                    &app,
                    &QueryMsg::Status {
                        service: curr_service.to_string(),
                        operator: operator2.to_string(),
                        timestamp: None,
                    },
                )
                .unwrap();
            assert_eq!(status, StatusResponse(1));
        }
    }

    // move the chain
    app.update_block(|block| {
        block.height += 10;
        block.time = block.time.plus_seconds(10);
    });

    // check if all services are registered to operator and operator2 at current timestamp - 5
    {
        for curr_service in [service.clone(), service2.clone()].iter() {
            let status: StatusResponse = registry
                .query(
                    &app,
                    &QueryMsg::Status {
                        service: curr_service.to_string(),
                        operator: operator.to_string(),
                        timestamp: Some(app.block_info().time.minus_seconds(5).seconds()),
                    },
                )
                .unwrap();
            assert_eq!(status, StatusResponse(1));

            let status: StatusResponse = registry
                .query(
                    &app,
                    &QueryMsg::Status {
                        service: curr_service.to_string(),
                        operator: operator2.to_string(),
                        timestamp: Some(app.block_info().time.minus_seconds(5).seconds()),
                    },
                )
                .unwrap();
            assert_eq!(status, StatusResponse(1));
        }
    }

    // deregister operator <-> service
    registry
        .execute(
            &mut app,
            &operator,
            &ExecuteMsg::DeregisterServiceFromOperator {
                service: service.to_string(),
            },
        )
        .unwrap();

    // check current status of operator <-> service and operator <-> service2
    {
        let status: StatusResponse = registry
            .query(
                &app,
                &QueryMsg::Status {
                    service: service.to_string(),
                    operator: operator.to_string(),
                    timestamp: None,
                },
            )
            .unwrap();
        assert_eq!(status, StatusResponse(0)); // inactive

        let status: StatusResponse = registry
            .query(
                &app,
                &QueryMsg::Status {
                    service: service2.to_string(),
                    operator: operator.to_string(),
                    timestamp: None,
                },
            )
            .unwrap();
        assert_eq!(status, StatusResponse(1));
    }

    // move the chain
    app.update_block(|block| {
        block.height += 10;
        block.time = block.time.plus_seconds(10);
    });

    // check if service is deregistered from operator at current timestamp - 5
    let status: StatusResponse = registry
        .query(
            &app,
            &QueryMsg::Status {
                service: service.to_string(),
                operator: operator.to_string(),
                timestamp: Some(app.block_info().time.minus_seconds(5).seconds()),
            },
        )
        .unwrap();
    assert_eq!(status, StatusResponse(0)); // inactive

    // check if service2 is still registered to operator at current timestamp - 5
    let status: StatusResponse = registry
        .query(
            &app,
            &QueryMsg::Status {
                service: service2.to_string(),
                operator: operator.to_string(),
                timestamp: Some(app.block_info().time.minus_seconds(5).seconds()),
            },
        )
        .unwrap();
    assert_eq!(status, StatusResponse(1)); // active
}

#[test]
fn enable_slashing_lifecycle() {
    let (mut app, registry, ..) = instantiate();

    let service = app.api().addr_make("service/1");
    let operator = app.api().addr_make("operator/1");
    let operator2 = app.api().addr_make("operator/2");
    let burn_address = app.api().addr_make("burn_address");

    // register service + operators
    {
        registry
            .execute(
                &mut app,
                &service,
                &ExecuteMsg::RegisterAsService {
                    metadata: Metadata {
                        name: Some(service.to_string()),
                        uri: Some("https://service.com".to_string()),
                    },
                },
            )
            .unwrap();
        registry
            .execute(
                &mut app,
                &operator,
                &ExecuteMsg::RegisterAsOperator {
                    metadata: Metadata {
                        name: Some(operator.to_string()),
                        uri: Some("https://operator.com".to_string()),
                    },
                },
            )
            .unwrap();
        registry
            .execute(
                &mut app,
                &operator2,
                &ExecuteMsg::RegisterAsOperator {
                    metadata: Metadata {
                        name: Some(operator2.to_string()),
                        uri: Some("https://operator2.com".to_string()),
                    },
                },
            )
            .unwrap();
    }

    // register service to operator and operator to service
    {
        registry
            .execute(
                &mut app,
                &operator,
                &ExecuteMsg::RegisterServiceToOperator {
                    service: service.to_string(),
                },
            )
            .unwrap();
        registry
            .execute(
                &mut app,
                &service,
                &ExecuteMsg::RegisterOperatorToService {
                    operator: operator.to_string(),
                },
            )
            .unwrap();
    }

    // NEGATIVE - operator opts in to slashing, before it is enabled
    {
        let opt_in_msg = &ExecuteMsg::OperatorOptInToSlashing {
            service: service.to_string(),
        };
        let err = registry
            .execute(&mut app, &operator, opt_in_msg)
            .unwrap_err();

        assert_eq!(
            err.root_cause().to_string(),
            ContractError::InvalidSlashingOptIn {
                msg: "Cannot opt in: slashing is not enabled for this service".to_string()
            }
            .to_string()
        );
    }

    // service enable slashing
    {
        let slashing_parameters = SlashingParameters {
            destination: Some(burn_address.clone()),
            max_slashing_bips: 5000, // 50%
            resolution_window: 1000,
        };
        let enable_slashing_msg = &ExecuteMsg::EnableSlashing {
            slashing_parameters: slashing_parameters.clone(),
        };
        let res = registry
            .execute(&mut app, &service, enable_slashing_msg)
            .unwrap();

        // assert events
        assert_eq!(
            res.events,
            vec![
                Event::new("execute").add_attribute("_contract_address", registry.addr.as_str()),
                Event::new("wasm-SlashingParametersEnabled")
                    .add_attribute("_contract_address", registry.addr.as_str())
                    .add_attribute("service", service.as_str())
                    .add_attribute("destination", burn_address.to_string())
                    .add_attribute("max_slashing_bips", "5000")
                    .add_attribute("resolution_window", "1000"),
            ]
        );

        // assert query
        let slashing_parameters_res: SlashingParameters = registry
            .query(
                &mut app,
                &QueryMsg::SlashingParameters {
                    service: service.to_string(),
                    timestamp: None,
                },
            )
            .unwrap();
        assert_eq!(slashing_parameters_res, slashing_parameters);
    }

    // NEGATIVE - operator opts-in to slashing in the same block as slash enabled
    {
        let opt_in_msg = &ExecuteMsg::OperatorOptInToSlashing {
            service: service.to_string(),
        };
        let err = registry
            .execute(&mut app, &operator, opt_in_msg)
            .unwrap_err();

        // assert events
        assert_eq!(
            err.root_cause().to_string(),
            ContractError::InvalidSlashingOptIn {
                msg: "Cannot opt in: slashing is not enabled for this service".to_string()
            }
            .to_string()
        );
    }

    // move blockchain
    app.update_block(|block| {
        block.height += 1;
        block.time = block.time.plus_seconds(10);
    });

    // operator opts-in to slashing
    {
        let opt_in_msg = &ExecuteMsg::OperatorOptInToSlashing {
            service: service.to_string(),
        };
        let res = registry.execute(&mut app, &operator, opt_in_msg).unwrap();

        // assert events
        assert_eq!(
            res.events,
            vec![
                Event::new("execute").add_attribute("_contract_address", registry.addr.as_str()),
                Event::new("wasm-OperatorOptedInToSlashing")
                    .add_attribute("_contract_address", registry.addr.as_str())
                    .add_attribute("operator", operator.as_str())
                    .add_attribute("service", service.as_str())
            ]
        );
        // assert query
        let is_operator_opted_in: IsOperatorOptedInToSlashingResponse = registry
            .query(
                &mut app,
                &QueryMsg::IsOperatorOptedInToSlashing {
                    service: service.to_string(),
                    operator: operator.to_string(),
                    timestamp: None,
                },
            )
            .unwrap();
        assert_eq!(
            is_operator_opted_in,
            IsOperatorOptedInToSlashingResponse(true)
        );
    }

    // NEGATIVE - operator2 opts in to service's slashing before Active registration
    {
        let opt_in_msg = &ExecuteMsg::OperatorOptInToSlashing {
            service: service.to_string(),
        };
        let err = registry
            .execute(&mut app, &operator2, opt_in_msg)
            .unwrap_err();

        // assert events
        assert_eq!(
            err.root_cause().to_string(),
            ContractError::InvalidRegistrationStatus {
                msg: "Operator and service must have active registration".to_string()
            }
            .to_string()
        );
    }

    // operator2 register service to operator => operator2 will be auto opt-in to slashing
    {
        registry
            .execute(
                &mut app,
                &operator2,
                &ExecuteMsg::RegisterServiceToOperator {
                    service: service.to_string(),
                },
            )
            .unwrap();
        registry
            .execute(
                &mut app,
                &service,
                &ExecuteMsg::RegisterOperatorToService {
                    operator: operator2.to_string(),
                },
            )
            .unwrap();

        // assert query
        let is_operator_opted_in: IsOperatorOptedInToSlashingResponse = registry
            .query(
                &mut app,
                &QueryMsg::IsOperatorOptedInToSlashing {
                    service: service.to_string(),
                    operator: operator2.to_string(),
                    timestamp: None,
                },
            )
            .unwrap();
        assert_eq!(
            is_operator_opted_in,
            IsOperatorOptedInToSlashingResponse(true)
        );
    }

    // move blockchain
    app.update_block(|block| {
        block.height += 1;
        block.time = block.time.plus_seconds(10);
    });

    // service updates slashing parameters
    {
        let slashing_parameters = SlashingParameters {
            destination: Some(burn_address.clone()),
            max_slashing_bips: 9000, // 90%
            resolution_window: 1000,
        };
        let enable_slashing_msg = &ExecuteMsg::EnableSlashing {
            slashing_parameters: slashing_parameters.clone(),
        };
        registry
            .execute(&mut app, &service, enable_slashing_msg)
            .unwrap();

        // assert query
        let slashing_parameters_res: SlashingParameters = registry
            .query(
                &mut app,
                &QueryMsg::SlashingParameters {
                    service: service.to_string(),
                    timestamp: None,
                },
            )
            .unwrap();
        assert_eq!(slashing_parameters_res, slashing_parameters);

        // assert previous slashing param is available if prev block timestamp
        let prev_timestamp = app.block_info().time.minus_seconds(1).seconds();
        let prev_slashing_parameters_res: SlashingParameters = registry
            .query(
                &mut app,
                &QueryMsg::SlashingParameters {
                    service: service.to_string(),
                    timestamp: Some(prev_timestamp),
                },
            )
            .unwrap();
        assert_eq!(
            prev_slashing_parameters_res,
            SlashingParameters {
                destination: Some(burn_address.clone()),
                max_slashing_bips: 5000, // 50%
                resolution_window: 1000,
            }
        );
    }

    // assert that operator and operator2 is not opted into new slashing parameters
    {
        let is_operator_opted_in: IsOperatorOptedInToSlashingResponse = registry
            .query(
                &mut app,
                &QueryMsg::IsOperatorOptedInToSlashing {
                    service: service.to_string(),
                    operator: operator.to_string(),
                    timestamp: None,
                },
            )
            .unwrap();
        assert_eq!(
            is_operator_opted_in,
            IsOperatorOptedInToSlashingResponse(false)
        );
        let is_operator2_opted_in: IsOperatorOptedInToSlashingResponse = registry
            .query(
                &mut app,
                &QueryMsg::IsOperatorOptedInToSlashing {
                    service: service.to_string(),
                    operator: operator2.to_string(),
                    timestamp: None,
                },
            )
            .unwrap();
        assert_eq!(
            is_operator2_opted_in,
            IsOperatorOptedInToSlashingResponse(false)
        );
    }

    // move blockchain
    app.update_block(|block| {
        block.height += 1;
        block.time = block.time.plus_seconds(10);
    });

    // operator2 opts into new slashing param
    {
        let opt_in_msg = &ExecuteMsg::OperatorOptInToSlashing {
            service: service.to_string(),
        };
        let res = registry.execute(&mut app, &operator2, opt_in_msg).unwrap();

        // assert events
        assert_eq!(
            res.events,
            vec![
                Event::new("execute").add_attribute("_contract_address", registry.addr.as_str()),
                Event::new("wasm-OperatorOptedInToSlashing")
                    .add_attribute("_contract_address", registry.addr.as_str())
                    .add_attribute("operator", operator2.as_str())
                    .add_attribute("service", service.as_str())
            ]
        );
        // assert query
        let is_operator2_opted_in: IsOperatorOptedInToSlashingResponse = registry
            .query(
                &mut app,
                &QueryMsg::IsOperatorOptedInToSlashing {
                    service: service.to_string(),
                    operator: operator2.to_string(),
                    timestamp: None,
                },
            )
            .unwrap();
        assert_eq!(
            is_operator2_opted_in,
            IsOperatorOptedInToSlashingResponse(true)
        );
    }

    // move blockchain
    app.update_block(|block| {
        block.height += 1;
        block.time = block.time.plus_seconds(10);
    });

    // operator2 deregister from service
    {
        let deregister_msg = &ExecuteMsg::DeregisterServiceFromOperator {
            service: service.to_string(),
        };
        registry
            .execute(&mut app, &operator2, deregister_msg)
            .unwrap();

        // assert query
        let operator2_status_res: StatusResponse = registry
            .query(
                &mut app,
                &QueryMsg::Status {
                    service: service.to_string(),
                    operator: operator2.to_string(),
                    timestamp: None,
                },
            )
            .unwrap();
        assert_eq!(
            operator2_status_res,
            StatusResponse(0) // Inactive
        );
    }

    // move blockchain
    app.update_block(|block| {
        block.height += 1;
        block.time = block.time.plus_seconds(10);
    });

    // operator2 registers to service
    {
        let register_msg = &ExecuteMsg::RegisterOperatorToService {
            operator: operator2.to_string(),
        };
        registry.execute(&mut app, &service, register_msg).unwrap();

        // assert operator2 is opted-in to slashing
        let is_operator2_opted_in: IsOperatorOptedInToSlashingResponse = registry
            .query(
                &mut app,
                &QueryMsg::IsOperatorOptedInToSlashing {
                    service: service.to_string(),
                    operator: operator2.to_string(),
                    timestamp: None,
                },
            )
            .unwrap();
        assert_eq!(
            is_operator2_opted_in,
            IsOperatorOptedInToSlashingResponse(true)
        );
    }
}

#[test]
fn query_status() {
    let (mut app, registry, _) = instantiate();

    let query_msg = &QueryMsg::Status {
        service: app.api().addr_make("service/44").to_string(),
        operator: app.api().addr_make("operator/44").to_string(),
        timestamp: None,
    };

    let status: StatusResponse = registry.query(&mut app, query_msg).unwrap();

    assert_eq!(status, StatusResponse(0));

    let status: StatusResponse = registry.query(&mut app, query_msg).unwrap();

    assert_eq!(status, StatusResponse(0));
}

#[test]
fn migrate_to_v2() {
    let (mut app, registry, ..) = instantiate();

    let service = app.api().addr_make("service/1");
    let operator = app.api().addr_make("operator/1");

    // populate initial contract state with data
    let old_registration_status: Map<(&Addr, &Addr), u8> = Map::new("registration_status");

    let operator_active_registration_count: Map<&Addr, u8> =
        Map::new("operator_active_registration_count");

    let service_active_operators_count: Map<&Addr, u8> = Map::new("service_active_operators_count");

    {
        // save some data into old contract state with same 'registration_status' namespace
        let mut contract_storage = app.contract_storage_mut(&registry.addr);
        old_registration_status
            .save(
                &mut *contract_storage,
                (&operator, &service),
                &(RegistrationStatus::Active as u8),
            )
            .unwrap();

        operator_active_registration_count
            .save(&mut *contract_storage, &operator, &1u8)
            .unwrap();

        service_active_operators_count
            .save(&mut *contract_storage, &service, &1u8)
            .unwrap();

        // assert that state is populated
        let res = old_registration_status
            .load(&*contract_storage, (&operator, &service))
            .unwrap();

        assert_eq!(res, RegistrationStatus::Active as u8);
    }

    let migrate_msg = &bvs_registry::msg::MigrateMsg {};
    let admin = app.api().addr_make("admin");

    registry.migrate(&mut app, &admin, migrate_msg).unwrap();

    // check if state is migrated
    let status: StatusResponse = registry
        .query(
            &mut app,
            &QueryMsg::Status {
                service: service.to_string(),
                operator: operator.to_string(),
                timestamp: None,
            },
        )
        .unwrap();
    assert_eq!(status, StatusResponse(1));

    let block_info = app.block_info();
    let status_at_timestamp: StatusResponse = registry
        .query(
            &mut app,
            &QueryMsg::Status {
                service: service.to_string(),
                operator: operator.to_string(),
                timestamp: Some(block_info.time.minus_seconds(1).seconds()),
            },
        )
        .unwrap();
    assert_eq!(status_at_timestamp, StatusResponse(1));

    // test other interaction with state after migrate
    {
        // deregister
        let deregister_msg = &ExecuteMsg::DeregisterServiceFromOperator {
            service: service.to_string(),
        };
        registry
            .execute(&mut app, &operator, deregister_msg)
            .unwrap();

        // check if state is changed
        let status: StatusResponse = registry
            .query(
                &mut app,
                &QueryMsg::Status {
                    service: service.to_string(),
                    operator: operator.to_string(),
                    timestamp: None,
                },
            )
            .unwrap();
        assert_eq!(status, StatusResponse(0));

        // check if state is changed at timestamp + 1
        let block_info = app.block_info();
        let status_at_timestamp: StatusResponse = registry
            .query(
                &mut app,
                &QueryMsg::Status {
                    service: service.to_string(),
                    operator: operator.to_string(),
                    timestamp: Some(block_info.time.plus_seconds(1).seconds()),
                },
            )
            .unwrap();
        assert_eq!(status_at_timestamp, StatusResponse(0));

        // check old state at timestamp - 10 -> should be active
        let block_info = app.block_info();
        let status_at_timestamp: StatusResponse = registry
            .query(
                &mut app,
                &QueryMsg::Status {
                    service: service.to_string(),
                    operator: operator.to_string(),
                    timestamp: Some(block_info.time.minus_seconds(10).seconds()),
                },
            )
            .unwrap();
        assert_eq!(status_at_timestamp, StatusResponse(1));
    }
}

#[test]
fn test_registration_counters() {
    let (mut app, registry, _) = instantiate();

    let services = vec![
        app.api().addr_make("service/1"),
        app.api().addr_make("service/2"),
    ];

    // 20 operators
    let operators = vec![
        app.api().addr_make("operator/1"),
        app.api().addr_make("operator/2"),
        app.api().addr_make("operator/3"),
        app.api().addr_make("operator/4"),
        app.api().addr_make("operator/5"),
        app.api().addr_make("operator/6"),
        app.api().addr_make("operator/7"),
        app.api().addr_make("operator/8"),
        app.api().addr_make("operator/9"),
        app.api().addr_make("operator/10"),
        app.api().addr_make("operator/11"),
        app.api().addr_make("operator/12"),
        app.api().addr_make("operator/13"),
        app.api().addr_make("operator/14"),
        app.api().addr_make("operator/15"),
        app.api().addr_make("operator/16"),
        app.api().addr_make("operator/17"),
        app.api().addr_make("operator/18"),
        app.api().addr_make("operator/19"),
        app.api().addr_make("operator/20"),
    ];

    for service in services.iter() {
        // register service
        registry
            .execute(
                &mut app,
                service,
                &ExecuteMsg::RegisterAsService {
                    metadata: Metadata {
                        name: Some(service.to_string()),
                        uri: Some(format!("https://service-{}.com", service).to_string()),
                    },
                },
            )
            .unwrap();
    }

    for operator in operators.iter() {
        // register operator
        registry
            .execute(
                &mut app,
                operator,
                &ExecuteMsg::RegisterAsOperator {
                    metadata: Metadata {
                        name: Some(operator.to_string()),
                        uri: Some(format!("https://operator-{}.com", operator).to_string()),
                    },
                },
            )
            .unwrap();
    }

    // query counters - should all be 0
    for operator in operators.iter() {
        let count: u64 = registry
            .query(
                &mut app,
                &QueryMsg::ActiveServicesCount {
                    operator: operator.to_string(),
                },
            )
            .unwrap();
        assert_eq!(count, 0);
    }

    for service in services.iter() {
        let count: u64 = registry
            .query(
                &mut app,
                &QueryMsg::ActiveOperatorsCount {
                    service: service.to_string(),
                },
            )
            .unwrap();
        assert_eq!(count, 0);
    }

    for operator in operators.iter() {
        for service in services.iter() {
            // register operator <-> service
            registry
                .execute(
                    &mut app,
                    operator,
                    &ExecuteMsg::RegisterServiceToOperator {
                        service: service.to_string(),
                    },
                )
                .unwrap();
            registry
                .execute(
                    &mut app,
                    service,
                    &ExecuteMsg::RegisterOperatorToService {
                        operator: operator.to_string(),
                    },
                )
                .unwrap();
        }
    }

    {
        let count: u64 = registry
            .query(
                &mut app,
                &QueryMsg::ActiveOperatorsCount {
                    service: services[0].to_string(),
                },
            )
            .unwrap();
        assert_eq!(count, 20);

        let count: u64 = registry
            .query(
                &mut app,
                &QueryMsg::ActiveOperatorsCount {
                    service: services[1].to_string(),
                },
            )
            .unwrap();

        assert_eq!(count, 20);
    }

    for operator in operators.iter() {
        let count: u64 = registry
            .query(
                &mut app,
                &QueryMsg::ActiveServicesCount {
                    operator: operator.to_string(),
                },
            )
            .unwrap();
        assert_eq!(count, 2);
    }

    registry
        .execute(
            &mut app,
            &operators[0],
            &ExecuteMsg::DeregisterServiceFromOperator {
                service: services[0].to_string(),
            },
        )
        .unwrap();

    {
        let count: u64 = registry
            .query(
                &mut app,
                &QueryMsg::ActiveOperatorsCount {
                    service: services[0].to_string(),
                },
            )
            .unwrap();
        assert_eq!(count, 19);

        let count: u64 = registry
            .query(
                &mut app,
                &QueryMsg::ActiveOperatorsCount {
                    service: services[1].to_string(),
                },
            )
            .unwrap();

        assert_eq!(count, 20);
    }

    {
        let count: u64 = registry
            .query(
                &mut app,
                &QueryMsg::ActiveServicesCount {
                    operator: operators[0].to_string(),
                },
            )
            .unwrap();
        assert_eq!(count, 1);

        for operator in operators.iter().skip(1) {
            let count: u64 = registry
                .query(
                    &mut app,
                    &QueryMsg::ActiveServicesCount {
                        operator: operator.to_string(),
                    },
                )
                .unwrap();
            assert_eq!(count, 2);
        }
    }

    // deregister the rest of operators from service[0]
    {
        for operator in operators.iter().skip(1) {
            registry
                .execute(
                    &mut app,
                    operator,
                    &ExecuteMsg::DeregisterServiceFromOperator {
                        service: services[0].to_string(),
                    },
                )
                .unwrap();
            let count: u64 = registry
                .query(
                    &mut app,
                    &QueryMsg::ActiveOperatorsCount {
                        service: services[0].to_string(),
                    },
                )
                .unwrap();
            assert_eq!(
                count,
                operators.len() as u64
                    - (operators.iter().position(|o| o == operator).unwrap() as u64)
                    - 1
            );
        }
    }

    // service 0 should not have any active operators
    {
        let count: u64 = registry
            .query(
                &mut app,
                &QueryMsg::ActiveOperatorsCount {
                    service: services[0].to_string(),
                },
            )
            .unwrap();
        assert_eq!(count, 0);
    }

    // service 1 should still have all operators
    {
        let count: u64 = registry
            .query(
                &mut app,
                &QueryMsg::ActiveOperatorsCount {
                    service: services[1].to_string(),
                },
            )
            .unwrap();
        assert_eq!(count, 20);
    }
}

/// Since CONTRACT_VERSION is loaded from env!("CARGO_PKG_VERSION"), "0.0.0"
/// cw2::ensure_from_older_version will fail
/// As a workaround, we can hardcode the `CONTRACT_VERSION` to "2.4.0" in `contract.rs` during local migration test
#[test]
fn test_migrate_service_active_operators_count() {
    // let (mut app, registry, _) = instantiate();
    //
    // let services = vec![
    //     app.api().addr_make("service/1"),
    //     app.api().addr_make("service/2"),
    // ];
    //
    // let operators = vec![
    //     app.api().addr_make("operator/1"),
    //     app.api().addr_make("operator/2"),
    //     app.api().addr_make("operator/3"),
    //     app.api().addr_make("operator/4"),
    //     app.api().addr_make("operator/5"),
    //     app.api().addr_make("operator/6"),
    //     app.api().addr_make("operator/7"),
    //     app.api().addr_make("operator/8"),
    //     app.api().addr_make("operator/9"),
    //     app.api().addr_make("operator/10"),
    // ];
    //
    // for service in services.iter() {
    //     // register service
    //     registry
    //         .execute(
    //             &mut app,
    //             service,
    //             &ExecuteMsg::RegisterAsService {
    //                 metadata: Metadata {
    //                     name: Some(service.to_string()),
    //                     uri: Some(format!("https://service-{}.com", service).to_string()),
    //                 },
    //             },
    //         )
    //         .unwrap();
    // }
    //
    // for operator in operators.iter() {
    //     // register operator
    //     registry
    //         .execute(
    //             &mut app,
    //             operator,
    //             &ExecuteMsg::RegisterAsOperator {
    //                 metadata: Metadata {
    //                     name: Some(operator.to_string()),
    //                     uri: Some(format!("https://operator-{}.com", operator).to_string()),
    //                 },
    //             },
    //         )
    //         .unwrap();
    // }
    //
    // // register some operators to services
    // for operator in operators.iter() {
    //     for service in services.iter() {
    //         // register operator <-> service
    //         registry
    //             .execute(
    //                 &mut app,
    //                 operator,
    //                 &ExecuteMsg::RegisterServiceToOperator {
    //                     service: service.to_string(),
    //                 },
    //             )
    //             .unwrap();
    //         registry
    //             .execute(
    //                 &mut app,
    //                 service,
    //                 &ExecuteMsg::RegisterOperatorToService {
    //                     operator: operator.to_string(),
    //                 },
    //             )
    //             .unwrap();
    //     }
    // }
    //
    // // contract code is up to date
    // // we'll have to manipulate the state to mimic pre-migration state
    // {
    //     let mut contract_storage = app.contract_storage_mut(&registry.addr);
    //
    //     let old_version = cw2::ContractVersion {
    //         contract: concat!("crates.io:", env!("CARGO_PKG_NAME")).to_string(),
    //         version: "2.3.0".to_string(),
    //     };
    //
    //     let old_contract_info_state: cw_storage_plus::Item<cw2::ContractVersion> = cw_storage_plus::Item::new("contract_info");
    //
    //     old_contract_info_state
    //         .save(&mut *contract_storage, &old_version)
    //         .unwrap();
    //
    //     // manually reset service_active_operators_count to 0
    //     let old_service_active_operators_count: Map<&Addr, u8> = Map::new("service_active_operators_count");
    //
    //     for service in services.iter() {
    //         old_service_active_operators_count
    //             .save(&mut *contract_storage, service, &0u8)
    //             .unwrap();
    //     }
    // }
    //
    // let pre_counts = services.iter().map(|service| {
    //     let count: u64 = registry
    //         .query(
    //             &mut app,
    //             &QueryMsg::ActiveOperatorsCount {
    //                 service: service.to_string(),
    //             },
    //         )
    //         .unwrap();
    //     count
    // }).collect::<Vec<u64>>();
    // assert_eq!(pre_counts, vec![0, 0]);
    //
    // let migrate_msg = &bvs_registry::msg::MigrateMsg {};
    // let admin = app.api().addr_make("admin");
    //
    // registry.migrate(&mut app, &admin, migrate_msg).unwrap();
    //
    // let count1: u64 = registry
    //     .query(
    //         &mut app,
    //         &QueryMsg::ActiveOperatorsCount {
    //             service: services[0].to_string(),
    //         },
    //     )
    //     .unwrap();
    //
    // assert_eq!(count1, 10);
    //
    // let count2: u64 = registry
    //     .query(
    //         &mut app,
    //         &QueryMsg::ActiveOperatorsCount {
    //             service: services[1].to_string(),
    //         },
    //     )
    //     .unwrap();
    // assert_eq!(count2, 10);
}
