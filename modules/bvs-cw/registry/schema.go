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
//    canExecuteResponse, err := UnmarshalCanExecuteResponse(bytes)
//    bytes, err = canExecuteResponse.Marshal()
//
//    isPausedResponse, err := UnmarshalIsPausedResponse(bytes)
//    bytes, err = isPausedResponse.Marshal()

package registry

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

type CanExecuteResponse int64

func UnmarshalCanExecuteResponse(data []byte) (CanExecuteResponse, error) {
	var r CanExecuteResponse
	err := json.Unmarshal(data, &r)
	return r, err
}

func (r *CanExecuteResponse) Marshal() ([]byte, error) {
	return json.Marshal(r)
}

type IsPausedResponse int64

func UnmarshalIsPausedResponse(data []byte) (IsPausedResponse, error) {
	var r IsPausedResponse
	err := json.Unmarshal(data, &r)
	return r, err
}

func (r *IsPausedResponse) Marshal() ([]byte, error) {
	return json.Marshal(r)
}

type InstantiateMsg struct {
	// Initial pause state
	InitialPaused bool `json:"initial_paused"`
	// Owner of this contract, who can pause and unpause
	Owner string `json:"owner"`
}

type ExecuteMsg struct {
	Pause             *Pause             `json:"pause,omitempty"`
	Unpause           *Unpause           `json:"unpause,omitempty"`
	TransferOwnership *TransferOwnership `json:"transfer_ownership,omitempty"`
}

type Pause struct {
}

type TransferOwnership struct {
	// Transfer ownership of the contract to a new owner. Contract admin (set for all BVS
	// contracts, a cosmwasm feature) has the omni-ability to override by migration; this logic
	// is app-level. > 2-step ownership transfer is mostly redundant for CosmWasm contracts with
	// the admin set. > You can override ownership with using CosmWasm migrate `entry_point`.
	NewOwner string `json:"new_owner"`
}

type Unpause struct {
}

type QueryMsg struct {
	IsPaused   *IsPaused   `json:"is_paused,omitempty"`
	CanExecute *CanExecute `json:"can_execute,omitempty"`
}

type CanExecute struct {
	// The (contract: Addr) calling this
	C string `json:"c"`
	// The (method: ExecuteMsg) to check if it is paused
	M string `json:"m"`
	// The (sender: Addr) of the message
	S string `json:"s"`
}

type IsPaused struct {
	// The (contract: Addr) calling this
	C string `json:"c"`
	// The (method: ExecuteMsg) to check if it is paused
	M string `json:"m"`
}
