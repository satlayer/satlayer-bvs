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
//    delegationManagerResponse, err := UnmarshalDelegationManagerResponse(bytes)
//    bytes, err = delegationManagerResponse.Marshal()
//
//    depositsResponse, err := UnmarshalDepositsResponse(bytes)
//    bytes, err = depositsResponse.Marshal()
//
//    stakerStrategyListResponse, err := UnmarshalStakerStrategyListResponse(bytes)
//    bytes, err = stakerStrategyListResponse.Marshal()
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
//    isTokenBlacklistedResponse, err := UnmarshalIsTokenBlacklistedResponse(bytes)
//    bytes, err = isTokenBlacklistedResponse.Marshal()
//
//    ownerResponse, err := UnmarshalOwnerResponse(bytes)
//    bytes, err = ownerResponse.Marshal()
//
//    stakerStrategyListLengthResponse, err := UnmarshalStakerStrategyListLengthResponse(bytes)
//    bytes, err = stakerStrategyListLengthResponse.Marshal()
//
//    tokenStrategyResponse, err := UnmarshalTokenStrategyResponse(bytes)
//    bytes, err = tokenStrategyResponse.Marshal()

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

func UnmarshalDelegationManagerResponse(data []byte) (DelegationManagerResponse, error) {
	var r DelegationManagerResponse
	err := json.Unmarshal(data, &r)
	return r, err
}

func (r *DelegationManagerResponse) Marshal() ([]byte, error) {
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

func UnmarshalStakerStrategyListResponse(data []byte) (StakerStrategyListResponse, error) {
	var r StakerStrategyListResponse
	err := json.Unmarshal(data, &r)
	return r, err
}

func (r *StakerStrategyListResponse) Marshal() ([]byte, error) {
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

func UnmarshalIsTokenBlacklistedResponse(data []byte) (IsTokenBlacklistedResponse, error) {
	var r IsTokenBlacklistedResponse
	err := json.Unmarshal(data, &r)
	return r, err
}

func (r *IsTokenBlacklistedResponse) Marshal() ([]byte, error) {
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

func UnmarshalStakerStrategyListLengthResponse(data []byte) (StakerStrategyListLengthResponse, error) {
	var r StakerStrategyListLengthResponse
	err := json.Unmarshal(data, &r)
	return r, err
}

func (r *StakerStrategyListLengthResponse) Marshal() ([]byte, error) {
	return json.Marshal(r)
}

func UnmarshalTokenStrategyResponse(data []byte) (TokenStrategyResponse, error) {
	var r TokenStrategyResponse
	err := json.Unmarshal(data, &r)
	return r, err
}

func (r *TokenStrategyResponse) Marshal() ([]byte, error) {
	return json.Marshal(r)
}

type InstantiateMsg struct {
	DelegationManager          string `json:"delegation_manager"`
	InitialOwner               string `json:"initial_owner"`
	InitialPausedStatus        int64  `json:"initial_paused_status"`
	InitialStrategyWhitelister string `json:"initial_strategy_whitelister"`
	Pauser                     string `json:"pauser"`
	SlashManager               string `json:"slash_manager"`
	Unpauser                   string `json:"unpauser"`
}

type ExecuteMsg struct {
	AddNewStrategy                *AddNewStrategy                `json:"add_new_strategy,omitempty"`
	BlacklistTokens               *BlacklistTokens               `json:"blacklist_tokens,omitempty"`
	AddStrategiesToWhitelist      *AddStrategiesToWhitelist      `json:"add_strategies_to_whitelist,omitempty"`
	RemoveStrategiesFromWhitelist *RemoveStrategiesFromWhitelist `json:"remove_strategies_from_whitelist,omitempty"`
	SetStrategyWhitelister        *SetStrategyWhitelister        `json:"set_strategy_whitelister,omitempty"`
	DepositIntoStrategy           *DepositIntoStrategy           `json:"deposit_into_strategy,omitempty"`
	WithdrawSharesAsTokens        *WithdrawSharesAsTokens        `json:"withdraw_shares_as_tokens,omitempty"`
	AddShares                     *AddShares                     `json:"add_shares,omitempty"`
	RemoveShares                  *RemoveShares                  `json:"remove_shares,omitempty"`
	SetDelegationManager          *SetDelegationManager          `json:"set_delegation_manager,omitempty"`
	SetSlashManager               *SetSlashManager               `json:"set_slash_manager,omitempty"`
	TransferOwnership             *TransferOwnership             `json:"transfer_ownership,omitempty"`
	Pause                         *Pause                         `json:"pause,omitempty"`
	Unpause                       *Unpause                       `json:"unpause,omitempty"`
	SetPauser                     *SetPauser                     `json:"set_pauser,omitempty"`
	SetUnpauser                   *SetUnpauser                   `json:"set_unpauser,omitempty"`
}

type AddNewStrategy struct {
	NewStrategy string `json:"new_strategy"`
	Token       string `json:"token"`
}

type AddShares struct {
	Shares   string `json:"shares"`
	Staker   string `json:"staker"`
	Strategy string `json:"strategy"`
	Token    string `json:"token"`
}

type AddStrategiesToWhitelist struct {
	Strategies []string `json:"strategies"`
}

type BlacklistTokens struct {
	Tokens []string `json:"tokens"`
}

type DepositIntoStrategy struct {
	Amount   string `json:"amount"`
	Strategy string `json:"strategy"`
	Token    string `json:"token"`
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

type SetStrategyWhitelister struct {
	NewStrategyWhitelister string `json:"new_strategy_whitelister"`
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
	GetDeposits              *GetDeposits              `json:"get_deposits,omitempty"`
	StakerStrategyListLength *StakerStrategyListLength `json:"staker_strategy_list_length,omitempty"`
	GetStakerStrategyShares  *GetStakerStrategyShares  `json:"get_staker_strategy_shares,omitempty"`
	GetStakerStrategyList    *GetStakerStrategyList    `json:"get_staker_strategy_list,omitempty"`
	Owner                    *Owner                    `json:"owner,omitempty"`
	IsStrategyWhitelisted    *IsStrategyWhitelisted    `json:"is_strategy_whitelisted,omitempty"`
	GetStrategyWhitelister   *GetStrategyWhitelister   `json:"get_strategy_whitelister,omitempty"`
	GetStrategyManagerState  *GetStrategyManagerState  `json:"get_strategy_manager_state,omitempty"`
	DelegationManager        *DelegationManager        `json:"delegation_manager,omitempty"`
	IsTokenBlacklisted       *IsTokenBlacklisted       `json:"is_token_blacklisted,omitempty"`
	TokenStrategy            *TokenStrategy            `json:"token_strategy,omitempty"`
}

type DelegationManager struct {
}

type GetDeposits struct {
	Staker string `json:"staker"`
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

type IsTokenBlacklisted struct {
	Token string `json:"token"`
}

type Owner struct {
}

type StakerStrategyListLength struct {
	Staker string `json:"staker"`
}

type TokenStrategy struct {
	Token string `json:"token"`
}

type DelegationManagerResponse struct {
	DelegationManager string `json:"delegation_manager"`
}

type DepositsResponse struct {
	Shares     []string `json:"shares"`
	Strategies []string `json:"strategies"`
}

type StakerStrategyListResponse struct {
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
}

type StrategyWhitelisterResponse struct {
	Whitelister string `json:"whitelister"`
}

type StrategyWhitelistedResponse struct {
	IsWhitelisted bool `json:"is_whitelisted"`
}

type IsTokenBlacklistedResponse struct {
	IsBlacklisted bool   `json:"is_blacklisted"`
	Token         string `json:"token"`
}

type OwnerResponse struct {
	OwnerAddr string `json:"owner_addr"`
}

type StakerStrategyListLengthResponse struct {
	StrategiesLen string `json:"strategies_len"`
}

type TokenStrategyResponse struct {
	Strategy string `json:"strategy"`
}
