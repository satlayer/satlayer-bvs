use cosmwasm_std::{
    from_json, testing::MockQuerier, Empty, Querier, QuerierResult, QueryRequest, StdResult,
    SystemError, SystemResult, WasmQuery,
};

use crate::bvs_delegation_manager_querier::BvsDelegationManagerQuerier;

pub struct BvsMockQuerier {
    base: MockQuerier<Empty>,
    bvs_delegation_mananger_querier: BvsDelegationManagerQuerier,
    // bvs_directory_querier: BvsDirectoryQuerier,
}

impl Querier for BvsMockQuerier {
    fn raw_query(&self, bin_request: &[u8]) -> QuerierResult {
        let request: QueryRequest<Empty> = match from_json(bin_request) {
            Ok(v) => v,
            Err(e) => {
                return SystemResult::Err(SystemError::InvalidRequest {
                    error: format!("Parsing query request: {e}"),
                    request: bin_request.into(),
                })
            }
        };

        self.handle_query(&request)
    }
}

impl BvsMockQuerier {
    pub fn new(base: MockQuerier<Empty>) -> Self {
        BvsMockQuerier {
            base,
            bvs_delegation_mananger_querier: BvsDelegationManagerQuerier::default(),
            // bvs_directory_querier: BvsDirectoryQuerier::default(),
        }
    }

    pub fn set_bvs_delegation_manager_querier_is_operator(
        &mut self,
        operator: String,
        is_operator: bool,
    ) {
        self.bvs_delegation_mananger_querier
            .is_operator
            .insert(operator, is_operator);
    }

    pub fn handle_query(&self, request: &QueryRequest<Empty>) -> QuerierResult {
        match &request {
            QueryRequest::Wasm(WasmQuery::Smart {
                contract_addr: _,
                msg,
            }) => {
                let parse_bvs_delegation_manager_query: StdResult<
                    bvs_delegation_manager::msg::QueryMsg,
                > = from_json(msg);
                if let Ok(bvs_delegation_manager_query) = parse_bvs_delegation_manager_query {
                    return self
                        .bvs_delegation_mananger_querier
                        .handle_query(bvs_delegation_manager_query);
                }

                panic!("[mock]: Unsupported wasm query: {msg:?}");
            }

            _ => self.base.handle_query(request),
        }
    }
}
