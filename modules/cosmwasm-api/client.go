package cosmwasmapi

import (
	"context"

	wasmtypes "github.com/CosmWasm/wasmd/x/wasm/types"
)

func Query(ctx context.Context) {
	queryClient := wasmtypes.NewQueryClient(c.clientCtx)
	queryMsg := &wasmtypes.QuerySmartContractStateRequest{
		Address:   opts.ContractAddr,
		QueryData: opts.QueryMsg,
	}

	resp, err := queryClient.SmartContractState(ctx, queryMsg)
	if err != nil {
		return nil, err
	}
	return resp, nil
}
