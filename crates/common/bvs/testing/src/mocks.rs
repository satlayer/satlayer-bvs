use cosmwasm_std::{
    testing::{MockApi, MockQuerier, MockStorage, MOCK_CONTRACT_ADDR},
    Addr, Coin, OwnedDeps,
};

use crate::BvsMockQuerier;

/// mock_dependencies replacement for cosmwasm_std::testing::mock_dependencies
pub fn mock_dependencies(
    contract_balance: &[Coin],
) -> OwnedDeps<MockStorage, MockApi, BvsMockQuerier> {
    let contract_addr = Addr::unchecked(MOCK_CONTRACT_ADDR);
    let custom_querier: BvsMockQuerier = BvsMockQuerier::new(MockQuerier::new(&[(
        contract_addr.as_ref(),
        contract_balance,
    )]));

    OwnedDeps {
        storage: MockStorage::default(),
        api: MockApi::default(),
        querier: custom_querier,
        custom_query_type: Default::default(),
    }
}
