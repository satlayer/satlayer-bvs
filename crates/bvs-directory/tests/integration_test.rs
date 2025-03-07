use bvs_delegation_manager::{
    msg::{ExecuteMsg as DelagtionExecuteMsg, OperatorDetails},
    testing::DelegationManagerContract,
};
use bvs_directory::{
    msg::{ExecuteMsg, QueryMsg, ServiceMetadata, StatusResponse},
    testing::DirectoryContract,
    ContractError,
};
use bvs_library::testing::TestingContract;
use bvs_registry::{api::RegistryError, testing::RegistryContract};
use bvs_strategy_manager::testing::StrategyManagerContract;

use cosmwasm_std::{testing::mock_env, Event};
use cw_multi_test::App;

fn instantiate() -> (
    App,
    DirectoryContract,
    DelegationManagerContract,
    RegistryContract,
) {
    let mut app = App::default();
    let env = mock_env();

    let registry = RegistryContract::new(&mut app, &env, None);
    let delegation_manager = DelegationManagerContract::new(&mut app, &env, None);
    let directory = DirectoryContract::new(&mut app, &env, None);
    let strategy_manager = StrategyManagerContract::new(&mut app, &env, None);

    let owner = app.api().addr_make("owner");
    let not_routed = app.api().addr_make("not_routed");

    directory
        .execute(
            &mut app,
            &owner,
            &ExecuteMsg::SetRouting {
                delegation_manager: delegation_manager.addr.to_string(),
            },
        )
        .unwrap();
    let msg = ExecuteMsg::SetRouting {
        delegation_manager: delegation_manager.addr().to_string(),
    };
    directory.execute(&mut app, &owner, &msg).unwrap();

    delegation_manager
        .execute(
            &mut app,
            &owner,
            &DelagtionExecuteMsg::SetRouting {
                strategy_manager: strategy_manager.addr().to_string(),
                slash_manager: not_routed.to_string(),
            },
        )
        .unwrap();

    (app, directory, delegation_manager, registry)
}

#[test]
fn register_service_successfully() {
    let (mut app, directory, ..) = instantiate();

    let register_msg = &ExecuteMsg::ServiceRegister {
        metadata: ServiceMetadata {
            name: Some("Service Name".to_string()),
            uri: Some("https://service.com".to_string()),
        },
    };

    let service = app.api().addr_make("service/11111");
    let response = directory
        .execute(&mut app, &service, &register_msg)
        .unwrap();

    assert_eq!(
        response.events,
        vec![
            Event::new("execute").add_attribute("_contract_address", directory.addr.as_str()),
            Event::new("wasm-ServiceRegistered")
                .add_attribute("_contract_address", directory.addr.as_str())
                .add_attribute("service", service.as_str()),
            Event::new("wasm-ServiceMetadataUpdated")
                .add_attribute("_contract_address", directory.addr.as_str())
                .add_attribute("service", service.as_str())
                .add_attribute("metadata.uri", "https://service.com")
                .add_attribute("metadata.name", "Service Name"),
        ]
    );
}

#[test]
fn register_service_but_paused_error() {
    let (mut app, directory, _, registry) = instantiate();
    let owner = app.api().addr_make("owner");

    let register_msg = &ExecuteMsg::ServiceRegister {
        metadata: ServiceMetadata {
            name: Some("Service Name".to_string()),
            uri: Some("https://service.com".to_string()),
        },
    };

    registry
        .execute(&mut app, &owner, &bvs_registry::msg::ExecuteMsg::Pause {})
        .unwrap();

    let err = directory
        .execute(&mut app, &owner, &register_msg)
        .unwrap_err();

    assert_eq!(
        err.root_cause().to_string(),
        ContractError::Registry(RegistryError::IsPaused).to_string()
    );
}

#[test]
fn register_service_but_already_registered_error() {
    let (mut app, directory, ..) = instantiate();

    let register_msg = &ExecuteMsg::ServiceRegister {
        metadata: ServiceMetadata {
            name: Some("Service Name".to_string()),
            uri: Some("https://service.com".to_string()),
        },
    };

    let service = app.api().addr_make("service/11111");
    directory
        .execute(&mut app, &service, &register_msg)
        .unwrap();

    let err = directory
        .execute(&mut app, &service, &register_msg)
        .unwrap_err();

    assert_eq!(
        err.root_cause().to_string(),
        ContractError::ServiceRegistered {}.to_string()
    );
}

#[test]
fn update_service_metadata_successfully() {
    let (mut app, directory, ..) = instantiate();

    let service = app.api().addr_make("service/11111");

    // regsiter service
    {
        let register_msg = &ExecuteMsg::ServiceRegister {
            metadata: ServiceMetadata {
                name: Some("Service Name".to_string()),
                uri: Some("https://service.com".to_string()),
            },
        };

        directory
            .execute(&mut app, &service, &register_msg)
            .unwrap();
    }

    // update name and uri
    {
        let update_msg = &ExecuteMsg::ServiceUpdateMetadata(ServiceMetadata {
            name: Some("New Service Name".to_string()),
            uri: Some("https://new-service.com".to_string()),
        });

        let response = directory.execute(&mut app, &service, &update_msg).unwrap();

        assert_eq!(
            response.events,
            vec![
                Event::new("execute").add_attribute("_contract_address", directory.addr.as_str()),
                Event::new("wasm-ServiceMetadataUpdated")
                    .add_attribute("_contract_address", directory.addr.as_str())
                    .add_attribute("service", service.as_str())
                    .add_attribute("metadata.uri", "https://new-service.com")
                    .add_attribute("metadata.name", "New Service Name"),
            ]
        );
    }

    // Don't update the name
    {
        let update_msg = &ExecuteMsg::ServiceUpdateMetadata(ServiceMetadata {
            name: None,
            uri: Some("https://new-new-service.com".to_string()),
        });

        let response = directory.execute(&mut app, &service, &update_msg).unwrap();

        assert_eq!(
            response.events,
            vec![
                Event::new("execute").add_attribute("_contract_address", directory.addr.as_str()),
                Event::new("wasm-ServiceMetadataUpdated")
                    .add_attribute("_contract_address", directory.addr.as_str())
                    .add_attribute("service", service.as_str())
                    .add_attribute("metadata.uri", "https://new-new-service.com"),
            ]
        );
    }
}

// Inactive -> ServiceRegistered
#[test]
fn service_register_operator_inactive_to_svc_registered_successfully() {
    let (mut app, directory, ..) = instantiate();

    let service = app.api().addr_make("service/bvs");

    // regsiter service
    {
        let register_msg = &ExecuteMsg::ServiceRegister {
            metadata: ServiceMetadata {
                name: Some("Service Name".to_string()),
                uri: Some("https://service.com".to_string()),
            },
        };

        directory
            .execute(&mut app, &service, &register_msg)
            .unwrap();
    }

    let operator = app.api().addr_make("operator");

    // register operator
    let msg = &ExecuteMsg::ServiceRegisterOperator {
        operator: operator.to_string(),
    };

    let res = directory.execute(&mut app, &service, &msg).unwrap();

    assert_eq!(
        res.events,
        vec![
            Event::new("execute").add_attribute("_contract_address", directory.addr.as_str()),
            Event::new("wasm-RegistrationStatusUpdated")
                .add_attribute("_contract_address", directory.addr.as_str())
                .add_attribute("method", "service_register_operator")
                .add_attribute("operator", operator.as_str())
                .add_attribute("service", service.as_str())
                .add_attribute("status", "ServiceRegistered"),
        ]
    );
}

// operator regsiters servcie and then service registers operator
#[test]
fn service_register_operator_operator_registered_successfully() {
    let (mut app, directory, delegation_manager, ..) = instantiate();

    let service = app.api().addr_make("service/bvs");

    // regsiter service
    {
        let register_msg = &ExecuteMsg::ServiceRegister {
            metadata: ServiceMetadata {
                name: Some("Service Name".to_string()),
                uri: Some("https://service.com".to_string()),
            },
        };

        directory
            .execute(&mut app, &service, &register_msg)
            .unwrap();
    }

    let operator = app.api().addr_make("operator");

    // register as operator in bvs-delegation-manager
    {
        let operator_details = OperatorDetails {
            staker_opt_out_window_blocks: 100,
        };
        let metadata_uri = "https://c4.service.com/metadata";
        let msg = bvs_delegation_manager::msg::ExecuteMsg::RegisterAsOperator {
            operator_details: operator_details.clone(),
            metadata_uri: metadata_uri.to_string(),
        };
        delegation_manager
            .execute(&mut app, &operator, &msg)
            .unwrap();
    }

    // operator registers service
    {
        let msg = ExecuteMsg::OperatorRegisterService {
            service: service.to_string(),
        };
        directory.execute(&mut app, &operator, &msg).unwrap();
    }

    // register operator
    let msg = &ExecuteMsg::ServiceRegisterOperator {
        operator: operator.to_string(),
    };

    let res = directory.execute(&mut app, &service, &msg).unwrap();
    assert_eq!(
        res.events,
        vec![
            Event::new("execute").add_attribute("_contract_address", directory.addr.as_str()),
            Event::new("wasm-RegistrationStatusUpdated")
                .add_attribute("_contract_address", directory.addr.as_str())
                .add_attribute("method", "service_register_operator")
                .add_attribute("operator", operator.as_str())
                .add_attribute("service", service.as_str())
                .add_attribute("status", "Active"),
        ]
    );
}

#[test]
fn service_register_operator_already_active_error() {
    let (mut app, directory, delegation_manager, ..) = instantiate();

    let service = app.api().addr_make("service/bvs");

    // regsiter service
    {
        let register_msg = &ExecuteMsg::ServiceRegister {
            metadata: ServiceMetadata {
                name: Some("Service Name".to_string()),
                uri: Some("https://service.com".to_string()),
            },
        };

        directory
            .execute(&mut app, &service, &register_msg)
            .unwrap();
    }

    let operator = app.api().addr_make("operator");

    // register as operator in bvs-delegation-manager
    {
        let operator_details = OperatorDetails {
            staker_opt_out_window_blocks: 100,
        };
        let metadata_uri = "https://c4.service.com/metadata";
        let msg = bvs_delegation_manager::msg::ExecuteMsg::RegisterAsOperator {
            operator_details: operator_details.clone(),
            metadata_uri: metadata_uri.to_string(),
        };
        delegation_manager
            .execute(&mut app, &operator, &msg)
            .unwrap();
    }

    // operator registers service
    {
        let msg = ExecuteMsg::OperatorRegisterService {
            service: service.to_string(),
        };
        directory.execute(&mut app, &operator, &msg).unwrap();
    }

    let msg = &ExecuteMsg::ServiceRegisterOperator {
        operator: operator.to_string(),
    };

    directory.execute(&mut app, &service, &msg).unwrap();

    // services register operator again
    let err = directory.execute(&mut app, &service, &msg).unwrap_err();
    assert_eq!(
        err.root_cause().to_string(),
        ContractError::InvalidRegistrationStatus {
            msg: "Registration is already active.".to_string(),
        }
        .to_string()
    );
}

// Inactive -> ServiceRegistered -> error
#[test]
fn service_register_operator_inactive_to_svc_already_registered_error() {
    let (mut app, directory, ..) = instantiate();

    let service = app.api().addr_make("service/bvs");

    // regsiter service
    {
        let register_msg = &ExecuteMsg::ServiceRegister {
            metadata: ServiceMetadata {
                name: Some("Service Name".to_string()),
                uri: Some("https://service.com".to_string()),
            },
        };

        directory
            .execute(&mut app, &service, &register_msg)
            .unwrap();
    }

    // register operator
    let operator = app.api().addr_make("operator");
    let msg = &ExecuteMsg::ServiceRegisterOperator {
        operator: operator.to_string(),
    };

    // first register operator
    directory.execute(&mut app, &service, &msg).unwrap();

    // register again
    let err = directory.execute(&mut app, &service, &msg).unwrap_err();
    assert_eq!(
        err.root_cause().to_string(),
        ContractError::InvalidRegistrationStatus {
            msg: "Service has already registered.".to_string()
        }
        .to_string()
    );
}

#[test]
fn service_deregister_operator_successfully() {
    let (mut app, directory, ..) = instantiate();

    let service = app.api().addr_make("service/bvs");

    // regsiter service
    {
        let register_msg = &ExecuteMsg::ServiceRegister {
            metadata: ServiceMetadata {
                name: Some("Service Name".to_string()),
                uri: Some("https://service.com".to_string()),
            },
        };

        directory
            .execute(&mut app, &service, &register_msg)
            .unwrap();
    }

    // register operator
    let operator = app.api().addr_make("operator");
    {
        let msg = &ExecuteMsg::ServiceRegisterOperator {
            operator: operator.to_string(),
        };

        directory.execute(&mut app, &service, &msg).unwrap();
    }

    // deregister operator
    let msg = &ExecuteMsg::ServiceDeregisterOperator {
        operator: operator.to_string(),
    };

    let res = directory.execute(&mut app, &service, &msg).unwrap();
    assert_eq!(
        res.events,
        vec![
            Event::new("execute").add_attribute("_contract_address", directory.addr.as_str()),
            Event::new("wasm-RegistrationStatusUpdated")
                .add_attribute("_contract_address", directory.addr.as_str())
                .add_attribute("method", "service_deregister_operator")
                .add_attribute("operator", operator.as_str())
                .add_attribute("service", service.as_str())
                .add_attribute("status", "Inactive"),
        ]
    );
}

#[test]
fn service_deregister_operator_already_registered_error() {
    let (mut app, directory, ..) = instantiate();

    let service = app.api().addr_make("service/bvs");

    // regsiter service
    {
        let register_msg = &ExecuteMsg::ServiceRegister {
            metadata: ServiceMetadata {
                name: Some("Service Name".to_string()),
                uri: Some("https://service.com".to_string()),
            },
        };

        directory
            .execute(&mut app, &service, &register_msg)
            .unwrap();
    }

    // register operator
    let operator = app.api().addr_make("operator");
    {
        let msg = &ExecuteMsg::ServiceRegisterOperator {
            operator: operator.to_string(),
        };

        directory.execute(&mut app, &service, &msg).unwrap();
    }

    // deregister operator
    let msg = &ExecuteMsg::ServiceDeregisterOperator {
        operator: operator.to_string(),
    };

    directory.execute(&mut app, &service, &msg).unwrap();

    // deregister operator again
    let err = directory.execute(&mut app, &service, &msg).unwrap_err();
    assert_eq!(
        err.root_cause().to_string(),
        ContractError::InvalidRegistrationStatus {
            msg: "Already deregistered.".to_string()
        }
        .to_string()
    );
}

#[test]
fn operator_register_service_inactive_operator_registered_successfully() {
    let (mut app, directory, delegation_manager, ..) = instantiate();

    let service = app.api().addr_make("service/c4");

    // register service
    {
        let register_msg = &ExecuteMsg::ServiceRegister {
            metadata: ServiceMetadata {
                name: Some("C4 Service".to_string()),
                uri: Some("https://c4.service.com".to_string()),
            },
        };

        directory
            .execute(&mut app, &service, &register_msg)
            .unwrap();
    }

    let operator = app.api().addr_make("operator");

    // register as operator in bvs-delegation-manager
    {
        let operator_details = OperatorDetails {
            staker_opt_out_window_blocks: 100,
        };
        let metadata_uri = "https://c4.service.com/metadata";
        let msg = bvs_delegation_manager::msg::ExecuteMsg::RegisterAsOperator {
            operator_details: operator_details.clone(),
            metadata_uri: metadata_uri.to_string(),
        };
        delegation_manager
            .execute(&mut app, &operator, &msg)
            .unwrap();
    }

    let msg = ExecuteMsg::OperatorRegisterService {
        service: service.to_string(),
    };
    let res = directory.execute(&mut app, &operator, &msg).unwrap();

    assert_eq!(
        res.events,
        vec![
            Event::new("execute").add_attribute("_contract_address", directory.addr.as_str()),
            Event::new("wasm-RegistrationStatusUpdated")
                .add_attribute("_contract_address", directory.addr.as_str())
                .add_attribute("method", "operator_register_service")
                .add_attribute("operator", operator.as_str())
                .add_attribute("service", service.as_str())
                .add_attribute("status", "OperatorRegistered"),
        ]
    );

    let status: StatusResponse = directory
        .query(
            &mut app,
            &QueryMsg::Status {
                service: service.to_string(),
                operator: operator.to_string(),
            },
        )
        .unwrap();
    assert_eq!(status, StatusResponse(2));
}

#[test]
fn operator_register_service_service_registered_successfully() {
    let (mut app, directory, delegation_manager, ..) = instantiate();

    let service = app.api().addr_make("service/c4");

    // register service
    {
        let register_msg = &ExecuteMsg::ServiceRegister {
            metadata: ServiceMetadata {
                name: Some("C4 Service".to_string()),
                uri: Some("https://c4.service.com".to_string()),
            },
        };

        directory
            .execute(&mut app, &service, &register_msg)
            .unwrap();
    }

    let operator = app.api().addr_make("operator");

    // register operator in bvs-directory
    {
        let msg = &ExecuteMsg::ServiceRegisterOperator {
            operator: operator.to_string(),
        };
        directory.execute(&mut app, &service, &msg).unwrap();
    }

    // register as operator in bvs-delegation-manager
    {
        let operator_details = OperatorDetails {
            staker_opt_out_window_blocks: 100,
        };
        let metadata_uri = "https://c4.service.com/metadata";
        let msg = bvs_delegation_manager::msg::ExecuteMsg::RegisterAsOperator {
            operator_details: operator_details.clone(),
            metadata_uri: metadata_uri.to_string(),
        };
        delegation_manager
            .execute(&mut app, &operator, &msg)
            .unwrap();
    }

    let msg = ExecuteMsg::OperatorRegisterService {
        service: service.to_string(),
    };
    let res = directory.execute(&mut app, &operator, &msg).unwrap();

    assert_eq!(
        res.events,
        vec![
            Event::new("execute").add_attribute("_contract_address", directory.addr.as_str()),
            Event::new("wasm-RegistrationStatusUpdated")
                .add_attribute("_contract_address", directory.addr.as_str())
                .add_attribute("method", "operator_register_service")
                .add_attribute("operator", operator.as_str())
                .add_attribute("service", service.as_str())
                .add_attribute("status", "Active"),
        ]
    );

    let status: StatusResponse = directory
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
fn operator_register_service_operator_not_found_error() {
    let (mut app, directory, _, _) = instantiate();
    let not_operator = app.api().addr_make("not_operator");

    let register_msg = &ExecuteMsg::ServiceRegister {
        metadata: ServiceMetadata {
            name: Some("Service Name".to_string()),
            uri: Some("https://service.com".to_string()),
        },
    };

    let service = app.api().addr_make("service/11111");
    directory
        .execute(&mut app, &service, &register_msg)
        .unwrap();

    let register_msg = &ExecuteMsg::OperatorRegisterService {
        service: service.to_string(),
    };

    let err = directory
        .execute(&mut app, &not_operator, &register_msg)
        .unwrap_err();

    assert_eq!(
        err.root_cause().to_string(),
        ContractError::OperatorNotFound {
            msg: "Operator is not registered on delegation manager.".to_string()
        }
        .to_string()
    );
}

#[test]
fn operator_register_service_already_active_error() {
    let (mut app, directory, delegation_manager, ..) = instantiate();

    let service = app.api().addr_make("service/c4");

    // register service
    {
        let register_msg = &ExecuteMsg::ServiceRegister {
            metadata: ServiceMetadata {
                name: Some("C4 Service".to_string()),
                uri: Some("https://c4.service.com".to_string()),
            },
        };

        directory
            .execute(&mut app, &service, &register_msg)
            .unwrap();
    }

    let operator = app.api().addr_make("operator");

    // register operator in bvs-directory
    {
        let msg = &ExecuteMsg::ServiceRegisterOperator {
            operator: operator.to_string(),
        };
        directory.execute(&mut app, &service, &msg).unwrap();
    }

    // register as operator in bvs-delegation-manager
    {
        let operator_details = OperatorDetails {
            staker_opt_out_window_blocks: 100,
        };
        let metadata_uri = "https://c4.service.com/metadata";
        let msg = bvs_delegation_manager::msg::ExecuteMsg::RegisterAsOperator {
            operator_details: operator_details.clone(),
            metadata_uri: metadata_uri.to_string(),
        };
        delegation_manager
            .execute(&mut app, &operator, &msg)
            .unwrap();
    }

    let msg = ExecuteMsg::OperatorRegisterService {
        service: service.to_string(),
    };
    directory.execute(&mut app, &operator, &msg).unwrap();

    let err = directory.execute(&mut app, &operator, &msg).unwrap_err();
    assert_eq!(
        err.root_cause().to_string(),
        ContractError::InvalidRegistrationStatus {
            msg: "Registration is already active.".to_string(),
        }
        .to_string()
    );
}

#[test]
fn operator_register_service_operator_already_registered_error() {
    let (mut app, directory, delegation_manager, ..) = instantiate();

    let service = app.api().addr_make("service/c4");

    // register service
    {
        let register_msg = &ExecuteMsg::ServiceRegister {
            metadata: ServiceMetadata {
                name: Some("C4 Service".to_string()),
                uri: Some("https://c4.service.com".to_string()),
            },
        };

        directory
            .execute(&mut app, &service, &register_msg)
            .unwrap();
    }

    let operator = app.api().addr_make("operator");

    // register as operator in bvs-delegation-manager
    {
        let operator_details = OperatorDetails {
            staker_opt_out_window_blocks: 100,
        };
        let metadata_uri = "https://c4.service.com/metadata";
        let msg = bvs_delegation_manager::msg::ExecuteMsg::RegisterAsOperator {
            operator_details: operator_details.clone(),
            metadata_uri: metadata_uri.to_string(),
        };
        delegation_manager
            .execute(&mut app, &operator, &msg)
            .unwrap();
    }

    let msg = ExecuteMsg::OperatorRegisterService {
        service: service.to_string(),
    };
    directory.execute(&mut app, &operator, &msg).unwrap();

    // register operator again
    let err = directory.execute(&mut app, &operator, &msg).unwrap_err();
    assert_eq!(
        err.root_cause().to_string(),
        ContractError::InvalidRegistrationStatus {
            msg: "Operator has already registered.".to_string(),
        }
        .to_string()
    );
}

#[test]
fn operator_deregister_service_successfully() {
    let (mut app, directory, delegation_manager, ..) = instantiate();

    let service = app.api().addr_make("service/c4");

    // register service
    {
        let register_msg = &ExecuteMsg::ServiceRegister {
            metadata: ServiceMetadata {
                name: Some("C4 Service".to_string()),
                uri: Some("https://c4.service.com".to_string()),
            },
        };

        directory
            .execute(&mut app, &service, &register_msg)
            .unwrap();
    }

    let operator = app.api().addr_make("operator");

    // register operator in bvs-directory
    {
        let msg = &ExecuteMsg::ServiceRegisterOperator {
            operator: operator.to_string(),
        };
        directory.execute(&mut app, &service, &msg).unwrap();
    }

    // register as operator in bvs-delegation-manager
    {
        let operator_details = OperatorDetails {
            staker_opt_out_window_blocks: 100,
        };
        let metadata_uri = "https://c4.service.com/metadata";
        let msg = bvs_delegation_manager::msg::ExecuteMsg::RegisterAsOperator {
            operator_details: operator_details.clone(),
            metadata_uri: metadata_uri.to_string(),
        };
        delegation_manager
            .execute(&mut app, &operator, &msg)
            .unwrap();
    }

    // operator register service
    {
        let msg = ExecuteMsg::OperatorRegisterService {
            service: service.to_string(),
        };
        directory.execute(&mut app, &operator, &msg).unwrap();
    }

    let msg = ExecuteMsg::OperatorDeregisterService {
        service: service.to_string(),
    };
    let res = directory.execute(&mut app, &operator, &msg).unwrap();
    assert_eq!(
        res.events,
        vec![
            Event::new("execute").add_attribute("_contract_address", directory.addr.as_str()),
            Event::new("wasm-RegistrationStatusUpdated")
                .add_attribute("_contract_address", directory.addr.as_str())
                .add_attribute("method", "operator_deregister_service")
                .add_attribute("operator", operator.as_str())
                .add_attribute("service", service.as_str())
                .add_attribute("status", "Inactive"),
        ]
    );

    let status: StatusResponse = directory
        .query(
            &mut app,
            &QueryMsg::Status {
                service: service.to_string(),
                operator: operator.to_string(),
            },
        )
        .unwrap();
    assert_eq!(status, StatusResponse(0));
}

#[test]
fn operator_deregister_service_deregister_error() {
    let (mut app, directory, delegation_manager, ..) = instantiate();

    let service = app.api().addr_make("service/c4");

    // register service
    {
        let register_msg = &ExecuteMsg::ServiceRegister {
            metadata: ServiceMetadata {
                name: Some("C4 Service".to_string()),
                uri: Some("https://c4.service.com".to_string()),
            },
        };

        directory
            .execute(&mut app, &service, &register_msg)
            .unwrap();
    }

    let operator = app.api().addr_make("operator");

    // register operator in bvs-directory
    {
        let msg = &ExecuteMsg::ServiceRegisterOperator {
            operator: operator.to_string(),
        };
        directory.execute(&mut app, &service, &msg).unwrap();
    }

    // register as operator in bvs-delegation-manager
    {
        let operator_details = OperatorDetails {
            staker_opt_out_window_blocks: 100,
        };
        let metadata_uri = "https://c4.service.com/metadata";
        let msg = bvs_delegation_manager::msg::ExecuteMsg::RegisterAsOperator {
            operator_details: operator_details.clone(),
            metadata_uri: metadata_uri.to_string(),
        };
        delegation_manager
            .execute(&mut app, &operator, &msg)
            .unwrap();
    }

    // operator register service
    {
        let msg = ExecuteMsg::OperatorRegisterService {
            service: service.to_string(),
        };
        directory.execute(&mut app, &operator, &msg).unwrap();
    }

    let msg = ExecuteMsg::OperatorDeregisterService {
        service: service.to_string(),
    };
    directory.execute(&mut app, &operator, &msg).unwrap();

    // deregister service again
    let err = directory.execute(&mut app, &operator, &msg).unwrap_err();
    assert_eq!(
        err.root_cause().to_string(),
        ContractError::InvalidRegistrationStatus {
            msg: "Already deregistered.".to_string(),
        }
        .to_string()
    );
}

#[test]
fn operator_register_service_but_service_not_registered() {
    let (mut app, directory, _, _) = instantiate();
    let operator = app.api().addr_make("operator");

    let register_msg = &ExecuteMsg::OperatorRegisterService {
        service: app.api().addr_make("service/11111").to_string(),
    };

    let err = directory
        .execute(&mut app, &operator, &register_msg)
        .unwrap_err();

    assert_eq!(
        err.root_cause().to_string(),
        ContractError::ServiceNotFound {}.to_string()
    );
}

#[test]
fn transfer_ownership_successfully() {
    let (mut app, directory, _, _) = instantiate();
    let owner = app.api().addr_make("owner");
    let new_owner = app.api().addr_make("new_owner");

    let transfer_msg = &ExecuteMsg::TransferOwnership {
        new_owner: new_owner.to_string(),
    };

    let response = directory.execute(&mut app, &owner, &transfer_msg).unwrap();

    assert_eq!(
        response.events,
        vec![
            Event::new("execute").add_attribute("_contract_address", directory.addr.as_str()),
            Event::new("wasm-TransferredOwnership")
                .add_attribute("_contract_address", directory.addr.as_str())
                .add_attribute("old_owner", owner.as_str())
                .add_attribute("new_owner", new_owner.as_str()),
        ]
    );
}

#[test]
fn transfer_ownership_but_not_owner() {
    let (mut app, directory, _, _) = instantiate();
    let not_owner = app.api().addr_make("not_owner");

    let transfer_msg = &ExecuteMsg::TransferOwnership {
        new_owner: not_owner.to_string(),
    };

    let err = directory
        .execute(&mut app, &not_owner, &transfer_msg)
        .unwrap_err();

    assert_eq!(
        err.root_cause().to_string(),
        ContractError::Unauthorized {}.to_string()
    );
}

#[test]
fn query_status() {
    let (mut app, directory, _, _) = instantiate();

    let query_msg = &QueryMsg::Status {
        service: app.api().addr_make("service/44").to_string(),
        operator: app.api().addr_make("operator/44").to_string(),
    };

    let status: StatusResponse = directory.query(&mut app, query_msg).unwrap();
    assert_eq!(status, StatusResponse(0));
}
