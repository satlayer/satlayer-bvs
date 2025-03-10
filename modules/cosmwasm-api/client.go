package cosmwasmapi

import (
	"context"
	"encoding/json"

	wasmtypes "github.com/CosmWasm/wasmd/x/wasm/types"
	"github.com/cosmos/cosmos-sdk/client"
	sdktypes "github.com/cosmos/cosmos-sdk/types"
)

type Client struct {
	clientCtx     client.Context
	ContractAddr  string
	gasAdjustment float64
	gasPrice      sdktypes.DecCoin
	gasLimit      uint64
}

func Query[Response interface{}](
	clientCtx client.Context, ctx context.Context, addr string, msg interface{},
) (Response, error) {
	var result Response
	queryClient := wasmtypes.NewQueryClient(clientCtx)

	queryBytes, err := json.Marshal(msg)
	if err != nil {
		return result, err
	}

	queryMsg := &wasmtypes.QuerySmartContractStateRequest{
		Address:   addr,
		QueryData: queryBytes,
	}

	response, err := queryClient.SmartContractState(ctx, queryMsg)
	if err != nil {
		return result, err
	}

	err = json.Unmarshal(response.Data, &result)
	return result, err
}
