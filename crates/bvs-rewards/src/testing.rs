#![cfg(not(target_arch = "wasm32"))]

use crate::error::RewardsError;
use crate::merkle::{Leaf, Sha3_256Algorithm};
use crate::msg::{ExecuteMsg, InstantiateMsg, QueryMsg};
use bvs_library::testing::TestingContract;
use cosmwasm_std::{Addr, Empty, Env, HexBinary};
use cw_multi_test::{App, Contract, ContractWrapper};
use rs_merkle::MerkleTree;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct RewardsContract {
    pub addr: Addr,
    pub init: InstantiateMsg,
}

impl TestingContract<InstantiateMsg, ExecuteMsg, QueryMsg> for RewardsContract {
    fn wrapper() -> Box<dyn Contract<Empty>> {
        Box::new(ContractWrapper::new(
            crate::contract::execute,
            crate::contract::instantiate,
            crate::contract::query,
        ))
    }

    fn default_init(app: &mut App, _env: &Env) -> InstantiateMsg {
        InstantiateMsg {
            owner: app.api().addr_make("owner").to_string(),
        }
    }

    fn new(app: &mut App, env: &Env, msg: Option<InstantiateMsg>) -> Self {
        let init = msg.unwrap_or(Self::default_init(app, env));
        let code_id = Self::store_code(app);
        let addr = Self::instantiate(app, code_id, "rewards", &init);
        Self { addr, init }
    }

    fn addr(&self) -> &Addr {
        &self.addr
    }
}

pub fn generate_merkle_tree(leaves: &[Leaf]) -> MerkleTree<Sha3_256Algorithm> {
    MerkleTree::<Sha3_256Algorithm>::from_leaves(
        leaves
            .iter()
            .map(|leaf| leaf.leaf_hash())
            .collect::<Vec<_>>()
            .as_slice(),
    )
}

pub fn generate_merkle_proof(
    tree: &MerkleTree<Sha3_256Algorithm>,
    leaf_index: u32,
) -> Result<Vec<HexBinary>, RewardsError> {
    // convert leaf index into usize
    let leaf_index: usize = leaf_index
        .try_into()
        .map_err(|_| RewardsError::InvalidProof {
            msg: "Leaf index is too large".to_string(),
        })?;

    let proof = tree.proof(&[leaf_index]);
    Ok(proof
        .proof_hashes()
        .iter()
        .map(|hash| HexBinary::from(hash.to_vec()))
        .collect())
}
