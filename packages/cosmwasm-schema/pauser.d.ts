// This file was automatically generated from pauser/schema.json.
// DO NOT MODIFY IT BY HAND.

type CanExecuteResponse = number;

type IsPausedResponse = number;

export interface InstantiateMsg {
  /**
   * Initial pause state
   */
  initial_paused: boolean;
  /**
   * Owner of this contract, who can pause and unpause
   */
  owner: string;
}

export interface ExecuteMsg {
  pause?: Pause;
  unpause?: Unpause;
  transfer_ownership?: TransferOwnership;
}

export interface Pause {}

export interface TransferOwnership {
  /**
   * See [`bvs_library::ownership::transfer_ownership`] for more information on this field
   */
  new_owner: string;
}

export interface Unpause {}

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
