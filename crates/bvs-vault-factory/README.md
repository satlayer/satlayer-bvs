# BVS Vault Factory

The BVS Vault Factory is a contract that enables operators to deploy new vault contracts on the SatLayer ecosystem.
It serves as a centralized deployment mechanism,
ensuring that only approved vault types can be created
and that they are properly configured to work with the SatLayer infrastructure.

The Vault Factory maintains a registry of approved code IDs for different vault types,
which can be updated by the contract owner.
This ensures that only secure and compatible vault implementations can be deployed through the factory.

## Contract Functions

### Execute Functions

- `DeployCw20Vault`: Deploy a new CW20 vault for a specific CW20 token
- `DeployBankVault`: Deploy a new bank vault for a specific native token denomination
- `SetCodeId`: Set the code ID for a specific vault type (only owner can call)

### Query Functions

- `CodeId`: Get the code ID for a specific vault type

## Vault Deployment Process

The Vault Factory manages the deployment of new vaults through a controlled process:

1. The contract owner sets approved code IDs for different vault types using `SetCodeId`
2. Operators can deploy new vaults using `DeployCw20Vault` or `DeployBankVault`
3. The factory instantiates a new vault contract with the appropriate code ID
4. The factory configures the vault with the correct router address and operator
5. The new vault is ready to be whitelisted by the router for accepting deposits

### Supported Vault Types

The Vault Factory supports two main types of vaults:

1. **CW20 Vaults**: For managing CW20 token deposits and withdrawals
2. **Bank Vaults**: For managing native token deposits and withdrawals

Each vault type has its own code ID that must be set by the contract owner before deployment.
