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
//    strategyResponse, err := UnmarshalStrategyResponse(bytes)
//    bytes, err = strategyResponse.Marshal()
//
//    blacklistStatusResponse, err := UnmarshalBlacklistStatusResponse(bytes)
//    bytes, err = blacklistStatusResponse.Marshal()

package strategyfactory

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

func UnmarshalStrategyResponse(data []byte) (StrategyResponse, error) {
	var r StrategyResponse
	err := json.Unmarshal(data, &r)
	return r, err
}

func (r *StrategyResponse) Marshal() ([]byte, error) {
	return json.Marshal(r)
}

func UnmarshalBlacklistStatusResponse(data []byte) (BlacklistStatusResponse, error) {
	var r BlacklistStatusResponse
	err := json.Unmarshal(data, &r)
	return r, err
}

func (r *BlacklistStatusResponse) Marshal() ([]byte, error) {
	return json.Marshal(r)
}

type InstantiateMsg struct {
	InitialOwner        string `json:"initial_owner"`
	InitialPausedStatus int64  `json:"initial_paused_status"`
	Pauser              string `json:"pauser"`
	StrategyCodeID      int64  `json:"strategy_code_id"`
	StrategyManager     string `json:"strategy_manager"`
	Unpauser            string `json:"unpauser"`
}

type ExecuteMsg struct {
	DeployNewStrategy               *DeployNewStrategy               `json:"deploy_new_strategy,omitempty"`
	UpdateConfig                    *UpdateConfig                    `json:"update_config,omitempty"`
	BlacklistTokens                 *BlacklistTokens                 `json:"blacklist_tokens,omitempty"`
	RemoveStrategiesFromWhitelist   *RemoveStrategiesFromWhitelist   `json:"remove_strategies_from_whitelist,omitempty"`
	SetThirdPartyTransfersForbidden *SetThirdPartyTransfersForbidden `json:"set_third_party_transfers_forbidden,omitempty"`
	WhitelistStrategies             *WhitelistStrategies             `json:"whitelist_strategies,omitempty"`
	SetStrategyManager              *SetStrategyManager              `json:"set_strategy_manager,omitempty"`
	TransferOwnership               *TransferOwnership               `json:"transfer_ownership,omitempty"`
	Pause                           *Pause                           `json:"pause,omitempty"`
	Unpause                         *Unpause                         `json:"unpause,omitempty"`
	SetPauser                       *SetPauser                       `json:"set_pauser,omitempty"`
	SetUnpauser                     *SetUnpauser                     `json:"set_unpauser,omitempty"`
}

type BlacklistTokens struct {
	Tokens []string `json:"tokens"`
}

type DeployNewStrategy struct {
	Pauser   string `json:"pauser"`
	Token    string `json:"token"`
	Unpauser string `json:"unpauser"`
}

type Pause struct {
}

type RemoveStrategiesFromWhitelist struct {
	Strategies []string `json:"strategies"`
}

type SetPauser struct {
	NewPauser string `json:"new_pauser"`
}

type SetStrategyManager struct {
	NewStrategyManager string `json:"new_strategy_manager"`
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

type UpdateConfig struct {
	NewOwner       string `json:"new_owner"`
	StrategyCodeID int64  `json:"strategy_code_id"`
}

type WhitelistStrategies struct {
	StrategiesToWhitelist              []string `json:"strategies_to_whitelist"`
	ThirdPartyTransfersForbiddenValues []bool   `json:"third_party_transfers_forbidden_values"`
}

type QueryMsg struct {
	GetStrategy        *GetStrategy        `json:"get_strategy,omitempty"`
	IsTokenBlacklisted *IsTokenBlacklisted `json:"is_token_blacklisted,omitempty"`
}

type GetStrategy struct {
	Token string `json:"token"`
}

type IsTokenBlacklisted struct {
	Token string `json:"token"`
}

type StrategyResponse struct {
	Strategy string `json:"strategy"`
}

type BlacklistStatusResponse struct {
	IsBlacklisted bool `json:"is_blacklisted"`
}
