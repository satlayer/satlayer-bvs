// This file was automatically generated from pauser/schema.json.
// DO NOT MODIFY IT BY HAND.

package pauser

type CanExecuteResponse int64

type IsPausedResponse int64

type InstantiateMsg struct {
	// The initial paused state of satlayer contracts
	InitialPaused bool `json:"initial_paused"`
	// Owner of this contract, who can pause and unpause
	Owner string `json:"owner"`
}

// ExecuteMsg Pause pauses a method on a contract. Callable by the owner of the pauser
// contract
//
// ExecuteMsg UnPause unpauses a method on a contract. Callable by the owner of the pauser
// contract
//
// ExecuteMsg PauseGlobal pauses all execution. Callable by the owner of the pauser contract
// Pauses Globally: Pause all contracts and methods.
//
// ExecuteMsg UnpauseGlobal unpauses all execution. Callable by the owner of the pauser
// contract Unpauses Globally
type ExecuteMsg struct {
	Pause             *Pause             `json:"pause,omitempty"`
	Unpause           *Unpause           `json:"unpause,omitempty"`
	PauseGlobal       *PauseGlobal       `json:"pause_global,omitempty"`
	UnpauseGlobal     *UnpauseGlobal     `json:"unpause_global,omitempty"`
	TransferOwnership *TransferOwnership `json:"transfer_ownership,omitempty"`
}

type Pause struct {
	// address of the contract to be paused
	Contract string `json:"contract"`
	// method of a particular contract to be paused
	Method string `json:"method"`
}

type PauseGlobal struct {
}

type TransferOwnership struct {
	// See [`bvs_library::ownership::transfer_ownership`] for more information on this field
	NewOwner string `json:"new_owner"`
}

type Unpause struct {
	// address of the contract to be unpaused
	Contract string `json:"contract"`
	// method of a particular contract to be unpaused
	Method string `json:"method"`
}

type UnpauseGlobal struct {
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
