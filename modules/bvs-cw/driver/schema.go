// This file was generated from JSON Schema using quicktype, do not modify it directly.
// To parse and unparse this JSON data, add this code to your project and do:
//
//    instantiateMsg, err := UnmarshalInstantiateMsg(bytes)
//    bytes, err = instantiateMsg.Marshal()
//
//    executeMsg, err := UnmarshalExecuteMsg(bytes)
//    bytes, err = executeMsg.Marshal()

package driver

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

type InstantiateMsg struct {
	InitialOwner string `json:"initial_owner"`
}

type ExecuteMsg struct {
	ExecuteBvsOffchain       *ExecuteBvsOffchain       `json:"execute_bvs_offchain,omitempty"`
	AddRegisteredBvsContract *AddRegisteredBvsContract `json:"add_registered_bvs_contract,omitempty"`
	TransferOwnership        *TransferOwnership        `json:"transfer_ownership,omitempty"`
}

type AddRegisteredBvsContract struct {
	Address string `json:"address"`
}

type ExecuteBvsOffchain struct {
	TaskID string `json:"task_id"`
}

type TransferOwnership struct {
	NewOwner string `json:"new_owner"`
}
