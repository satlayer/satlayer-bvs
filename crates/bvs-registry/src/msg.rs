use crate::state::RegistrationStatus;
use cosmwasm_schema::{cw_serde, QueryResponses};

#[cw_serde]
pub struct InstantiateMsg {
    pub owner: String,
    pub pauser: String,
}

#[cw_serde]
#[derive(bvs_pauser::api::Display)]
pub enum ExecuteMsg {
    RegisterAsService {
        metadata: Metadata,
    },
    UpdateServiceMetadata(Metadata),
    RegisterAsOperator {
        metadata: Metadata,
    },
    UpdateOperatorMetadata(Metadata),
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

/// metadata is emitted as events and not stored on-chain.
#[cw_serde]
pub struct Metadata {
    pub name: Option<String>,
    pub uri: Option<String>,
}

#[cw_serde]
#[derive(QueryResponses)]
pub enum QueryMsg {
    #[returns(StatusResponse)]
    Status { service: String, operator: String },

    #[returns(StatusResponse)]
    StatusAtHeight {
        service: String,
        operator: String,
        height: u64,
    },

    #[returns(IsServiceResponse)]
    IsService(String),

    #[returns(IsOperatorResponse)]
    IsOperator(String),

    #[returns(IsOperatorActiveResponse)]
    IsOperatorActive(String),
}

#[cw_serde]
pub struct StatusResponse(pub u8);

impl From<RegistrationStatus> for StatusResponse {
    fn from(value: RegistrationStatus) -> Self {
        StatusResponse(value as u8)
    }
}

#[cw_serde]
pub struct IsServiceResponse(pub bool);

#[cw_serde]
pub struct IsOperatorResponse(pub bool);

#[cw_serde]
pub struct IsOperatorActiveResponse(pub bool);

#[cw_serde]
pub struct MigrateMsg {}

#[cfg(test)]
mod tests {
    use crate::msg::{ExecuteMsg, Metadata};

    #[test]
    fn test_method_name() {
        let msg = ExecuteMsg::RegisterOperatorToService {
            operator: "operator".to_string(),
        };
        assert_eq!(msg.to_string(), "RegisterOperatorToService");

        let msg = ExecuteMsg::UpdateServiceMetadata(Metadata {
            name: Some("name".to_string()),
            uri: Some("uri".to_string()),
        });
        assert_eq!(msg.to_string(), "UpdateServiceMetadata");
    }
}
