use base64::{engine::general_purpose, Engine as _};
use bvs_delegation_manager::msg::InstantiateMsg as BvsDelegationManagerInstantiateMsg;
use bvs_directory::{
    msg::{ExecuteMsg as BvsDirectoryExecuteMsg, InstantiateMsg as BvsDirectoryInstantiateMsg},
    utils::{calculate_digest_hash, sha256},
};
use bvs_library::testing::{Account, TestingContract};
use bvs_registry::msg::InstantiateMsg;
use bvs_registry::testing::RegistryContract;
use bvs_testing::integration::{mock_contracts::mock_app, mock_env::MockEnvBuilder};
use cosmwasm_std::testing::mock_env;
use cosmwasm_std::{Binary, Event, StdError};

#[test]
fn register_bvs_successfully() {
    let mut app = mock_app();
    let env = mock_env();

    let owner = app.api().addr_make("owner");
    let empty_addr = app.api().addr_make("");
    let bvs_contract = app.api().addr_make("bvs_contract").to_string();
    let anyone = app.api().addr_make("anyone");

    let hash_result = sha256(bvs_contract.clone().as_bytes());
    let bvs_hash = hex::encode(hash_result);

    let registry = RegistryContract::new(&mut app, &env, None);

    let mut mock_env = MockEnvBuilder::new(app, None, owner.clone())
        .deploy_bvs_directory(&BvsDirectoryInstantiateMsg {
            initial_owner: owner.clone().to_string(),
            delegation_manager: empty_addr.to_string(),
            registry: registry.addr.to_string(),
        })
        .build();

    let register_bvs_msg = &BvsDirectoryExecuteMsg::RegisterBvs {
        bvs_contract: bvs_contract.clone(),
    };

    let bvs_driectory = mock_env.bvs_directory.clone();
    let response = bvs_driectory
        .execute(&mut mock_env, anyone, register_bvs_msg, &[])
        .unwrap();

    assert_eq!(response.events.len(), 2);
    assert_eq!(
        response.events[1],
        Event::new("wasm")
            .add_attribute("_contract_address", mock_env.bvs_directory.contract_addr)
            .add_attribute("method", "register_bvs")
            .add_attribute("bvs_hash", bvs_hash.clone())
    );
}

// TODO: need solve contract dependencies: bvs-delegation-manager and bvs-strategy-manager
#[test]
fn register_operator_failure() {
    let mut app = mock_app();
    let env = mock_env();

    let operator = Account::new("operator".into());
    let expiry = 2722875888;
    let salt = Binary::from(b"salt");
    let contract_addr = app.api().addr_make("contract_addr");
    let owner = app.api().addr_make("owner");

    let message_hash = calculate_digest_hash(
        app.block_info().chain_id,
        &Binary::from(operator.public_key.clone().serialize()),
        &owner,
        &salt,
        expiry,
        &contract_addr,
    );
    let signature = operator.sign(message_hash);
    let signature_bytes = signature.serialize_compact().to_vec();
    let signature_base64 = general_purpose::STANDARD.encode(signature_bytes);

    let owner = app.api().addr_make("owner");
    let empty_addr = app.api().addr_make("");
    let strategy1 = app.api().addr_make("strategy1").to_string();
    let strategy2 = app.api().addr_make("strategy2").to_string();

    let registry = RegistryContract::new(&mut app, &env, None);

    let mut mock_env = MockEnvBuilder::new(app, None, owner.clone())
        .deploy_bvs_directory(&BvsDirectoryInstantiateMsg {
            initial_owner: owner.clone().to_string(),
            delegation_manager: empty_addr.clone().into_string(),
            registry: registry.addr.to_string(),
        })
        .deploy_bvs_delegation_manager(&BvsDelegationManagerInstantiateMsg {
            strategy_manager: empty_addr.clone().into_string(),
            slash_manager: empty_addr.clone().into_string(),
            min_withdrawal_delay_blocks: 100,
            initial_owner: owner.clone().to_string(),
            strategies: vec![strategy1.clone(), strategy2.clone()],
            withdrawal_delay_blocks: vec![50, 60],
            pauser: owner.clone().to_string(),
            unpauser: owner.clone().to_string(),
            initial_paused_status: 0,
        })
        .build();

    let set_delegation_manager_msg = &BvsDirectoryExecuteMsg::SetDelegationManager {
        delegation_manager: mock_env
            .bvs_delegation_manager
            .contract_addr
            .clone()
            .into_string(),
    };

    let bvs_directory = mock_env.bvs_directory.clone();
    let bvs_delegation_manager_addr = mock_env.bvs_delegation_manager.contract_addr.clone();
    let response = bvs_directory
        .clone()
        .execute(
            &mut mock_env,
            owner.clone(),
            set_delegation_manager_msg.into(),
            &[],
        )
        .unwrap();
    assert_eq!(response.events.len(), 2);
    assert_eq!(
        response.events[1],
        Event::new("wasm")
            .add_attribute("_contract_address", bvs_directory.clone().contract_addr)
            .add_attribute("method", "set_delegation_manager")
            .add_attribute(
                "delegation_manager",
                bvs_delegation_manager_addr.into_string()
            )
    );

    let regsiter_operator_to_bvs_msg = &BvsDirectoryExecuteMsg::RegisterOperatorToBvs {
        operator: operator.address.to_string(),
        public_key: Binary::from_base64(&operator.public_key_base64()).unwrap(),
        contract_addr: contract_addr.to_string(),
        signature_with_salt_and_expiry: bvs_directory::msg::SignatureWithSaltAndExpiry {
            signature: Binary::from_base64(&signature_base64).unwrap(),
            salt,
            expiry,
        },
    };
    let response = bvs_directory.clone().execute(
        &mut mock_env,
        owner.clone(),
        regsiter_operator_to_bvs_msg.into(),
        &[],
    );
    assert!(response.is_err())
}

// TODO: need solve contract dependencies: bvs-delegation-manager and bvs-strategy-manager
#[test]
fn deregister_operator_failure() {
    let mut app = mock_app();
    let env = mock_env();

    let owner = app.api().addr_make("owner");
    let empty_addr = app.api().addr_make("");
    let operator = app.api().addr_make("operator");

    let registry = RegistryContract::new(&mut app, &env, None);

    let mut mock_env = MockEnvBuilder::new(app, None, owner.clone())
        .deploy_bvs_directory(&BvsDirectoryInstantiateMsg {
            initial_owner: owner.clone().to_string(),
            delegation_manager: empty_addr.to_string(),
            registry: registry.addr.to_string(),
        })
        .build();

    let deregister_operator_msg = &BvsDirectoryExecuteMsg::DeregisterOperatorFromBvs {
        operator: operator.clone().into_string(),
    };
    let directory = mock_env.bvs_directory.clone();
    let response = directory.execute(&mut mock_env, owner.clone(), deregister_operator_msg, &[]);

    assert!(response.is_err());
}

#[test]
fn register_update_metadata_uri_successfully() {
    let mut app = mock_app();
    let env = mock_env();

    let owner = app.api().addr_make("owner");
    let empty_addr = app.api().addr_make("");
    let anyone = app.api().addr_make("anyone");

    let registry = RegistryContract::new(&mut app, &env, None);

    let mut mock_env = MockEnvBuilder::new(app, None, owner.clone())
        .deploy_bvs_directory(&BvsDirectoryInstantiateMsg {
            initial_owner: owner.clone().to_string(),
            delegation_manager: empty_addr.to_string(),
            registry: registry.addr.to_string(),
        })
        .build();

    let metadata_uri = "https://bvs.com".to_string();
    let update_metadata_msg = &BvsDirectoryExecuteMsg::UpdateBvsMetadataUri {
        metadata_uri: metadata_uri.clone(),
    };

    let bvs_driectory = mock_env.bvs_directory.clone();
    let response = bvs_driectory
        .execute(&mut mock_env, anyone.clone(), update_metadata_msg, &[])
        .unwrap();

    assert_eq!(response.events.len(), 2);
    assert_eq!(
        response.events[1],
        Event::new("wasm-BVSMetadataURIUpdated")
            .add_attribute("_contract_address", mock_env.bvs_directory.contract_addr)
            .add_attribute("method", "update_metadata_uri")
            .add_attribute("bvs", anyone.into_string())
            .add_attribute("metadata_uri", metadata_uri)
    );
}

#[test]
fn set_delegation_manager_successfully() {
    let mut app = mock_app();
    let env = mock_env();

    let owner = app.api().addr_make("owner");
    let empty_addr = app.api().addr_make("");
    let delegation_manager_addr = app.api().addr_make("delegation_manager");

    let registry = RegistryContract::new(&mut app, &env, None);

    let mut mock_env = MockEnvBuilder::new(app, None, owner.clone())
        .deploy_bvs_directory(&BvsDirectoryInstantiateMsg {
            initial_owner: owner.clone().to_string(),
            delegation_manager: empty_addr.to_string(),
            registry: registry.addr.to_string(),
        })
        .build();

    let set_delegation_manager_msg = &BvsDirectoryExecuteMsg::SetDelegationManager {
        delegation_manager: delegation_manager_addr.clone().into_string(),
    };

    let bvs_driectory = mock_env.bvs_directory.clone();
    let response = bvs_driectory
        .execute(
            &mut mock_env,
            owner.clone(),
            set_delegation_manager_msg,
            &[],
        )
        .unwrap();

    assert_eq!(response.events.len(), 2);
    assert_eq!(
        response.events[1],
        Event::new("wasm")
            .add_attribute("_contract_address", mock_env.bvs_directory.contract_addr)
            .add_attribute("method", "set_delegation_manager")
            .add_attribute("delegation_manager", delegation_manager_addr.into_string())
    );
}

#[test]
fn cancel_salt_successfully() {
    let mut app = mock_app();
    let env = mock_env();

    let owner = app.api().addr_make("owner");
    let empty_addr = app.api().addr_make("");
    let anyone = app.api().addr_make("anyone");
    let salt = Binary::from(b"salt");

    let registry = RegistryContract::new(&mut app, &env, None);

    let mut mock_env = MockEnvBuilder::new(app, None, owner.clone())
        .deploy_bvs_directory(&BvsDirectoryInstantiateMsg {
            initial_owner: owner.clone().to_string(),
            delegation_manager: empty_addr.to_string(),
            registry: registry.addr.to_string(),
        })
        .build();

    let set_delegation_manager_msg = &BvsDirectoryExecuteMsg::CancelSalt { salt: salt.clone() };

    let bvs_driectory = mock_env.bvs_directory.clone();
    let response = bvs_driectory
        .execute(
            &mut mock_env,
            anyone.clone(),
            set_delegation_manager_msg,
            &[],
        )
        .unwrap();

    assert_eq!(response.events.len(), 2);
    assert_eq!(
        response.events[1],
        Event::new("wasm")
            .add_attribute("_contract_address", mock_env.bvs_directory.contract_addr)
            .add_attribute("method", "cancel_salt")
            .add_attribute("operator", anyone.into_string())
            .add_attribute("salt", salt.to_base64())
    );
}

#[test]
fn transfer_ownership_successfully() {
    let mut app = mock_app();
    let env = mock_env();

    let owner = app.api().addr_make("owner");
    let empty_addr = app.api().addr_make("");
    let anyone = app.api().addr_make("anyone");

    let registry = RegistryContract::new(&mut app, &env, None);

    let mut mock_env = MockEnvBuilder::new(app, None, owner.clone())
        .deploy_bvs_directory(&BvsDirectoryInstantiateMsg {
            initial_owner: owner.clone().to_string(),
            delegation_manager: empty_addr.to_string(),
            registry: registry.addr.to_string(),
        })
        .build();

    let set_delegation_manager_msg = &BvsDirectoryExecuteMsg::TransferOwnership {
        new_owner: anyone.clone().into_string(),
    };

    let bvs_driectory = mock_env.bvs_directory.clone();
    let response = bvs_driectory
        .execute(
            &mut mock_env,
            owner.clone(),
            set_delegation_manager_msg,
            &[],
        )
        .unwrap();

    assert_eq!(response.events.len(), 2);
    assert_eq!(
        response.events[1],
        Event::new("wasm")
            .add_attribute("_contract_address", mock_env.bvs_directory.contract_addr)
            .add_attribute("method", "transfer_ownership")
            .add_attribute("new_owner", anyone.into_string())
    );
}

#[test]
fn register_bvs_but_paused() {
    let mut app = mock_app();
    let env = mock_env();

    let owner = app.api().addr_make("owner");
    let delegation_manager = app.api().addr_make("delegation_manager");
    let bvs_contract = app.api().addr_make("bvs_contract").to_string();
    let anyone = app.api().addr_make("anyone");

    let registry = RegistryContract::new(
        &mut app,
        &env,
        InstantiateMsg {
            owner: owner.to_string(),
            initial_paused: true,
        }
        .into(),
    );

    let mut mock_env = MockEnvBuilder::new(app, None, owner.clone())
        .deploy_bvs_directory(&bvs_directory::msg::InstantiateMsg {
            initial_owner: owner.clone().to_string(),
            delegation_manager: delegation_manager.into_string(),
            registry: registry.addr.to_string(),
        })
        .build();

    let register_bvs_msg = &bvs_directory::msg::ExecuteMsg::RegisterBvs {
        bvs_contract: bvs_contract.clone(),
    };

    let directory = mock_env.bvs_directory.clone();

    let err = directory
        .execute(&mut mock_env, anyone, register_bvs_msg, &[])
        .unwrap_err();

    assert_eq!(
        err.root_cause().to_string(),
        bvs_directory::ContractError::Std(StdError::generic_err("Paused")).to_string()
    );
}
