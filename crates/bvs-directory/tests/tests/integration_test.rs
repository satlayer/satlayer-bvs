use bvs_testing::integration::*;
use cosmwasm_std::{Addr, Event, StdError};
use cw_multi_test::{App, Executor};

#[test]
fn register_bvs() {
    let mut app = App::default();

    let owner = app.api().addr_make("owner");
    let delegation_manager = app.api().addr_make("delegation_manager");
    let bvs_contract = app.api().addr_make("bvs_contract").to_string();

    let hash_result = bvs_directory::utils::sha256(bvs_contract.clone().as_bytes());
    let bvs_hash = hex::encode(hash_result);

    let code_id = app.store_code(bvs_registry::testing::contract());
    let (registry_addr, _) = bvs_registry::testing::instantiate(&mut app, code_id, None);

    let mut mock_env1 = mock_env::MockEnvBuilder::new(app, None, owner.clone())
        .deploy_bvs_directory(&bvs_directory::msg::InstantiateMsg {
            initial_owner: owner.clone().to_string(),
            delegation_manager: delegation_manager.into_string(),
            pauser: owner.clone().to_string(),
            unpauser: owner.clone().to_string(),
            initial_paused_status: 0,
            registry_addr: registry_addr.to_string(),
        })
        .build();

    let register_bvs_msg = &bvs_directory::msg::ExecuteMsg::RegisterBvs {
        bvs_contract: bvs_contract.clone(),
    };

    let directory = mock_env1.bvs_directory.clone();
    let response = directory
        .execute(
            &mut mock_env1,
            Addr::unchecked("anyone"),
            register_bvs_msg,
            &[],
        )
        .unwrap();

    assert_eq!(response.events.len(), 2);
    assert_eq!(
        response.events[1],
        Event::new("wasm")
            .add_attribute("_contract_address", mock_env1.bvs_directory.contract_addr)
            .add_attribute("method", "register_bvs")
            .add_attribute("bvs_hash", bvs_hash.clone())
    );
}

#[test]
fn register_bvs_but_paused() {
    let mut app = App::default();

    let owner = app.api().addr_make("owner");
    let delegation_manager = app.api().addr_make("delegation_manager");
    let bvs_contract = app.api().addr_make("bvs_contract").to_string();

    let hash_result = bvs_directory::utils::sha256(bvs_contract.clone().as_bytes());
    let bvs_hash = hex::encode(hash_result);

    let code_id = app.store_code(bvs_registry::testing::contract());
    let registry_owner = app.api().addr_make("owner").to_string();
    let (registry_addr, _) = bvs_registry::testing::instantiate(
        &mut app,
        code_id,
        bvs_registry::msg::InstantiateMsg {
            owner: registry_owner,
            initial_paused: true,
        }
        .into(),
    );

    let mut mock_env1 = mock_env::MockEnvBuilder::new(app, None, owner.clone())
        .deploy_bvs_directory(&bvs_directory::msg::InstantiateMsg {
            initial_owner: owner.clone().to_string(),
            delegation_manager: delegation_manager.into_string(),
            pauser: owner.clone().to_string(),
            unpauser: owner.clone().to_string(),
            initial_paused_status: 0,
            registry_addr: registry_addr.to_string(),
        })
        .build();

    let register_bvs_msg = &bvs_directory::msg::ExecuteMsg::RegisterBvs {
        bvs_contract: bvs_contract.clone(),
    };

    let directory = mock_env1.bvs_directory.clone();

    let err = directory
        .execute(
            &mut mock_env1,
            Addr::unchecked("anyone"),
            register_bvs_msg,
            &[],
        )
        .unwrap_err();

    assert_eq!(
        err.root_cause().to_string(),
        bvs_directory::ContractError::Std(StdError::generic_err("Paused")).to_string()
    );
}

// #[test]
// fn register_operator() {
//     let app = mock_app();

//     let private_key_hex = "af8785d6fbb939d228464a94224e986f9b1b058e583b83c16cd265fbb99ff586";
//     let (operator, secret_key, public_key_bytes) =
//         generate_babylon_public_key_from_private_key(private_key_hex);

//     let expiry = 2722875888;
//     let salt = Binary::from(b"salt");
//     let contract_addr: Addr = Addr::unchecked("bbn1l3u09t2x6ey8xcrhc5e48ygauqlxy3fa8pqsu2");
//     let owner = app.api().addr_make("owner");

//     let message_byte = bvs_directory::utils::calculate_digest_hash(
//         app.block_info().chain_id,
//         &Binary::from(public_key_bytes.clone()),
//         &owner,
//         &salt,
//         expiry,
//         &contract_addr,
//     );

//     let secp = Secp256k1::new();
//     let message = Message::from_digest_slice(&message_byte).expect("32 bytes");
//     let signature = secp.sign_ecdsa(&message, &secret_key);
//     let signature_bytes = signature.serialize_compact().to_vec();
//     let signature_base64 = general_purpose::STANDARD.encode(signature_bytes);
//     let public_key_hex = "A0IJwpjN/lGg+JTUFHJT8gF6+G7SOSBuK8CIsuv9hwvD";

//     let owner = app.api().addr_make("owner");
//     let delegation_manager = app.api().addr_make("delegation_manager");
//     let strategy_manager = app.api().addr_make("strategy_manager");
//     let slash_manager = app.api().addr_make("slash_manager");

//     println!(
//         "delegation manager111: {:?}",
//         delegation_manager.clone().into_string()
//     );

//     let mut mock_env1 = mock_env::MockEnvBuilder::new(app, None, owner.clone())
//         .deploy_bvs_directory(&bvs_directory::msg::InstantiateMsg {
//             initial_owner: owner.clone().to_string(),
//             delegation_manager: delegation_manager.into_string(),
//             pauser: owner.clone().to_string(),
//             unpauser: owner.clone().to_string(),
//             initial_paused_status: 0,
//         })
//         .deploy_bvs_delegation_manager(&bvs_delegation_manager::msg::InstantiateMsg {
//             strategy_manager: strategy_manager.into_string(),
//             slash_manager: slash_manager.into_string(),
//             min_withdrawal_delay_blocks: 0,
//             initial_owner: owner.clone().to_string(),
//             strategies: vec![],
//             withdrawal_delay_blocks: vec![],
//             pauser: owner.clone().to_string(),
//             unpauser: owner.clone().to_string(),
//             initial_paused_status: 0,
//         })
//         .build();

//     let msg = WasmMsg::Execute {
//         contract_addr: mock_env1.bvs_directory.contract_addr.to_string(),
//         msg: to_json_binary(&bvs_directory::msg::ExecuteMsg::SetDelegationManager {
//             delegation_manager: mock_env1
//                 .bvs_delegation_manager
//                 .contract_addr
//                 .clone()
//                 .into_string(),
//         })
//         .unwrap(),
//         funds: vec![],
//     };

//     let response = mock_env1.app.execute(owner.clone(), msg.into()).unwrap();
//     println!("response: {:?}", response);

//     let res: bvs_directory::query::DelegationManagerResponse = mock_env1
//         .app
//         .wrap()
//         .query_wasm_smart(
//             mock_env1.bvs_directory.contract_addr.clone(),
//             &bvs_directory::msg::QueryMsg::DelegationManager {},
//         )
//         .unwrap();

//     let msg = bvs_directory::msg::ExecuteMsg::RegisterOperatorToBvs {
//         operator: operator.to_string(),
//         public_key: public_key_hex.to_string(),
//         contract_addr: contract_addr.to_string(),
//         signature_with_salt_and_expiry: bvs_directory::msg::SignatureWithSaltAndExpiry {
//             signature: signature_base64.to_string(),
//             salt: salt.to_string(),
//             expiry,
//         },
//     };

//     let msg = WasmMsg::Execute {
//         contract_addr: mock_env1.bvs_directory.contract_addr.to_string(),
//         msg: to_json_binary(&msg).unwrap(),
//         funds: vec![],
//     };

//     let response = mock_env1
//         .app
//         .execute(Addr::unchecked("anyone"), msg.into())
//         .unwrap();
// }
