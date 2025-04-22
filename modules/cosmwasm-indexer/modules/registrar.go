package modules

import (
	"github.com/cosmos/cosmos-sdk/codec"
	"github.com/forbole/juno/v6/modules/registrar"
	"github.com/satlayer/satlayer-bvs/cosmwasm-indexer/database"
	"github.com/satlayer/satlayer-bvs/cosmwasm-indexer/modules/types"
	"github.com/satlayer/satlayer-bvs/cosmwasm-indexer/modules/wasm"

	junomod "github.com/forbole/juno/v6/modules"
)

// ModulesRegistrar represents the modules.Registrar that allows to register all custom modules
type ModulesRegistrar struct {
	cdc codec.Codec
}

// NewModulesRegistrar allows to build a new ModulesRegistrar instance
func NewModulesRegistrar(cdc codec.Codec) *ModulesRegistrar {
	return &ModulesRegistrar{cdc: cdc}
}

// BuildModules implements modules.Registrar
func (r *ModulesRegistrar) BuildModules(ctx registrar.Context) junomod.Modules {
	sources, err := types.BuildSources(ctx.JunoConfig.Node, r.cdc)
	if err != nil {
		panic(err)
	}

	db := database.Cast(ctx.Database)
	wasmModule := wasm.NewModule(ctx.JunoConfig, sources.WasmSource, r.cdc, db)

	return []junomod.Module{
		wasmModule,
	}
}
