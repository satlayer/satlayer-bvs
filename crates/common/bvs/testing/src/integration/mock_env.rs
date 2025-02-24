use cosmwasm_std::{coin, Addr, Coin, StdResult, Uint128};
use cw_multi_test::{error::AnyResult, App, AppResponse, BankSudo, BasicApp, Executor, SudoMsg};
use serde::Serialize;
use std::fmt::Debug;
use std::{collections::HashMap, mem::take};

use crate::integration::mock_contracts::{mock_bvs_delegation_manager, mock_bvs_directory};

pub struct MockEnv {
    pub app: App,
    pub owner: Addr,
    pub bvs_delegation_manager: BvsDelegationManager,
    pub bvs_directory: BvsDirectory,
}

#[derive(Clone)]
pub struct BvsDelegationManager {
    pub contract_addr: Addr,
}

#[derive(Clone)]
pub struct BvsDirectory {
    pub contract_addr: Addr,
}

impl MockEnv {
    pub fn increment_by_blocks(&mut self, num_of_blocks: u64) {
        self.app.update_block(|block| {
            block.height += num_of_blocks;
            // assume block time = 6 sec
            block.time = block.time.plus_seconds(num_of_blocks * 6);
        })
    }

    pub fn increment_by_time(&mut self, seconds: u64) {
        self.app.update_block(|block| {
            block.height += seconds / 6;
            // assume block time = 6 sec
            block.time = block.time.plus_seconds(seconds);
        })
    }

    pub fn fund_accounts(&mut self, addrs: &[&Addr], amount: u128, denoms: &[&str]) {
        for addr in addrs {
            let coins: Vec<_> = denoms.iter().map(|&d| coin(amount, d)).collect();
            self.fund_account(addr, &coins);
        }
    }

    pub fn fund_account(&mut self, addr: &Addr, coins: &[Coin]) {
        self.app
            .sudo(SudoMsg::Bank(BankSudo::Mint {
                to_address: addr.to_string(),
                amount: coins.to_vec(),
            }))
            .unwrap();
    }

    pub fn query_balance(&self, addr: &Addr, denom: &str) -> StdResult<Coin> {
        self.app.wrap().query_balance(addr, denom)
    }

    pub fn query_all_balances(&self, addr: &Addr) -> HashMap<String, Uint128> {
        let res: Vec<Coin> = self.app.wrap().query_all_balances(addr).unwrap();
        res.into_iter().map(|r| (r.denom, r.amount)).collect()
    }
}

impl BvsDelegationManager {
    pub fn execute<T: Serialize + Debug>(
        &self,
        env: &mut MockEnv,
        sender: Addr,
        msg: &T,
        send_funds: &[Coin],
    ) -> AnyResult<AppResponse> {
        env.app
            .execute_contract(sender, self.contract_addr.clone(), msg, send_funds)
    }

    pub fn migrate<T: Serialize + Debug>(
        &self,
        env: &mut MockEnv,
        sender: Addr,
        msg: &T,
        new_code_id: u64,
    ) -> AnyResult<AppResponse> {
        env.app
            .migrate_contract(sender, self.contract_addr.clone(), msg, new_code_id)
    }
}

impl BvsDirectory {
    pub fn execute<T: Serialize + Debug>(
        &self,
        env: &mut MockEnv,
        sender: Addr,
        msg: &T,
        send_funds: &[Coin],
    ) -> AnyResult<AppResponse> {
        env.app
            .execute_contract(sender, self.contract_addr.clone(), msg, send_funds)
    }

    pub fn migrate<T: Serialize + Debug>(
        &self,
        env: &mut MockEnv,
        sender: Addr,
        msg: &T,
        new_code_id: u64,
    ) -> AnyResult<AppResponse> {
        env.app
            .migrate_contract(sender, self.contract_addr.clone(), msg, new_code_id)
    }
}

pub struct MockEnvBuilder {
    app: BasicApp,
    admin: Option<String>,
    owner: Addr,
    bvs_delegation_manager: Addr,
    bvs_directory: Addr,
}

impl MockEnvBuilder {
    pub fn new(app: BasicApp, admin: Option<String>, owner: Addr) -> Self {
        Self {
            app,
            admin,
            owner,
            bvs_delegation_manager: Addr::unchecked(""),
            bvs_directory: Addr::unchecked(""),
        }
    }

    pub fn build(mut self) -> MockEnv {
        MockEnv {
            app: take(&mut self.app),
            owner: self.owner.clone(),
            bvs_delegation_manager: BvsDelegationManager {
                contract_addr: self.bvs_delegation_manager,
            },
            bvs_directory: BvsDirectory {
                contract_addr: self.bvs_directory,
            },
        }
    }

    pub fn deploy_bvs_delegation_manager(
        mut self,
        instantiate_msg: &bvs_delegation_manager::msg::InstantiateMsg,
    ) -> Self {
        let code_id = self.app.store_code(mock_bvs_delegation_manager());

        self.bvs_delegation_manager = self
            .app
            .instantiate_contract(
                code_id,
                self.owner.clone(),
                instantiate_msg,
                &[],
                "bvs_delegation_manager",
                None,
            )
            .unwrap();
        self
    }

    pub fn deploy_bvs_directory(
        mut self,
        instantiate_msg: &bvs_directory::msg::InstantiateMsg,
    ) -> Self {
        let code_id = self.app.store_code(mock_bvs_directory());

        self.bvs_directory = self
            .app
            .instantiate_contract(
                code_id,
                self.owner.clone(),
                &instantiate_msg,
                &[],
                "bvs_directory",
                self.admin.clone(),
            )
            .unwrap();
        self
    }
}
