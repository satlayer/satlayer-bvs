package wasm

import (
	"log/slog"

	"github.com/cosmos/cosmos-sdk/codec"
	"github.com/forbole/juno/v6/modules"
	"github.com/forbole/juno/v6/types/config"

	"github.com/satlayer/satlayer-bvs/cosmwasm-indexer/database"
	wasmsource "github.com/satlayer/satlayer-bvs/cosmwasm-indexer/modules/wasm/source"
)

var (
	_ modules.Module        = &Module{}
	_ modules.MessageModule = &Module{}
)

// Module represent x/wasm module
type Module struct {
	cfg    Config
	cdc    codec.Codec
	db     *database.DB
	source wasmsource.Source
}

// NewModule returns a new Module instance
func NewModule(cfg config.Config, source wasmsource.Source, cdc codec.Codec, db *database.DB) *Module {
	bz, err := cfg.GetBytes()
	if err != nil {
		slog.Error("failed to get config bytes", "error", err)
		panic(err)
	}

	wasmCfg, err := ParseConfig(bz)
	if err != nil {
		slog.Error("failed to parse config from bytes", "error", err)
		panic(err)
	}

	return &Module{
		cfg:    wasmCfg,
		cdc:    cdc,
		db:     db,
		source: source,
	}
}

// Name implements modules.Module
func (m *Module) Name() string {
	return "wasm"
}
