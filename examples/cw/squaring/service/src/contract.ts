export interface InstantiateMsg {
  registry: string;
  router: string;
  owner: string;
}

/**
 * Execute messages for the contract.
 * These messages allow modifying the state of the contract and emits event.
 */
export interface ExecuteMsg {
  request?: Request;
  respond?: Respond;
  compute?: Compute;
  register_operator?: RegisterOperator;
}

/**
 * Request for a new `input` to be computed.
 */
export interface Request {
  input: number;
}

/**
 * Respond to a `Request` with the computed `output`.
 */
export interface Respond {
  input: number;
  output: number;
}

/**
 * Compute the square of the `input` on-chain to correct the `output`.
 * The operator that responded to the request with the wrong output will be slashed.
 * This will be used to kick-start the slashing process.
 * If the operator can't be slashed,
 * the contract will still apply but the operator will not be slashed to allow of service continuity.
 */
export interface Compute {
  input: number;
  operator: string;
}

/**
 * Register the operator that will be used to compute the square of the input.
 */
export interface RegisterOperator {
  operator: string;
}

export interface QueryMsg {
  get_response: GetResponse;
}

export interface GetResponse {
  input: number;
  operator: string;
}

export type GetResponseResponse = number;
