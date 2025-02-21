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
//
//    calculateDigestHashResponse, err := UnmarshalCalculateDigestHashResponse(bytes)
//    bytes, err = calculateDigestHashResponse.Marshal()
//
//    delegationManagerResponse, err := UnmarshalDelegationManagerResponse(bytes)
//    bytes, err = delegationManagerResponse.Marshal()
//
//    depositTypehashResponse, err := UnmarshalDepositTypeHashResponse(bytes)
//    bytes, err = depositTypehashResponse.Marshal()
//
//    depositsResponse, err := UnmarshalDepositsResponse(bytes)
//    bytes, err = depositsResponse.Marshal()
//
//    domainNameResponse, err := UnmarshalDomainNameResponse(bytes)
//    bytes, err = domainNameResponse.Marshal()
//
//    domainTypehashResponse, err := UnmarshalDomainTypeHashResponse(bytes)
//    bytes, err = domainTypehashResponse.Marshal()
//
//    nonceResponse, err := UnmarshalNonceResponse(bytes)
//    bytes, err = nonceResponse.Marshal()
//
//    ownerResponse, err := UnmarshalOwnerResponse(bytes)
//    bytes, err = ownerResponse.Marshal()
//
//    stakerStrategyLisResponse, err := UnmarshalStakerStrategyLisResponse(bytes)
//    bytes, err = stakerStrategyLisResponse.Marshal()
//
//    stakerStrategySharesResponse, err := UnmarshalStakerStrategySharesResponse(bytes)
//    bytes, err = stakerStrategySharesResponse.Marshal()
//
//    strategyManagerStateResponse, err := UnmarshalStrategyManagerStateResponse(bytes)
//    bytes, err = strategyManagerStateResponse.Marshal()
//
//    strategyWhitelisterResponse, err := UnmarshalStrategyWhitelisterResponse(bytes)
//    bytes, err = strategyWhitelisterResponse.Marshal()
//
//    strategyWhitelistedResponse, err := UnmarshalStrategyWhitelistedResponse(bytes)
//    bytes, err = strategyWhitelistedResponse.Marshal()
//
//    thirdPartyTransfersForbiddenResponse, err := UnmarshalThirdPartyTransfersForbiddenResponse(bytes)
//    bytes, err = thirdPartyTransfersForbiddenResponse.Marshal()
//
//    stakerStrategyListLengthResponse, err := UnmarshalStakerStrategyListLengthResponse(bytes)
//    bytes, err = stakerStrategyListLengthResponse.Marshal()

package strategymanager

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

func UnmarshalCalculateDigestHashResponse(data []byte) (CalculateDigestHashResponse, error) {
	var r CalculateDigestHashResponse
	err := json.Unmarshal(data, &r)
	return r, err
}

func (r *CalculateDigestHashResponse) Marshal() ([]byte, error) {
	return json.Marshal(r)
}

func UnmarshalDelegationManagerResponse(data []byte) (DelegationManagerResponse, error) {
	var r DelegationManagerResponse
	err := json.Unmarshal(data, &r)
	return r, err
}

func (r *DelegationManagerResponse) Marshal() ([]byte, error) {
	return json.Marshal(r)
}

func UnmarshalDepositTypeHashResponse(data []byte) (DepositTypeHashResponse, error) {
	var r DepositTypeHashResponse
	err := json.Unmarshal(data, &r)
	return r, err
}

func (r *DepositTypeHashResponse) Marshal() ([]byte, error) {
	return json.Marshal(r)
}

func UnmarshalDepositsResponse(data []byte) (DepositsResponse, error) {
	var r DepositsResponse
	err := json.Unmarshal(data, &r)
	return r, err
}

func (r *DepositsResponse) Marshal() ([]byte, error) {
	return json.Marshal(r)
}

func UnmarshalDomainNameResponse(data []byte) (DomainNameResponse, error) {
	var r DomainNameResponse
	err := json.Unmarshal(data, &r)
	return r, err
}

func (r *DomainNameResponse) Marshal() ([]byte, error) {
	return json.Marshal(r)
}

func UnmarshalDomainTypeHashResponse(data []byte) (DomainTypeHashResponse, error) {
	var r DomainTypeHashResponse
	err := json.Unmarshal(data, &r)
	return r, err
}

func (r *DomainTypeHashResponse) Marshal() ([]byte, error) {
	return json.Marshal(r)
}

func UnmarshalNonceResponse(data []byte) (NonceResponse, error) {
	var r NonceResponse
	err := json.Unmarshal(data, &r)
	return r, err
}

func (r *NonceResponse) Marshal() ([]byte, error) {
	return json.Marshal(r)
}

func UnmarshalOwnerResponse(data []byte) (OwnerResponse, error) {
	var r OwnerResponse
	err := json.Unmarshal(data, &r)
	return r, err
}

func (r *OwnerResponse) Marshal() ([]byte, error) {
	return json.Marshal(r)
}

func UnmarshalStakerStrategyLisResponse(data []byte) (StakerStrategyLisResponse, error) {
	var r StakerStrategyLisResponse
	err := json.Unmarshal(data, &r)
	return r, err
}

func (r *StakerStrategyLisResponse) Marshal() ([]byte, error) {
	return json.Marshal(r)
}

func UnmarshalStakerStrategySharesResponse(data []byte) (StakerStrategySharesResponse, error) {
	var r StakerStrategySharesResponse
	err := json.Unmarshal(data, &r)
	return r, err
}

func (r *StakerStrategySharesResponse) Marshal() ([]byte, error) {
	return json.Marshal(r)
}

func UnmarshalStrategyManagerStateResponse(data []byte) (StrategyManagerStateResponse, error) {
	var r StrategyManagerStateResponse
	err := json.Unmarshal(data, &r)
	return r, err
}

func (r *StrategyManagerStateResponse) Marshal() ([]byte, error) {
	return json.Marshal(r)
}

func UnmarshalStrategyWhitelisterResponse(data []byte) (StrategyWhitelisterResponse, error) {
	var r StrategyWhitelisterResponse
	err := json.Unmarshal(data, &r)
	return r, err
}

func (r *StrategyWhitelisterResponse) Marshal() ([]byte, error) {
	return json.Marshal(r)
}

func UnmarshalStrategyWhitelistedResponse(data []byte) (StrategyWhitelistedResponse, error) {
	var r StrategyWhitelistedResponse
	err := json.Unmarshal(data, &r)
	return r, err
}

func (r *StrategyWhitelistedResponse) Marshal() ([]byte, error) {
	return json.Marshal(r)
}

func UnmarshalThirdPartyTransfersForbiddenResponse(data []byte) (ThirdPartyTransfersForbiddenResponse, error) {
	var r ThirdPartyTransfersForbiddenResponse
	err := json.Unmarshal(data, &r)
	return r, err
}

func (r *ThirdPartyTransfersForbiddenResponse) Marshal() ([]byte, error) {
	return json.Marshal(r)
}

func UnmarshalStakerStrategyListLengthResponse(data []byte) (StakerStrategyListLengthResponse, error) {
	var r StakerStrategyListLengthResponse
	err := json.Unmarshal(data, &r)
	return r, err
}

func (r *StakerStrategyListLengthResponse) Marshal() ([]byte, error) {
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

type CalculateDigestHashResponse struct {
	DigestHash string `json:"digest_hash"`
}

type DelegationManagerResponse struct {
	DelegationManager string `json:"delegation_manager"`
}

type DepositTypeHashResponse struct {
	DepositTypeHash string `json:"deposit_type_hash"`
}

type DepositsResponse struct {
	Shares     []string `json:"shares"`
	Strategies []string `json:"strategies"`
}

type DomainNameResponse struct {
	DomainName string `json:"domain_name"`
}

type DomainTypeHashResponse struct {
	DomainTypeHash string `json:"domain_type_hash"`
}

type NonceResponse struct {
	Nonce int64 `json:"nonce"`
}

type OwnerResponse struct {
	OwnerAddr string `json:"owner_addr"`
}

type StakerStrategyLisResponse struct {
	Strategies []string `json:"strategies"`
}

type StakerStrategySharesResponse struct {
	Shares string `json:"shares"`
}

type StrategyManagerStateResponse struct {
	State StrategyManagerState `json:"state"`
}

type StrategyManagerState struct {
	DelegationManager string `json:"delegation_manager"`
	SlashManager      string `json:"slash_manager"`
	StrategyFactory   string `json:"strategy_factory"`
}

type StrategyWhitelisterResponse struct {
	Whitelister string `json:"whitelister"`
}

type StrategyWhitelistedResponse struct {
	IsWhitelisted bool `json:"is_whitelisted"`
}

type ThirdPartyTransfersForbiddenResponse struct {
	IsForbidden bool `json:"is_forbidden"`
}

type StakerStrategyListLengthResponse struct {
	StrategiesLen string `json:"strategies_len"`
}
