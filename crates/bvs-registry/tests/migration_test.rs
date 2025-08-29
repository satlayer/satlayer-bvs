use bvs_pauser::testing::PauserContract;
use cw_storage_plus::Map;

use bvs_library::testing::TestingContract;
use bvs_registry::msg::{ExecuteMsg, InstantiateMsg, Metadata, MigrateMsg, QueryMsg};
use cosmwasm_std::{testing::mock_env, Addr, Empty, Env};
use cw_multi_test::{App, Contract, ContractWrapper};
use serde::{Deserialize, Serialize};

pub mod bvs_registry_v3 {
    use bvs_library::ownership;
    use bvs_registry::msg::{InstantiateMsg, MigrateMsg};
    use bvs_registry::ContractError;
    use cosmwasm_std::entry_point;
    use cosmwasm_std::{DepsMut, Env, MessageInfo, Response};
    use cw2::set_contract_version;

    const CONTRACT_NAME: &str = concat!("crates.io:", env!("CARGO_PKG_NAME"));
    const CONTRACT_VERSION: &str = "3.0.0"; // next planned version

    //The following function are identical to those in bvs_registry::contract
    //But we need to bypass the !env() - loading of version that is 0.0.0
    //This make it very hard to test migration normally
    //Thus the module mock the functions with the same code but with a next planned version
    #[cfg_attr(not(feature = "library"), entry_point)]
    pub fn instantiate(
        deps: DepsMut,
        _env: Env,
        _info: MessageInfo,
        msg: InstantiateMsg,
    ) -> Result<Response, ContractError> {
        set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;

        let owner = deps.api.addr_validate(&msg.owner)?;
        ownership::set_owner(deps.storage, &owner)?;

        let pauser = deps.api.addr_validate(&msg.pauser)?;
        bvs_pauser::api::set_pauser(deps.storage, &pauser)?;

        Ok(Response::new()
            .add_attribute("method", "instantiate")
            .add_attribute("owner", owner)
            .add_attribute("pauser", pauser))
    }

    #[cfg_attr(not(feature = "library"), entry_point)]
    pub fn migrate(
        deps: DepsMut,
        _env: Env,
        _msg: Option<MigrateMsg>,
    ) -> Result<Response, ContractError> {
        let old_version =
            cw2::ensure_from_older_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;
        match old_version.major {
            2 => {
                bvs_registry::migration::fill_service_active_operators_count(deps)?;
                Ok(Response::default())
            }
            _ => Ok(Response::default()),
        }
    }
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct RegistryContractV3 {
    pub addr: Addr,
    pub init: InstantiateMsg,
}

impl TestingContract<InstantiateMsg, ExecuteMsg, QueryMsg, MigrateMsg> for RegistryContractV3 {
    fn wrapper() -> Box<dyn Contract<Empty>> {
        Box::new(
            ContractWrapper::new(
                bvs_registry::contract::execute,
                bvs_registry_v3::instantiate,
                bvs_registry::contract::query,
            )
            .with_migrate(bvs_registry_v3::migrate),
        )
    }

    fn default_init(app: &mut App, _env: &Env) -> InstantiateMsg {
        InstantiateMsg {
            owner: app.api().addr_make("owner").to_string(),
            pauser: Self::get_contract_addr(app, "pauser").to_string(),
        }
    }

    fn new(app: &mut App, env: &Env, msg: Option<InstantiateMsg>) -> Self {
        let init = msg.unwrap_or(Self::default_init(app, env));
        let code_id = Self::store_code(app);
        let addr = Self::instantiate(app, code_id, "registry", &init);
        Self { addr, init }
    }

    fn addr(&self) -> &Addr {
        &self.addr
    }
}

fn instantiate() -> (App, RegistryContractV3, PauserContract) {
    let mut app = App::default();
    let env = mock_env();

    let pauser = PauserContract::new(&mut app, &env, None);
    let registry = RegistryContractV3::new(&mut app, &env, None);

    (app, registry, pauser)
}

#[test]
fn test_migrate_service_active_operators_count() {
    let (mut app, registry, _) = instantiate();

    let services = vec![
        app.api().addr_make("service/1"),
        app.api().addr_make("service/2"),
    ];

    let operators = vec![
        app.api().addr_make("operator/1"),
        app.api().addr_make("operator/2"),
        app.api().addr_make("operator/3"),
        app.api().addr_make("operator/4"),
        app.api().addr_make("operator/5"),
        app.api().addr_make("operator/6"),
        app.api().addr_make("operator/7"),
        app.api().addr_make("operator/8"),
        app.api().addr_make("operator/9"),
        app.api().addr_make("operator/10"),
    ];

    for service in services.iter() {
        // register service
        registry
            .execute(
                &mut app,
                service,
                &ExecuteMsg::RegisterAsService {
                    metadata: Metadata {
                        name: Some(service.to_string()),
                        uri: Some(format!("https://service-{}.com", service).to_string()),
                    },
                },
            )
            .unwrap();
    }

    for operator in operators.iter() {
        // register operator
        registry
            .execute(
                &mut app,
                operator,
                &ExecuteMsg::RegisterAsOperator {
                    metadata: Metadata {
                        name: Some(operator.to_string()),
                        uri: Some(format!("https://operator-{}.com", operator).to_string()),
                    },
                },
            )
            .unwrap();
    }

    // register some operators to services
    for operator in operators.iter() {
        for service in services.iter() {
            // register operator <-> service
            registry
                .execute(
                    &mut app,
                    operator,
                    &ExecuteMsg::RegisterServiceToOperator {
                        service: service.to_string(),
                    },
                )
                .unwrap();
            registry
                .execute(
                    &mut app,
                    service,
                    &ExecuteMsg::RegisterOperatorToService {
                        operator: operator.to_string(),
                    },
                )
                .unwrap();
        }
    }

    // contract code is up to date
    // we'll have to manipulate the state to mimic pre-migration state
    {
        let mut contract_storage = app.contract_storage_mut(&registry.addr);

        let old_version = cw2::ContractVersion {
            contract: concat!("crates.io:", env!("CARGO_PKG_NAME")).to_string(),
            version: "2.0.0".to_string(),
        };

        let old_contract_info_state: cw_storage_plus::Item<cw2::ContractVersion> =
            cw_storage_plus::Item::new("contract_info");

        old_contract_info_state
            .save(&mut *contract_storage, &old_version)
            .unwrap();

        // manually reset service_active_operators_count to 0
        let old_service_active_operators_count: Map<&Addr, u64> =
            Map::new("service_active_operators_count");

        for service in services.iter() {
            old_service_active_operators_count
                .save(&mut *contract_storage, service, &0u64)
                .unwrap();
        }
    }

    let pre_counts = services
        .iter()
        .map(|service| {
            let count: u64 = registry
                .query(
                    &mut app,
                    &QueryMsg::ActiveOperatorsCount {
                        service: service.to_string(),
                    },
                )
                .unwrap();
            count
        })
        .collect::<Vec<u64>>();
    assert_eq!(pre_counts, vec![0, 0]);

    let migrate_msg = &bvs_registry::msg::MigrateMsg {};
    let admin = app.api().addr_make("admin");

    registry.migrate(&mut app, &admin, migrate_msg).unwrap();

    let count1: u64 = registry
        .query(
            &mut app,
            &QueryMsg::ActiveOperatorsCount {
                service: services[0].to_string(),
            },
        )
        .unwrap();

    assert_eq!(count1, 10);

    let count2: u64 = registry
        .query(
            &mut app,
            &QueryMsg::ActiveOperatorsCount {
                service: services[1].to_string(),
            },
        )
        .unwrap();
    assert_eq!(count2, 10);
}
