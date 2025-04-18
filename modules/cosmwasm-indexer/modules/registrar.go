package modules

import (
	"fmt"

	"github.com/forbole/juno/v6/modules/registrar"
	"github.com/satlayer/satlayer-bvs/cosmwasm-indexer/database"
	"github.com/satlayer/satlayer-bvs/cosmwasm-indexer/modules/wasm"

	wasmtypes "github.com/CosmWasm/wasmd/x/wasm/types"
	junomod "github.com/forbole/juno/v6/modules"
	junoremote "github.com/forbole/juno/v6/node/remote"
)

// ModulesRegistrar represents the modules.Registrar that allows to register all custom modules
type ModulesRegistrar struct {
}

// NewModulesRegistrar allows to build a new ModulesRegistrar instance
func NewModulesRegistrar() *ModulesRegistrar {
	return &ModulesRegistrar{}
}

// BuildModules implements modules.Registrar
func (r *ModulesRegistrar) BuildModules(ctx registrar.Context) junomod.Modules {
	remoteCfg, ok := ctx.JunoConfig.Node.Details.(*junoremote.Details)
	if !ok {
		panic(fmt.Errorf("invalid remote grpc config"))
	}

	grpcConnection, err := junoremote.CreateGrpcConnection(remoteCfg.GRPC)
	if err != nil {
		panic(err)
	}

	client := wasmtypes.NewQueryClient(grpcConnection)
	wasmDB := database.Cast(ctx.Database)

	return []junomod.Module{
		wasm.NewModule(wasmDB, client),
	}
}
