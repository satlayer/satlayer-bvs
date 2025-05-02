package wasm

import (
	"log"
	"log/slog"
	"slices"

	"github.com/cosmos/cosmos-sdk/codec"
	"github.com/forbole/juno/v6/modules"
	"github.com/forbole/juno/v6/types/config"

	"github.com/satlayer/satlayer-bvs/cosmwasm-indexer/database"
	"github.com/satlayer/satlayer-bvs/cosmwasm-indexer/utils"
)

var (
	_ modules.Module        = &Module{}
	_ modules.MessageModule = &Module{}
)

// Module represent x/wasm module
type Module struct {
	cfg *Config
	cdc codec.Codec
	db  *database.DB
}

// NewModule returns a new Module instance
func NewModule(cfg config.Config, cdc codec.Codec, db *database.DB) *Module {
	bz, err := cfg.GetBytes()
	if err != nil {
		slog.Error("Failed to get config bytes", "error", err)
		log.Fatal(err)
	}

	wasmCfg, err := ParseConfig(bz)
	if err != nil {
		slog.Error("Failed to parse config from bytes", "error", err)
		log.Fatal(err)
	}

	validateWASMConfig(wasmCfg, cfg.Chain.Bech32Prefix)

	if len(wasmCfg.CodeIDs) != 0 {
		// sort codeIDs in config, if they're present
		slices.Sort(wasmCfg.CodeIDs)
	}

	slog.Info("Run wasm module", "wasm config", wasmCfg)

	return &Module{
		cfg: wasmCfg,
		cdc: cdc,
		db:  db,
	}
}

// Name implements modules.Module
func (m *Module) Name() string {
	return "wasm"
}

func validateWASMConfig(wasmCfg *Config, bech32prefix string) {
	if wasmCfg == nil {
		log.Fatal("The config of wasm module shouldn't be nil")
	}

	if len(wasmCfg.Contracts) == 0 {
		log.Fatal("Contracts shouldn't be empty")
	}

	for addr := range wasmCfg.Contracts {
		if ok := utils.IsValidContractAddr(addr, bech32prefix); !ok {
			log.Fatalf("Invalid contract address: %s", addr)
		}
	}
}
