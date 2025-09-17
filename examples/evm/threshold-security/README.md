# Threshold Security (EVM)

This example demonstrates a simple threshold-weighted verification service (BVS) where multiple operators submit a computed result (square of a number). A request is finalized only when the total USD AUM (Assets Under Management) of the operators that agreed on the same result meets a configured threshold.

## Key ideas

- Weighted consensus by AUM: Each operator’s vote is weighted by their AUM in USD using `SLAYOracle`.
- Finalization threshold: Only when the sum of AUM for the same output reaches the threshold does the request finalize.
- Relationship-gated responses: Only operators with an active relationship to the service can respond.

read more about [`SLAYOracle`](https://build.satlayer.xyz/evm/contracts/extension/ISLAYOracle.sol).

## Structure

- `src/BVS.sol`: Core contract maintained by the Service (owner) implementing request/response, AUM weighting via SLAYOracle, and threshold finalization.
- `src/operatorNode.ts`: Minimal Operator node that listens for `Requested` and calls `respond` with the square.
- `src/lifecycle.test.ts`: End-to-end lifecycle test that is meant to showcase the flow of threshold security.

## How it works (flow)

If you prefer reading code, look at `src/lifecycle.test.ts`.

### Setups

1. The Service deploys `BVS.sol` and register it as a service on the SatLayer Registry.
2. The Operators register as operators on the SatLayer Registry.
3. The Operators creates a Vault.
4. The Service enables slashing on the Registry.
5. The Service and Operators establish active relationships on the Registry.

### Core flow

1. The Service requests a work (number to be squared) to the contract through `request(num)` and emits `Requested`.
2. The Operators listen to `Requested` and respond with `respond(requestId, num*num)`.
3. For each response, the contract records the response and the operator's AUM in USD through `SLAYOracle`.
4. Anyone can call `finalize(requestId, output)` once the response has hit the threshold.

## Run the example

1. Install dependencies: `pnpm install`
2. Build: `pnpm build`
3. Test: `pnpm test` to run the `lifecycle.test.ts`

Notes

- Slashing: `enableSlashing` wires the service to registry-managed slashing. This example shows the setup; the challenge flow is outside the scope of this sample.
- Prices and AUM: In the test, vault deposits and oracle prices together determine operators’ AUM in USD, which the SLAYOracle exposes to the BVS.
- The prices in the test are fixed but in reality, prices fluctuate over time. It might be a good idea to expect responses from Operator within a certain timeframe.
