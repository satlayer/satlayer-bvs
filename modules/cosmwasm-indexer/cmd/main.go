package main

import (
	"log"

	junocmd "github.com/forbole/juno/v6/cmd"
	parsetypes "github.com/forbole/juno/v6/cmd/parse/types"
	"github.com/satlayer/satlayer-bvs/cosmwasm-indexer/database"
	"github.com/satlayer/satlayer-bvs/cosmwasm-indexer/modules"
)

func main() {
	// Set up the config
	parseCfg := parsetypes.NewConfig().
		WithRegistrar(modules.NewModulesRegistrar()).
		WithDBBuilder(database.Builder)

	cfg := junocmd.NewConfig("wasmx").
		WithParseConfig(parseCfg)

	// Run the commands and panic on any error
	executor := junocmd.BuildDefaultExecutor(cfg)
	err := executor.Execute()
	if err != nil {
		log.Fatal(err)
	}
}
