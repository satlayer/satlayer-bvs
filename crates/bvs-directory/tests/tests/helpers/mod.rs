use bech32::{self, ToBase32, Variant};
use bvs_testing::{mock_dependencies, BvsMockQuerier};
use cosmwasm_std::{
    testing::{message_info, mock_env, MockApi, MockStorage},
    Addr, Coin, Env, OwnedDeps, Uint128,
};
use cw_multi_test::{App, AppBuilder};
use ripemd::Ripemd160;
use secp256k1::{PublicKey, Secp256k1, SecretKey};
use sha2::{Digest, Sha256};

use bvs_directory::{contract::instantiate, msg::InstantiateMsg};

pub fn mock_app() -> App {
    AppBuilder::new()
        .with_api(MockApi::default().with_prefix("bbn"))
        .build(|router, _, storage| {
            router
                .bank
                .init_balance(
                    storage,
                    &Addr::unchecked("admin"),
                    vec![Coin::new(Uint128::new(100), "ubbn")],
                )
                .unwrap();
        })
}

pub(crate) fn generate_babylon_public_key_from_private_key(
    private_key_hex: &str,
) -> (Addr, SecretKey, Vec<u8>) {
    let secp = Secp256k1::new();
    let secret_key = SecretKey::from_slice(&hex::decode(private_key_hex).unwrap()).unwrap();
    let public_key = PublicKey::from_secret_key(&secp, &secret_key);
    let public_key_bytes = public_key.serialize();
    let sha256_result = Sha256::digest(public_key_bytes);
    let ripemd160_result = Ripemd160::digest(sha256_result);
    let address = bech32::encode("bbn", ripemd160_result.to_base32(), Variant::Bech32).unwrap();
    (
        Addr::unchecked(address),
        secret_key,
        public_key_bytes.to_vec(),
    )
}

pub fn setup() -> OwnedDeps<MockStorage, MockApi, BvsMockQuerier> {
    setup_with_env(mock_env())
}

pub fn setup_with_env(env: Env) -> OwnedDeps<MockStorage, MockApi, BvsMockQuerier> {
    let mut deps = mock_dependencies(&[]);

    let msg = InstantiateMsg {
        initial_owner: "owner".to_string(),
        delegation_manager: "bvs_delegation_manager".to_string(),
        pauser: "owner".to_string(),
        unpauser: "owner".to_string(),
        initial_paused_status: 0,
    };
    let info = message_info(&Addr::unchecked("owner"), &[]);
    instantiate(deps.as_mut(), env, info, msg).unwrap();

    deps
}
