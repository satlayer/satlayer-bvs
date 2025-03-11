use crate::state::RegistrationStatus;
use cosmwasm_schema::{cw_serde, QueryResponses};

#[cw_serde]
pub struct InstantiateMsg {
    pub owner: String,
    pub pauser: String,
}

#[cw_serde]
pub struct OperatorDetails {
    pub staker_opt_out_window_blocks: u64,
}

#[cw_serde]
#[derive(bvs_pauser::api::Display)]
pub enum ExecuteMsg {
    RegisterAsService {
        metadata: ServiceMetadata,
    },
    ServiceUpdateMetadata(ServiceMetadata),
    RegisterAsOperator {
        operator_details: OperatorDetails,
        metadata_uri: String,
    },
    UpdateOperatorDetails {
        new_operator_details: OperatorDetails,
    },
    UpdateOperatorMetadataUri {
        metadata_uri: String,
    },
    RegisterOperatorToService {
        operator: String,
    },
    DeregisterOperatorFromService {
        operator: String,
    },
    RegisterServiceToOperator {
        service: String,
    },
    DeregisterServiceFromOperator {
        service: String,
    },
    TransferOwnership {
        /// See [`bvs_library::ownership::transfer_ownership`] for more information on this field
        new_owner: String,
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

impl From<RegistrationStatus> for StatusResponse {
    fn from(value: RegistrationStatus) -> Self {
        StatusResponse(value as u8)
    }
}

#[cfg(test)]
mod tests {
    use crate::msg::{ExecuteMsg, ServiceMetadata};

    #[test]
    fn test_method_name() {
        let msg = ExecuteMsg::RegisterOperatorToService {
            operator: "operator".to_string(),
        };
        assert_eq!(msg.to_string(), "RegisterOperatorToService");

        let msg = ExecuteMsg::ServiceUpdateMetadata(ServiceMetadata {
            name: Some("name".to_string()),
            uri: Some("uri".to_string()),
        });
        assert_eq!(msg.to_string(), "ServiceUpdateMetadata")
    }
}
