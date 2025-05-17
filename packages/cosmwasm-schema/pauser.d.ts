// This file was automatically generated from pauser/schema.json.
// DO NOT MODIFY IT BY HAND.

type CanExecuteResponse = number;

type IsPausedResponse = number;

export interface InstantiateMsg {
  /**
   * The initial paused state of satlayer contracts
   */
  initial_paused: boolean;
  /**
   * Owner of this contract, who can pause and unpause
   */
  owner: string;
}

/**
 * ExecuteMsg Pause pauses a method on a contract. Callable by the owner of the pauser
 * contract
 *
 * ExecuteMsg UnPause unpauses a method on a contract. Callable by the owner of the pauser
 * contract
 *
 * ExecuteMsg PauseGlobal pauses all execution. Callable by the owner of the pauser contract
 * Pauses Globally: Pause all contracts and methods.
 *
 * ExecuteMsg UnpauseGlobal unpauses all execution. Callable by the owner of the pauser
 * contract Unpauses Globally
 */
export interface ExecuteMsg {
  pause?: Pause;
  unpause?: Unpause;
  pause_global?: PauseGlobal;
  unpause_global?: UnpauseGlobal;
  transfer_ownership?: TransferOwnership;
}

export interface Pause {
  /**
   * address of the contract to be paused
   */
  contract: string;
  /**
   * method of a particular contract to be paused
   */
  method: string;
}

export interface PauseGlobal {}

export interface TransferOwnership {
  /**
   * See [`bvs_library::ownership::transfer_ownership`] for more information on this field
   */
  new_owner: string;
}

export interface Unpause {
  /**
   * address of the contract to be unpaused
   */
  contract: string;
  /**
   * method of a particular contract to be unpaused
   */
  method: string;
}

export interface UnpauseGlobal {}

export interface QueryMsg {
  is_paused?: IsPaused;
  can_execute?: CanExecute;
}

export interface CanExecute {
  /**
   * The (contract: Addr) calling this
   */
  c: string;
  /**
   * The (method: ExecuteMsg) to check if it is paused
   */
  m: string;
  /**
   * The (sender: Addr) of the message
   */
  s: string;
}

export interface IsPaused {
  /**
   * The (contract: Addr) calling this
   */
  c: string;
  /**
   * The (method: ExecuteMsg) to check if it is paused
   */
  m: string;
}
