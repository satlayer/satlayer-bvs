use crate::state::RegistrationStatus;
use cosmwasm_schema::{cw_serde, QueryResponses};

#[cw_serde]
pub struct InstantiateMsg {
    pub owner: String,
    pub registry: String,
}

#[cw_serde]
#[derive(bvs_registry::api::Display)]
pub enum ExecuteMsg {
    ServiceRegister {
        metadata: ServiceMetadata,
    },
    ServiceUpdateMetadata(ServiceMetadata),
    ServiceRegisterOperator {
        operator: String,
    },
    OperatorDeregisterService {
        service: String,
    },
    OperatorRegisterService {
        service: String,
    },
    ServiceDeregisterOperator {
        operator: String,
    },
    TransferOwnership {
        /// See `ownership::transfer_ownership` for more information on this field
        new_owner: String,
    },
    SetRouting {
        delegation_manager: String,
    },
}

/// Service metadata is emitted as events and not stored on-chain.
#[cw_serde]
pub struct ServiceMetadata {
    pub name: Option<String>,
    pub uri: Option<String>,
}

#[cw_serde]
#[derive(QueryResponses)]
pub enum QueryMsg {
    #[returns(StatusResponse)]
    Status { service: String, operator: String },
}

#[cw_serde]
pub struct StatusResponse(pub u8);

impl Into<StatusResponse> for RegistrationStatus {
    fn into(self) -> StatusResponse {
        StatusResponse(self as u8)
    }
}

#[cfg(test)]
mod tests {
    use crate::msg::{ExecuteMsg, ServiceMetadata};

    #[test]
    fn test_method_name() {
        let msg = ExecuteMsg::ServiceRegisterOperator {
            operator: "operator".to_string(),
        };
        assert_eq!(msg.to_string(), "ServiceRegisterOperator");

        let msg = ExecuteMsg::ServiceUpdateMetadata(ServiceMetadata {
            name: Some("name".to_string()),
            uri: Some("uri".to_string()),
        });
        assert_eq!(msg.to_string(), "ServiceUpdateMetadata")
    }
}
