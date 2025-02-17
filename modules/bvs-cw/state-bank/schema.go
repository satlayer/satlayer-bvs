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

package statebank

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
	InitialOwner string `json:"initial_owner"`
}

type ExecuteMsg struct {
	Set                      *Set                      `json:"set,omitempty"`
	AddRegisteredBvsContract *AddRegisteredBvsContract `json:"add_registered_bvs_contract,omitempty"`
	TransferOwnership        *TransferOwnership        `json:"transfer_ownership,omitempty"`
}

type AddRegisteredBvsContract struct {
	Address string `json:"address"`
}

type Set struct {
	Key   string `json:"key"`
	Value string `json:"value"`
}

type TransferOwnership struct {
	NewOwner string `json:"new_owner"`
}

type QueryMsg struct {
	Get Get `json:"get"`
}

type Get struct {
	Key string `json:"key"`
}
