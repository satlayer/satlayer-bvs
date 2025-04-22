package main

import (
	"log"

	junocmd "github.com/forbole/juno/v6/cmd"
	initcmd "github.com/forbole/juno/v6/cmd/init"
	parsetypes "github.com/forbole/juno/v6/cmd/parse/types"

	"github.com/satlayer/satlayer-bvs/cosmwasm-indexer/database"
	"github.com/satlayer/satlayer-bvs/cosmwasm-indexer/modules"
	"github.com/satlayer/satlayer-bvs/cosmwasm-indexer/types/config"
)

func main() {
	// Set up the config
	initCfg := initcmd.NewConfig().WithConfigCreator(config.Creator)

	parseCfg := parsetypes.NewConfig().
		WithRegistrar(modules.NewModulesRegistrar()).
		WithDBBuilder(database.Builder)

	cfg := junocmd.NewConfig("indexer").
		WithInitConfig(initCfg).
		WithParseConfig(parseCfg)

	// Run the commands and panic on any error
	executor := junocmd.BuildDefaultExecutor(cfg)
	err := executor.Execute()
	if err != nil {
		log.Fatal(err)
	}
}
