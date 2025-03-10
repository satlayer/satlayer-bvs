use crate::state::RegistrationStatus;
use cosmwasm_schema::{cw_serde, QueryResponses};
use cosmwasm_std::{Addr, StdError};

#[cw_serde]
pub struct InstantiateMsg {
    pub owner: String,
    pub pauser: String,
}

#[cw_serde]
#[derive(bvs_pauser::api::Display)]
pub enum ExecuteMsg {
    ServiceRegister {
        metadata: Metadata,
    },
    ServiceUpdateMetadata(Metadata),
    ServiceRegisterOperator {
        operator: String,
    },
    ServiceDeregisterOperator {
        operator: String,
    },
    OperatorRegister {
        metadata: Metadata,
    },
    OperatorUpdateMetadata(Metadata),
    OperatorDeregisterService {
        service: String,
    },
    OperatorRegisterService {
        service: String,
    },
    TransferOwnership {
        /// See [`bvs_library::ownership::transfer_ownership`] for more information on this field
        new_owner: String,
    },
}

/// Metadata is emitted as events and not stored on-chain.
#[cw_serde]
pub struct Metadata {
    pub name: Option<String>,
    pub uri: Option<String>,
}

#[cw_serde]
#[derive(QueryResponses)]
pub enum QueryMsg {
    #[returns(RegistrationStatusResponse)]
    RegistrationStatus { service: String, operator: String },
}

#[cw_serde]
pub struct RegistrationStatusResponse(u8);

impl From<RegistrationStatus> for RegistrationStatusResponse {
    fn from(value: RegistrationStatus) -> Self {
        RegistrationStatusResponse(value as u8)
    }
}

impl From<RegistrationStatus> for u8 {
    fn from(value: RegistrationStatus) -> u8 {
        value as u8
    }
}

impl TryFrom<u8> for RegistrationStatus {
    type Error = StdError;

    fn try_from(value: u8) -> Result<Self, StdError> {
        match value {
            0 => Ok(RegistrationStatus::Inactive),
            1 => Ok(RegistrationStatus::Active),
            2 => Ok(RegistrationStatus::OperatorRegistered),
            3 => Ok(RegistrationStatus::ServiceRegistered),
            _ => Err(StdError::generic_err("RegistrationStatus out of range")),
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::msg::{ExecuteMsg, Metadata};

    #[test]
    fn test_method_name() {
        let msg = ExecuteMsg::ServiceRegisterOperator {
            operator: "operator".to_string(),
        };
        assert_eq!(msg.to_string(), "ServiceRegisterOperator");

        let msg = ExecuteMsg::ServiceUpdateMetadata(Metadata {
            name: Some("name".to_string()),
            uri: Some("uri".to_string()),
        });
        assert_eq!(msg.to_string(), "ServiceUpdateMetadata")
    }
}
