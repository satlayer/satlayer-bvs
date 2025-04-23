use bvs_library::testing::TestingContract;
use bvs_pauser::api::PauserError;
use bvs_pauser::testing::PauserContract;
use bvs_registry::msg::{ExecuteMsg, Metadata, QueryMsg, StatusResponse};
use bvs_registry::testing::RegistryContract;
use bvs_registry::{ContractError, RegistrationStatus};
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
        .execute(&mut app, &owner, &bvs_pauser::msg::ExecuteMsg::Pause {})
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
            },
        )
        .unwrap();
    assert_eq!(status, StatusResponse(1));
}

// TODO: deregister from service
// TODO: deregister from operator
// TODO: already deregistered
// TODO: already active
// TODO: operator already registered
// TODO: service already registered

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
                    },
                )
                .unwrap();
            assert_eq!(status, StatusResponse(1));
        }
    }

    // move the chain
    app.update_block(|block| {
        block.height += 10;
    });

    // check if all services are registered to operator and operator2 at current height - 5
    {
        for curr_service in [service.clone(), service2.clone()].iter() {
            let status: StatusResponse = registry
                .query(
                    &app,
                    &QueryMsg::StatusAtHeight {
                        service: curr_service.to_string(),
                        operator: operator.to_string(),
                        height: app.block_info().height - 5,
                    },
                )
                .unwrap();
            assert_eq!(status, StatusResponse(1));

            let status: StatusResponse = registry
                .query(
                    &app,
                    &QueryMsg::StatusAtHeight {
                        service: curr_service.to_string(),
                        operator: operator2.to_string(),
                        height: app.block_info().height - 5,
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
                },
            )
            .unwrap();
        assert_eq!(status, StatusResponse(1));
    }

    // move the chain
    app.update_block(|block| {
        block.height += 10;
    });

    // check if service is deregistered from operator at current height - 5
    let status: StatusResponse = registry
        .query(
            &app,
            &QueryMsg::StatusAtHeight {
                service: service.to_string(),
                operator: operator.to_string(),
                height: app.block_info().height - 5,
            },
        )
        .unwrap();
    assert_eq!(status, StatusResponse(0)); // inactive

    // check if service2 is still registered to operator at current height - 5
    let status: StatusResponse = registry
        .query(
            &app,
            &QueryMsg::StatusAtHeight {
                service: service2.to_string(),
                operator: operator.to_string(),
                height: app.block_info().height - 5,
            },
        )
        .unwrap();
    assert_eq!(status, StatusResponse(1)); // active
}

#[test]
fn query_status() {
    let (mut app, registry, _) = instantiate();

    let query_msg = &QueryMsg::Status {
        service: app.api().addr_make("service/44").to_string(),
        operator: app.api().addr_make("operator/44").to_string(),
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
            },
        )
        .unwrap();
    assert_eq!(status, StatusResponse(1));

    let block_info = app.block_info();
    let status_at_height: StatusResponse = registry
        .query(
            &mut app,
            &QueryMsg::StatusAtHeight {
                service: service.to_string(),
                operator: operator.to_string(),
                height: block_info.height - 1,
            },
        )
        .unwrap();
    assert_eq!(status_at_height, StatusResponse(1));

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
                },
            )
            .unwrap();
        assert_eq!(status, StatusResponse(0));

        // check if state is changed at height + 1
        let block_info = app.block_info();
        let status_at_height: StatusResponse = registry
            .query(
                &mut app,
                &QueryMsg::StatusAtHeight {
                    service: service.to_string(),
                    operator: operator.to_string(),
                    height: block_info.height + 1,
                },
            )
            .unwrap();
        assert_eq!(status_at_height, StatusResponse(0));

        // check old state at height - 10 -> should be active
        let block_info = app.block_info();
        let status_at_height: StatusResponse = registry
            .query(
                &mut app,
                &QueryMsg::StatusAtHeight {
                    service: service.to_string(),
                    operator: operator.to_string(),
                    height: block_info.height - 10,
                },
            )
            .unwrap();
        assert_eq!(status_at_height, StatusResponse(1));
    }
}
