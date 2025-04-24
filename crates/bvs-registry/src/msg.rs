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
    /// QueryMsg Status: Returns the registration status of an operator to a service
    /// The response is a StatusResponse
    /// that contains a u8 value that maps to a RegistrationStatus:
    ///
    /// - 0: Inactive - Default state when neither the Operator nor the Service has registered,
    /// or when either has unregistered
    ///
    /// - 1: Active - State when both the Operator and Service have registered with each other,
    /// indicating a fully established relationship
    ///
    /// - 2: OperatorRegistered -
    /// State when only the Operator has registered but the Service hasn't yet,
    /// indicating a pending registration from the Service side
    ///
    /// - 3: ServiceRegistered -
    /// State when only the Service has registered but the Operator hasn't yet,
    /// indicating a pending registration from the Operator side
    #[returns(StatusResponse)]
    Status {
        service: String,
        operator: String,
        height: Option<u64>,
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
