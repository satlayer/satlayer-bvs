package wasm

import (
	"github.com/satlayer/satlayer-bvs/cosmwasm-indexer/database"

	wasmtypes "github.com/CosmWasm/wasmd/x/wasm/types"
	"github.com/forbole/juno/v6/modules"
)

var (
	_ modules.Module        = &Module{}
	_ modules.MessageModule = &Module{}
)

// Module represents the x/wasm module handler
type Module struct {
	db     *database.DB
	client wasmtypes.QueryClient
}

// NewModule allows to build a new Module instance
func NewModule(db *database.DB, client wasmtypes.QueryClient) *Module {
	return &Module{
		db:     db,
		client: client,
	}
}

// Name implements modules.Module
func (m *Module) Name() string {
	return "wasm"
}
