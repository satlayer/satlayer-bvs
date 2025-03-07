use bvs_delegation_manager::testing::DelegationManagerContract;
use bvs_directory::msg::{ExecuteMsg, QueryMsg, ServiceMetadata, StatusResponse};
use bvs_directory::testing::DirectoryContract;
use bvs_directory::ContractError;
use bvs_library::testing::TestingContract;
use bvs_pauser::api::PauserError;
use bvs_pauser::testing::PauserContract;
use cosmwasm_std::testing::mock_env;
use cosmwasm_std::Event;
use cw_multi_test::App;

fn instantiate() -> (
    App,
    DirectoryContract,
    DelegationManagerContract,
    PauserContract,
) {
    let mut app = App::default();
    let env = mock_env();

    let pauser = PauserContract::new(&mut app, &env, None);
    let delegation = DelegationManagerContract::new(&mut app, &env, None);
    let directory = DirectoryContract::new(&mut app, &env, None);

    let owner = app.api().addr_make("owner");
    let not_routed = app.api().addr_make("not_routed");

    directory
        .execute(
            &mut app,
            &owner,
            &ExecuteMsg::SetRouting {
                delegation_manager: delegation.addr.to_string(),
            },
        )
        .unwrap();

    delegation
        .execute(
            &mut app,
            &owner,
            &bvs_delegation_manager::msg::ExecuteMsg::SetRouting {
                strategy_manager: not_routed.to_string(),
                slash_manager: not_routed.to_string(),
            },
        )
        .unwrap();

    (app, directory, delegation, pauser)
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
fn register_service_but_paused() {
    let (mut app, directory, _, pauser) = instantiate();
    let owner = app.api().addr_make("owner");

    let register_msg = &ExecuteMsg::ServiceRegister {
        metadata: ServiceMetadata {
            name: Some("Service Name".to_string()),
            uri: Some("https://service.com".to_string()),
        },
    };

    pauser
        .execute(&mut app, &owner, &bvs_pauser::msg::ExecuteMsg::Pause {})
        .unwrap();

    let err = directory
        .execute(&mut app, &owner, &register_msg)
        .unwrap_err();

    assert_eq!(
        err.root_cause().to_string(),
        ContractError::Pauser(PauserError::IsPaused).to_string()
    );
}

#[test]
fn register_service_but_already_registered() {
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
fn operator_register_service_but_self_not_operator() {
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
fn register_lifecycle_operator_first() {
    let (mut app, directory, delegation, ..) = instantiate();

    let register_msg = &ExecuteMsg::ServiceRegister {
        metadata: ServiceMetadata {
            name: Some("Service Name".to_string()),
            uri: Some("https://service.com".to_string()),
        },
    };

    let service = app.api().addr_make("service/bvs");
    directory
        .execute(&mut app, &service, &register_msg)
        .unwrap();

    // TODO(fuxingloh): need strategy-manager setup.
    // let operator = app.api().addr_make("operator");
    // let register_msg = &bvs_delegation_manager::msg::ExecuteMsg::RegisterAsOperator {
    //     operator_details: bvs_delegation_manager::msg::OperatorDetails {
    //         staker_opt_out_window_blocks: 100
    //     },
    //     metadata_uri: "operator.com".to_string(),
    // };
}

#[test]
fn register_lifecycle_service_first() {
    let (mut app, directory, delegation, ..) = instantiate();

    let register_msg = &ExecuteMsg::ServiceRegister {
        metadata: ServiceMetadata {
            name: Some("C4 Service".to_string()),
            uri: Some("https://c4.service.com".to_string()),
        },
    };

    let service = app.api().addr_make("service/c4");
    let operator = app.api().addr_make("operator");
    directory
        .execute(&mut app, &service, &register_msg)
        .unwrap();

    // Register Service

    let register_msg = &ExecuteMsg::ServiceRegisterOperator {
        operator: operator.to_string(),
    };

    let res = directory
        .execute(&mut app, &service, &register_msg)
        .unwrap();

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

    let status: StatusResponse = directory
        .query(
            &mut app,
            &QueryMsg::Status {
                service: service.to_string(),
                operator: operator.to_string(),
            },
        )
        .unwrap();
    assert_eq!(status, StatusResponse(3));

    // TODO(fuxingloh): need strategy-manager setup.
    // let operator = app.api().addr_make("operator");
    // let register_msg = &bvs_delegation_manager::msg::ExecuteMsg::RegisterAsOperator {
    //     operator_details: bvs_delegation_manager::msg::OperatorDetails {
    //         staker_opt_out_window_blocks: 100
    //     },
    //     metadata_uri: "operator.com".to_string(),
    // };
}

// TODO: deregister from service
// TODO: deregister from operator
// TODO: already deregistered
// TODO: already active
// TODO: operator already registered
// TODO: service already registered

#[test]
fn update_metadata_successfully() {
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

    // Don't update the name
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

    let status: StatusResponse = directory.query(&mut app, query_msg).unwrap();

    assert_eq!(status, StatusResponse(0));
}
