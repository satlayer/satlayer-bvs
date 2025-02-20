use cosmwasm_std::{testing::MockApi, Addr, Coin, Uint128};
use cw_multi_test::{App, AppBuilder};

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
