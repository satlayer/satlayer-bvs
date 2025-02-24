use super::helpers::mock_app;
use bvs_testing::integration::*;
use cosmwasm_std::{Addr, Event};

// #[test]
// fn register_bvs() {
//     let app = mock_app();

//     let owner = app.api().addr_make("owner");
//     let delegation_manager = app.api().addr_make("delegation_manager");
//     let bvs_contract = app.api().addr_make("bvs_contract").to_string();
//     let state_bank = app.api().addr_make("state_bank").to_string();
//     let bvs_driver = app.api().addr_make("bvs_driver").to_string();

//     let hash_result = bvs_directory::utils::sha256(bvs_contract.clone().as_bytes());
//     let bvs_hash = hex::encode(hash_result);

//     let mut mock_env1 = mock_env::MockEnvBuilder::new(app, None, owner.clone())
//         .deploy_bvs_directory(&bvs_directory::msg::InstantiateMsg {
//             initial_owner: owner.clone().to_string(),
//             delegation_manager: delegation_manager.into_string(),
//             state_bank: state_bank.clone(),
//             bvs_driver: bvs_driver.clone(),
//             pauser: owner.clone().to_string(),
//             unpauser: owner.clone().to_string(),
//             initial_paused_status: 0,
//         })
//         .build();

//     let register_bvs_msg = &bvs_directory::msg::ExecuteMsg::RegisterBVS {
//         bvs_contract: bvs_contract.clone(),
//     };

//     let bvs_driectory = mock_env1.bvs_directory.clone();
//     let response = bvs_driectory
//         .execute(
//             &mut mock_env1,
//             Addr::unchecked("anyone"),
//             register_bvs_msg,
//             &[],
//         )
//         .unwrap();

//     assert_eq!(response.events.len(), 2);
//     assert_eq!(
//         response.events[1],
//         Event::new("wasm")
//             .add_attribute("_contract_address", mock_env1.bvs_directory.contract_addr)
//             .add_attribute("method", "register_bvs")
//             .add_attribute("bvs_hash", bvs_hash.clone())
//     );
// }

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

//     let res: bvs_directory::query::DelegationResponse = mock_env1
//         .app
//         .wrap()
//         .query_wasm_smart(
//             mock_env1.bvs_directory.contract_addr.clone(),
//             &bvs_directory::msg::QueryMsg::GetDelegationManager {},
//         )
//         .unwrap();

//     let msg = bvs_directory::msg::ExecuteMsg::RegisterOperatorToBvs {
//         operator: operator.to_string(),
//         public_key: public_key_hex.to_string(),
//         contract_addr: contract_addr.to_string(),
//         signature_with_salt_and_expiry: bvs_directory::msg::ExecuteSignatureWithSaltAndExpiry {
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
