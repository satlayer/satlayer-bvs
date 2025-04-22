package types

import (
	"fmt"

	wasmtypes "github.com/CosmWasm/wasmd/x/wasm/types"
	"github.com/cosmos/cosmos-sdk/codec"
	nodeconfig "github.com/forbole/juno/v6/node/config"
	"github.com/forbole/juno/v6/node/remote"

	wasmsource "github.com/satlayer/satlayer-bvs/cosmwasm-indexer/modules/wasm/source"
	remotewasmsource "github.com/satlayer/satlayer-bvs/cosmwasm-indexer/modules/wasm/source/remote"
)

type Sources struct {
	WasmSource wasmsource.Source
}

func BuildSources(nodeCfg nodeconfig.Config, cdc codec.Codec) (*Sources, error) {
	switch cfg := nodeCfg.Details.(type) {
	case *remote.Details:
		return buildRemoteSources(cfg)
	default:
		return nil, fmt.Errorf("invalid configuration type: %T", cfg)
	}
}

func buildRemoteSources(cfg *remote.Details) (*Sources, error) {
	source, err := remote.NewSource(cfg.GRPC)
	if err != nil {
		return nil, fmt.Errorf("error while creating remote source: %s", err)
	}

	return &Sources{
		WasmSource: remotewasmsource.NewSource(source, wasmtypes.NewQueryClient(source.GrpcConn)),
	}, nil
}
