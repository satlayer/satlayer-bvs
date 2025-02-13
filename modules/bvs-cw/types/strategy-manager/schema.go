// This file was generated from JSON Schema using quicktype, do not modify it directly.
// To parse and unparse this JSON data, add this code to your project and do:
//
//    instantiateMsg, err := UnmarshalInstantiateMsg(bytes)
//    bytes, err = instantiateMsg.Marshal()
//
//    executeMsg, err := UnmarshalExecuteMsg(bytes)
//    bytes, err = executeMsg.Marshal()
//
//    queryMsg, err := UnmarshalQueryMsg(bytes)
//    bytes, err = queryMsg.Marshal()

package strategy_manager

import "encoding/json"

func UnmarshalInstantiateMsg(data []byte) (InstantiateMsg, error) {
	var r InstantiateMsg
	err := json.Unmarshal(data, &r)
	return r, err
}

func (r *InstantiateMsg) Marshal() ([]byte, error) {
	return json.Marshal(r)
}

func UnmarshalExecuteMsg(data []byte) (ExecuteMsg, error) {
	var r ExecuteMsg
	err := json.Unmarshal(data, &r)
	return r, err
}

func (r *ExecuteMsg) Marshal() ([]byte, error) {
	return json.Marshal(r)
}

func UnmarshalQueryMsg(data []byte) (QueryMsg, error) {
	var r QueryMsg
	err := json.Unmarshal(data, &r)
	return r, err
}

func (r *QueryMsg) Marshal() ([]byte, error) {
	return json.Marshal(r)
}

type InstantiateMsg struct {
	DelegationManager          string `json:"delegation_manager"`
	InitialOwner               string `json:"initial_owner"`
	InitialPausedStatus        int64  `json:"initial_paused_status"`
	InitialStrategyWhitelister string `json:"initial_strategy_whitelister"`
	Pauser                     string `json:"pauser"`
	SlashManager               string `json:"slash_manager"`
	StrategyFactory            string `json:"strategy_factory"`
	Unpauser                   string `json:"unpauser"`
}

type ExecuteMsg struct {
	AddStrategiesToWhitelist         *AddStrategiesToWhitelist         `json:"add_strategies_to_whitelist,omitempty"`
	RemoveStrategiesFromWhitelist    *RemoveStrategiesFromWhitelist    `json:"remove_strategies_from_whitelist,omitempty"`
	SetStrategyWhitelister           *SetStrategyWhitelister           `json:"set_strategy_whitelister,omitempty"`
	DepositIntoStrategy              *DepositIntoStrategy              `json:"deposit_into_strategy,omitempty"`
	SetThirdPartyTransfersForbidden  *SetThirdPartyTransfersForbidden  `json:"set_third_party_transfers_forbidden,omitempty"`
	DepositIntoStrategyWithSignature *DepositIntoStrategyWithSignature `json:"deposit_into_strategy_with_signature,omitempty"`
	RemoveShares                     *RemoveShares                     `json:"remove_shares,omitempty"`
	WithdrawSharesAsTokens           *WithdrawSharesAsTokens           `json:"withdraw_shares_as_tokens,omitempty"`
	AddShares                        *AddShares                        `json:"add_shares,omitempty"`
	SetDelegationManager             *SetDelegationManager             `json:"set_delegation_manager,omitempty"`
	SetSlashManager                  *SetSlashManager                  `json:"set_slash_manager,omitempty"`
	SetStrategyFactory               *SetStrategyFactory               `json:"set_strategy_factory,omitempty"`
	TransferOwnership                *TransferOwnership                `json:"transfer_ownership,omitempty"`
	Pause                            *Pause                            `json:"pause,omitempty"`
	Unpause                          *Unpause                          `json:"unpause,omitempty"`
	SetPauser                        *SetPauser                        `json:"set_pauser,omitempty"`
	SetUnpauser                      *SetUnpauser                      `json:"set_unpauser,omitempty"`
}

type AddShares struct {
	Shares   string `json:"shares"`
	Staker   string `json:"staker"`
	Strategy string `json:"strategy"`
	Token    string `json:"token"`
}

type AddStrategiesToWhitelist struct {
	Strategies                         []string `json:"strategies"`
	ThirdPartyTransfersForbiddenValues []bool   `json:"third_party_transfers_forbidden_values"`
}

type DepositIntoStrategy struct {
	Amount   string `json:"amount"`
	Strategy string `json:"strategy"`
	Token    string `json:"token"`
}

type DepositIntoStrategyWithSignature struct {
	Amount    string `json:"amount"`
	Expiry    int64  `json:"expiry"`
	PublicKey string `json:"public_key"`
	Signature string `json:"signature"`
	Staker    string `json:"staker"`
	Strategy  string `json:"strategy"`
	Token     string `json:"token"`
}

type Pause struct {
}

type RemoveShares struct {
	Shares   string `json:"shares"`
	Staker   string `json:"staker"`
	Strategy string `json:"strategy"`
}

type RemoveStrategiesFromWhitelist struct {
	Strategies []string `json:"strategies"`
}

type SetDelegationManager struct {
	NewDelegationManager string `json:"new_delegation_manager"`
}

type SetPauser struct {
	NewPauser string `json:"new_pauser"`
}

type SetSlashManager struct {
	NewSlashManager string `json:"new_slash_manager"`
}

type SetStrategyFactory struct {
	NewStrategyFactory string `json:"new_strategy_factory"`
}

type SetStrategyWhitelister struct {
	NewStrategyWhitelister string `json:"new_strategy_whitelister"`
}

type SetThirdPartyTransfersForbidden struct {
	Strategy string `json:"strategy"`
	Value    bool   `json:"value"`
}

type SetUnpauser struct {
	NewUnpauser string `json:"new_unpauser"`
}

type TransferOwnership struct {
	NewOwner string `json:"new_owner"`
}

type Unpause struct {
}

type WithdrawSharesAsTokens struct {
	Recipient string `json:"recipient"`
	Shares    string `json:"shares"`
	Strategy  string `json:"strategy"`
	Token     string `json:"token"`
}

type QueryMsg struct {
	GetDeposits                    *GetDeposits                    `json:"get_deposits,omitempty"`
	StakerStrategyListLength       *StakerStrategyListLength       `json:"staker_strategy_list_length,omitempty"`
	GetStakerStrategyShares        *GetStakerStrategyShares        `json:"get_staker_strategy_shares,omitempty"`
	IsThirdPartyTransfersForbidden *IsThirdPartyTransfersForbidden `json:"is_third_party_transfers_forbidden,omitempty"`
	GetNonce                       *GetNonce                       `json:"get_nonce,omitempty"`
	GetStakerStrategyList          *GetStakerStrategyList          `json:"get_staker_strategy_list,omitempty"`
	GetOwner                       *GetOwner                       `json:"get_owner,omitempty"`
	IsStrategyWhitelisted          *IsStrategyWhitelisted          `json:"is_strategy_whitelisted,omitempty"`
	CalculateDigestHash            *CalculateDigestHash            `json:"calculate_digest_hash,omitempty"`
	GetStrategyWhitelister         *GetStrategyWhitelister         `json:"get_strategy_whitelister,omitempty"`
	GetStrategyManagerState        *GetStrategyManagerState        `json:"get_strategy_manager_state,omitempty"`
	GetDepositTypehash             *GetDepositTypehash             `json:"get_deposit_typehash,omitempty"`
	GetDomainTypehash              *GetDomainTypehash              `json:"get_domain_typehash,omitempty"`
	GetDomainName                  *GetDomainName                  `json:"get_domain_name,omitempty"`
	GetDelegationManager           *GetDelegationManager           `json:"get_delegation_manager,omitempty"`
}

type CalculateDigestHash struct {
	DigstHashParams QueryDigestHashParams `json:"digst_hash_params"`
}

type QueryDigestHashParams struct {
	Amount       string `json:"amount"`
	ChainID      string `json:"chain_id"`
	ContractAddr string `json:"contract_addr"`
	Expiry       int64  `json:"expiry"`
	Nonce        int64  `json:"nonce"`
	PublicKey    string `json:"public_key"`
	Staker       string `json:"staker"`
	Strategy     string `json:"strategy"`
	Token        string `json:"token"`
}

type GetDelegationManager struct {
}

type GetDepositTypehash struct {
}

type GetDeposits struct {
	Staker string `json:"staker"`
}

type GetDomainName struct {
}

type GetDomainTypehash struct {
}

type GetNonce struct {
	Staker string `json:"staker"`
}

type GetOwner struct {
}

type GetStakerStrategyList struct {
	Staker string `json:"staker"`
}

type GetStakerStrategyShares struct {
	Staker   string `json:"staker"`
	Strategy string `json:"strategy"`
}

type GetStrategyManagerState struct {
}

type GetStrategyWhitelister struct {
}

type IsStrategyWhitelisted struct {
	Strategy string `json:"strategy"`
}

type IsThirdPartyTransfersForbidden struct {
	Strategy string `json:"strategy"`
}

type StakerStrategyListLength struct {
	Staker string `json:"staker"`
}
