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
//    uint64, err := UnmarshalUint64(bytes)
//    bytes, err = uint64.Marshal()
//
//    int64, err := UnmarshalInt64(bytes)
//    bytes, err = int64.Marshal()

package squaringcontract

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

type Uint64 int64

func UnmarshalUint64(data []byte) (Uint64, error) {
	var r Uint64
	err := json.Unmarshal(data, &r)
	return r, err
}

func (r *Uint64) Marshal() ([]byte, error) {
	return json.Marshal(r)
}

type Int64 int64

func UnmarshalInt64(data []byte) (Int64, error) {
	var r Int64
	err := json.Unmarshal(data, &r)
	return r, err
}

func (r *Int64) Marshal() ([]byte, error) {
	return json.Marshal(r)
}

type InstantiateMsg struct {
	Aggregator string `json:"aggregator"`
	BvsDriver  string `json:"bvs_driver"`
	StateBank  string `json:"state_bank"`
}

type ExecuteMsg struct {
	CreateNewTask *CreateNewTask `json:"create_new_task,omitempty"`
	RespondToTask *RespondToTask `json:"respond_to_task,omitempty"`
}

type CreateNewTask struct {
	Input int64 `json:"input"`
}

type RespondToTask struct {
	Operators string `json:"operators"`
	Result    int64  `json:"result"`
	TaskID    int64  `json:"task_id"`
}

type QueryMsg struct {
	GetTaskInput    *GetTaskInput    `json:"get_task_input,omitempty"`
	GetTaskResult   *GetTaskResult   `json:"get_task_result,omitempty"`
	GetLatestTaskID *GetLatestTaskID `json:"get_latest_task_id,omitempty"`
}

type GetLatestTaskID struct {
}

type GetTaskInput struct {
	TaskID int64 `json:"task_id"`
}

type GetTaskResult struct {
	TaskID int64 `json:"task_id"`
}
